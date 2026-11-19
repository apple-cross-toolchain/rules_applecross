#!/usr/bin/env python3
"""Checks that every Swift async function in an arm64 Mach-O gets as large an async context as
its code uses.

A Swift async function `F` has an async function pointer `FTu`: a relative offset to `F` and
the size of the context its callers allocate for it. When two object files carry different
definitions of the same weak async function (one module built for macOS 10.13 and another for
15.0 both specialising the standard library's `Clock.sleep`, say), a linker may keep the pointer
from one and the function from the other. The function then writes past the context its
callers allocated, which corrupts the task allocator: `swift_task_dealloc` aborts at run time.
Xcode 27.2's Swift and ld built Visor 0.62.1 that way.

This reads each `…Tu` in the binary, finds the function it points to and that function's resume
partials (`…TQ<n>_`, `…TY<n>_`), decodes their loads and stores relative to x22 (the async
context register), and reports any function that uses more of its context than its pointer
allocates. It reads the Mach-O itself: no Xcode or LLVM tools needed, on any host.

    bazel run @rules_applecross//tools:check_async_frames -- <App.app | binary>...

Exit status 1 when any function's context is too small, 0 otherwise.
"""

import os
import re
import struct
import sys

CPU_ARM64 = 0x0100000C
LC_SEGMENT_64, LC_SYMTAB, LC_FUNCTION_STARTS = 0x19, 0x2, 0x26
N_SECT, N_TYPE, N_STAB = 0x0E, 0x0E, 0xE0
PARTIAL = re.compile(r"^T[QY]\d+_$")


def arm64_slice(data):
    """The arm64 Mach-O in `data` (a thin one, or a universal binary's slice), or None."""
    if data[:4] == b"\xcf\xfa\xed\xfe":
        cputype, = struct.unpack_from("<I", data, 4)
        return data if cputype == CPU_ARM64 else None
    if data[:4] == b"\xca\xfe\xba\xbe":  # universal, big-endian headers
        nfat, = struct.unpack_from(">I", data, 4)
        for i in range(nfat):
            cputype, _, offset, size, _ = struct.unpack_from(">IIIII", data, 8 + i * 20)
            if cputype == CPU_ARM64:
                return data[offset:offset + size]
    return None


class MachO:
    def __init__(self, data):
        self.data = data
        ncmds, = struct.unpack_from("<I", data, 16)
        self.segments, self.sections, at, symtab, starts = [], [], 32, None, None
        for _ in range(ncmds):
            cmd, size = struct.unpack_from("<II", data, at)
            if cmd == LC_SEGMENT_64:
                vmaddr, vmsize, fileoff = struct.unpack_from("<QQQ", data, at + 24)
                self.segments.append((vmaddr, vmsize, fileoff))
                nsects, = struct.unpack_from("<I", data, at + 64)
                for i in range(nsects):
                    s = at + 72 + i * 80
                    sectname = data[s:s + 16].rstrip(b"\0").decode()
                    segname = data[s + 16:s + 32].rstrip(b"\0").decode()
                    addr, sz = struct.unpack_from("<QQ", data, s + 32)
                    self.sections.append((segname, sectname, addr, sz))
            elif cmd == LC_SYMTAB:
                symtab = struct.unpack_from("<IIII", data, at + 8)
            elif cmd == LC_FUNCTION_STARTS:
                starts = struct.unpack_from("<II", data, at + 8)
            at += size
        # defined symbols: name -> address, and every address in a section, sorted
        self.symbols, self.text = {}, set()
        if symtab:
            symoff, nsyms, stroff, _ = symtab
            for i in range(nsyms):
                n_strx, n_type, n_sect, _, n_value = struct.unpack_from("<IBBHQ", data, symoff + i * 16)
                if n_type & N_STAB or n_type & N_TYPE != N_SECT or n_sect == 0:
                    continue
                end = data.index(b"\0", stroff + n_strx)
                name = data[stroff + n_strx:end].decode(errors="replace")
                self.symbols[name] = n_value
                self.text.add(n_value)
        # where each function ends: the next function's start, from LC_FUNCTION_STARTS (every
        # function, symbol or not: a linker may drop a local one's symbol), symbols and section ends
        functions = set()
        if starts:
            off, size = starts
            addr = next((vm for vm, _, fo in self.segments if fo == 0 and vm), 0)
            at, end = off, off + size
            while at < end:
                delta, shift = 0, 0
                while True:
                    byte = data[at]; at += 1
                    delta |= (byte & 0x7F) << shift; shift += 7
                    if byte < 0x80:
                        break
                if delta == 0:
                    break
                addr += delta
                functions.add(addr)
        self.starts = sorted(self.text | functions | {addr + sz for _, _, addr, sz in self.sections})

    def read(self, addr, n):
        for vmaddr, vmsize, fileoff in self.segments:
            if vmaddr <= addr < vmaddr + vmsize:
                return self.data[fileoff + addr - vmaddr:fileoff + addr - vmaddr + n]
        return None

    def extent(self, addr):
        """A function's bytes: from its address to the next symbol or section end."""
        import bisect
        i = bisect.bisect_right(self.starts, addr)
        end = self.starts[i] if i < len(self.starts) else addr
        return self.read(addr, end - addr) or b""


def x22_extent(code):
    """The furthest byte of the async context (x22) that `code` loads or stores. A load into x22
    itself is the context register reloaded (a resume partial finding its own context through
    the one it was given), not a field of this function's context: it doesn't count."""
    furthest = 0
    for (insn,) in struct.iter_unpack("<I", code[:len(code) // 4 * 4]):
        if (insn >> 5) & 31 != 22:
            continue
        pair, simd, opc = insn & 0x3B800000 == 0x29000000, (insn >> 26) & 1, (insn >> 22) & 3
        # LDP's L bit; LDR's opc: any non-zero for a general register, the low bit for a SIMD one
        is_load = (insn >> 22) & 1 if pair else (opc & 1 if simd else opc != 0)
        if is_load and (insn & 31 == 22 or (insn & 0x3B800000 == 0x29000000 and (insn >> 10) & 31 == 22)):
            continue
        if insn & 0x3B000000 == 0x39000000:  # LDR/STR (unsigned immediate)
            size, v, opc = insn >> 30, (insn >> 26) & 1, (insn >> 22) & 3
            width = 16 if v and size == 0 and opc & 2 else 1 << size
            offset = ((insn >> 10) & 0xFFF) * width
            furthest = max(furthest, offset + width)
        elif insn & 0x3B200C00 == 0x38000000:  # LDUR/STUR (unscaled)
            size, v, opc = insn >> 30, (insn >> 26) & 1, (insn >> 22) & 3
            width = 16 if v and size == 0 and opc & 2 else 1 << size
            imm9 = (insn >> 12) & 0x1FF
            offset = imm9 - 0x200 if imm9 & 0x100 else imm9
            furthest = max(furthest, offset + width)
        elif insn & 0x3B800000 == 0x29000000:  # LDP/STP (signed offset)
            opc, v = insn >> 30, (insn >> 26) & 1
            width = (4, 8, 16)[opc] if v else (8 if opc == 2 else 4)
            imm7 = (insn >> 15) & 0x7F
            offset = (imm7 - 0x80 if imm7 & 0x40 else imm7) * width
            furthest = max(furthest, offset + 2 * width)
    return furthest


def check(path):
    """(checked, [(function, allocated, used)]) for one Mach-O file, or None if it isn't arm64."""
    m = arm64_slice(open(path, "rb").read())
    if m is None:
        return None
    macho = MachO(m)
    partials = {}
    for name in macho.symbols:
        cut = re.search(r"T[QY]\d+_$", name)
        if cut:
            partials.setdefault(name[:cut.start()], []).append(name)
    checked, short = 0, []
    for name, addr in macho.symbols.items():
        if not name.endswith("Tu"):
            continue
        function = name[:-2]
        target = macho.symbols.get(function)
        pointer = macho.read(addr, 8)
        if target is None or pointer is None:
            continue
        relative, allocated = struct.unpack("<iI", pointer)
        if addr + relative != target:
            continue  # not an async function pointer to `function`
        checked += 1
        used = max(x22_extent(macho.extent(macho.symbols[f])) for f in [function] + partials.get(function, []))
        if used > allocated:
            short.append((function, allocated, used))
    return checked, short


def binaries(path):
    if os.path.isfile(path):
        yield path
        return
    for dirpath, _, files in os.walk(path):
        for f in files:
            p = os.path.join(dirpath, f)
            if not os.path.islink(p):
                with open(p, "rb") as fh:
                    if fh.read(4) in (b"\xcf\xfa\xed\xfe", b"\xca\xfe\xba\xbe"):
                        yield p


def main(argv):
    if not argv:
        print(__doc__.strip().splitlines()[-3].strip(), file=sys.stderr)
        return 2
    bad = False
    for arg in argv:
        for path in binaries(os.path.abspath(arg)):
            result = check(path)
            if result is None:
                continue
            checked, short = result
            for function, allocated, used in short:
                bad = True
                print(f"{path}: {function}: its async function pointer allocates {allocated} bytes of context; its code uses {used}")
            print(f"{path}: {checked} async functions, {len(short)} with too small a context")
    return 1 if bad else 0


if __name__ == "__main__":
    os.chdir(os.environ.get("BUILD_WORKING_DIRECTORY", "."))
    sys.exit(main(sys.argv[1:]))
