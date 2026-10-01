#!/usr/bin/env python3
"""Runs an unhosted iOS XCTest bundle inside darwin-vm."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import plistlib
import pty
import re
import select
import shlex
import shutil
import signal
import struct
import subprocess
import sys
import tempfile
import time
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path


_MACHO_MAGIC_64 = 0xFEEDFACF
_CPU_TYPE_ARM64 = 0x0100000C
_LC_SEGMENT_64 = 0x19
_LC_CODE_SIGNATURE = 0x1D
_LC_LOAD_DYLIB = 0xC
_LC_LOAD_WEAK_DYLIB = 0x80000018
_LC_REEXPORT_DYLIB = 0x8000001F
_LC_LAZY_LOAD_DYLIB = 0x20
_LC_LOAD_UPWARD_DYLIB = 0x80000023
_LC_DYLD_INFO_ONLY = 0x80000022
_LC_DYLD_CHAINED_FIXUPS = 0x80000034
_MH_EXECUTE = 0x2
_UNAVAILABLE_DYLIBS = {
    # rules_apple links these even when a logic test imports no symbols from
    # them. UIKit is absent from the restore image, while its standalone
    # libc++ is arm64e-only and cannot load into the arm64 XCTest process.
    b"/System/Library/Frameworks/UIKit.framework/UIKit",
    b"/usr/lib/libc++.1.dylib",
}
_PROMPT_RE = re.compile(rb"(?:^|[\r\n])# ")
_PANIC_RE = re.compile(
    rb"panic_with_register_state: \[DISPATCH_ID_SPTM_CORE\] \[TXM\]|"
    rb"nested panic count exceeds limit",
    re.IGNORECASE,
)
_BOOT_EVENT_RE = re.compile(
    rb"(?:^|[\r\n])# |"
    rb"panic_with_register_state: \[DISPATCH_ID_SPTM_CORE\] \[TXM\]|"
    rb"Nested panic count exceeds limit",
    re.IGNORECASE,
)
_EXIT_RE = re.compile(rb"__DARWIN_VM_XCTEST_EXIT__([0-9]+)")
_TEST_EVENT_RE = re.compile(
    rb"__DARWIN_VM_XCTEST_EXIT__[0-9]+|"
    rb"panic_with_register_state: \[DISPATCH_ID_SPTM_CORE\] \[TXM\]|"
    rb"Nested panic count exceeds limit",
    re.IGNORECASE,
)
_ENV_NAME_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
_SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


class RunnerError(RuntimeError):
    pass


def _read_uleb(data: bytearray, offset: int, end: int) -> tuple[int, int]:
    value = 0
    shift = 0
    start = offset
    while offset < end:
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return value, offset - start
        shift += 7
    raise RunnerError("truncated dyld bind opcode")


def _skip_sleb(data: bytearray, offset: int, end: int) -> int:
    start = offset
    while offset < end:
        byte = data[offset]
        offset += 1
        if not byte & 0x80:
            return offset - start
    raise RunnerError("truncated dyld bind opcode")


def _rewrite_bind_ordinals(
    data: bytearray,
    offset: int,
    size: int,
    removed_ordinals: set[int],
) -> None:
    end = offset + size
    if offset < 0 or size < 0 or end > len(data):
        raise RunnerError("invalid dyld bind stream")
    while offset < end:
        opcode = data[offset] & 0xF0
        immediate = data[offset] & 0x0F
        offset += 1
        if opcode == 0x00 or opcode == 0x30 or opcode == 0x50 or opcode == 0x90 or opcode == 0xB0:
            continue
        if opcode == 0x10:
            if immediate in removed_ordinals:
                raise RunnerError("test binary imports symbols from UIKit or libc++")
            replacement = immediate - sum(value < immediate for value in removed_ordinals)
            data[offset - 1] = opcode | replacement
        elif opcode == 0x20:
            ordinal, length = _read_uleb(data, offset, end)
            if ordinal in removed_ordinals:
                raise RunnerError("test binary imports symbols from UIKit or libc++")
            replacement = ordinal - sum(value < ordinal for value in removed_ordinals)
            if length != 1 or replacement >= 0x80:
                raise RunnerError("unsupported multi-byte dylib ordinal")
            data[offset] = replacement
            offset += length
        elif opcode == 0x40:
            terminator = data.find(0, offset, end)
            if terminator < 0:
                raise RunnerError("truncated dyld symbol name")
            offset = terminator + 1
        elif opcode == 0x60:
            offset += _skip_sleb(data, offset, end)
        elif opcode in (0x70, 0x80, 0xA0):
            _, length = _read_uleb(data, offset, end)
            offset += length
        elif opcode == 0xC0:
            _, length = _read_uleb(data, offset, end)
            offset += length
            _, length = _read_uleb(data, offset, end)
            offset += length
        elif opcode == 0xD0:
            if immediate == 0:
                _, length = _read_uleb(data, offset, end)
                offset += length
        else:
            raise RunnerError(f"unsupported dyld bind opcode 0x{opcode:02x}")


def _remove_unavailable_dylibs(data: bytearray) -> None:
    """Removes unused rules_apple UIKit/libc++ edges and fixes ordinals."""
    command_count, command_bytes = struct.unpack_from("<II", data, 16)
    offset = 32
    commands_end = offset + command_bytes
    if commands_end > len(data):
        raise RunnerError("truncated Mach-O load commands")
    commands = []
    dylib_ordinal = 0
    removed_ordinals = set()
    dyld_info = None
    for _ in range(command_count):
        if offset + 8 > commands_end:
            raise RunnerError("truncated Mach-O load command")
        command, size = struct.unpack_from("<II", data, offset)
        if size < 8 or offset + size > commands_end:
            raise RunnerError("invalid Mach-O load command")
        remove = False
        if command == _LC_DYLD_CHAINED_FIXUPS:
            raise RunnerError(
                "test binary uses LC_DYLD_CHAINED_FIXUPS; link with -no_fixup_chains",
            )
        if command == _LC_DYLD_INFO_ONLY:
            dyld_info = offset
        if command in (
            _LC_LOAD_DYLIB,
            _LC_LOAD_WEAK_DYLIB,
            _LC_REEXPORT_DYLIB,
            _LC_LAZY_LOAD_DYLIB,
            _LC_LOAD_UPWARD_DYLIB,
        ):
            dylib_ordinal += 1
            name_offset = struct.unpack_from("<I", data, offset + 8)[0]
            name_start = offset + name_offset
            name_end = data.find(0, name_start, offset + size)
            if name_offset < 24 or name_start >= offset + size or name_end < 0:
                raise RunnerError("invalid Mach-O dylib load command")
            original = bytes(data[name_start:name_end])
            if original in _UNAVAILABLE_DYLIBS:
                remove = True
                removed_ordinals.add(dylib_ordinal)
        commands.append((offset, size, remove))
        offset += size

    if not removed_ordinals:
        return
    if dyld_info is None:
        raise RunnerError("test binary has no LC_DYLD_INFO_ONLY command")
    _, _, _, _, bind_offset, bind_size, _, _, lazy_offset, lazy_size, _, _ = \
        struct.unpack_from("<12I", data, dyld_info)
    _rewrite_bind_ordinals(data, bind_offset, bind_size, removed_ordinals)
    _rewrite_bind_ordinals(data, lazy_offset, lazy_size, removed_ordinals)

    replacement_commands = b"".join(
        data[command_offset:command_offset + size]
        for command_offset, size, remove in commands
        if not remove
    )
    old_end = 32 + command_bytes
    data[32:32 + len(replacement_commands)] = replacement_commands
    data[32 + len(replacement_commands):old_end] = b"\0" * (
        command_bytes - len(replacement_commands)
    )
    struct.pack_into(
        "<II",
        data,
        16,
        command_count - len(removed_ordinals),
        len(replacement_commands),
    )


def _load_commands(
    data: bytearray,
) -> tuple[int, int, int, int | None, int, int, int]:
    if len(data) < 32 or struct.unpack_from("<I", data, 0)[0] != _MACHO_MAGIC_64:
        raise RunnerError("test executable must be a thin 64-bit Mach-O")
    if struct.unpack_from("<I", data, 4)[0] != _CPU_TYPE_ARM64:
        raise RunnerError("test executable must target device arm64")

    file_type = struct.unpack_from("<I", data, 12)[0]
    command_count, command_size = struct.unpack_from("<II", data, 16)
    offset = 32
    linkedit = None
    text = None
    code_signature = None
    first_section = len(data)

    for _ in range(command_count):
        if offset + 8 > len(data):
            raise RunnerError("truncated Mach-O load commands")
        command, size = struct.unpack_from("<II", data, offset)
        if size < 8 or offset + size > len(data):
            raise RunnerError("invalid Mach-O load command")
        if command == _LC_SEGMENT_64:
            if size < 72:
                raise RunnerError("invalid Mach-O segment command")
            segment_name = data[offset + 8:offset + 24].rstrip(b"\0")
            if segment_name == b"__LINKEDIT":
                linkedit = offset
            elif segment_name == b"__TEXT":
                text = offset

            section_count = struct.unpack_from("<I", data, offset + 64)[0]
            section_offset = offset + 72
            for _ in range(section_count):
                if section_offset + 80 > offset + size:
                    raise RunnerError("invalid Mach-O section table")
                file_offset = struct.unpack_from("<I", data, section_offset + 48)[0]
                if file_offset:
                    first_section = min(first_section, file_offset)
                section_offset += 80
        elif command == _LC_CODE_SIGNATURE:
            code_signature = offset
        offset += size

    if linkedit is None or text is None:
        raise RunnerError("test executable is missing __TEXT or __LINKEDIT")
    if offset != 32 + command_size:
        raise RunnerError("invalid Mach-O load command size")
    return file_type, command_count, command_size, code_signature, linkedit, text, first_section


def adhoc_sign_for_slot(source: Path, output: Path, slot_size: int, identifier: str) -> bytes:
    """Produces an Apple-compatible ad-hoc signature filling slot_size bytes."""
    data = bytearray(source.read_bytes())
    _load_commands(data)
    cpu_subtype = struct.unpack_from("<I", data, 8)[0] & 0x00FFFFFF
    if cpu_subtype == 2:
        raise RunnerError("arm64e test executables are not supported; build device arm64")
    _remove_unavailable_dylibs(data)
    (
        file_type,
        command_count,
        command_size,
        code_signature,
        linkedit,
        text,
        first_section,
    ) = _load_commands(data)

    if code_signature is not None:
        old_offset, old_size = struct.unpack_from("<II", data, code_signature + 8)
        if old_offset + old_size != len(data):
            raise RunnerError("invalid existing Mach-O code signature")
        base = bytearray(data[:old_offset])
    else:
        code_signature = 32 + command_size
        if code_signature + 16 > first_section or any(data[code_signature:code_signature + 16]):
            raise RunnerError("Mach-O has no room for an LC_CODE_SIGNATURE command")
        struct.pack_into("<II", data, 16, command_count + 1, command_size + 16)
        struct.pack_into("<IIII", data, code_signature, _LC_CODE_SIGNATURE, 16, 0, 0)
        base = data

    requirements = struct.pack(">III", 0xFADE0C01, 12, 0)
    cms_wrapper = struct.pack(">II", 0xFADE0B01, 8)
    identifier_bytes = identifier.encode("utf-8") + b"\0"
    page_size = 16384

    signature_size = 1024
    for _ in range(20):
        code_limit = slot_size - signature_size
        code_slots = (code_limit + page_size - 1) // page_size
        hash_offset = 88 + len(identifier_bytes) + 64
        code_directory_size = hash_offset + code_slots * 32
        superblob_size = 36 + code_directory_size + len(requirements) + len(cms_wrapper)
        next_signature_size = (superblob_size + 15) & ~15
        if next_signature_size == signature_size:
            break
        signature_size = next_signature_size
    else:
        raise RunnerError("could not size the ad-hoc code signature")

    code_limit = slot_size - signature_size
    if len(base) > code_limit:
        raise RunnerError(
            "test executable is too large for the VM slot "
            f"({len(base)} bytes before signing, {slot_size} byte slot)",
        )

    content = bytearray(base)
    content.extend(b"\0" * (code_limit - len(content)))

    linkedit_file_offset = struct.unpack_from("<Q", content, linkedit + 40)[0]
    linkedit_file_size = slot_size - linkedit_file_offset
    struct.pack_into("<Q", content, linkedit + 48, linkedit_file_size)
    struct.pack_into("<Q", content, linkedit + 32, (linkedit_file_size + 0x3FFF) & ~0x3FFF)
    struct.pack_into("<II", content, code_signature + 8, code_limit, signature_size)

    text_file_offset, text_file_size = struct.unpack_from("<QQ", content, text + 40)
    executable_segment_flags = 1 if file_type == _MH_EXECUTE else 0
    code_slots = (code_limit + page_size - 1) // page_size
    hash_offset = 88 + len(identifier_bytes) + 64
    code_directory_size = hash_offset + code_slots * 32

    code_directory = bytearray(struct.pack(
        ">9I4B4I4Q",
        0xFADE0C02,
        code_directory_size,
        0x20400,
        2,  # CS_ADHOC
        hash_offset,
        88,
        2,
        code_slots,
        min(code_limit, 0xFFFFFFFF),
        32,
        2,  # SHA-256
        0,
        14,  # iOS arm64 code-signing pages are 16 KiB
        0,
        0,
        0,
        0,
        code_limit,
        text_file_offset,
        text_file_size,
        executable_segment_flags,
    ))
    code_directory.extend(identifier_bytes)
    code_directory.extend(hashlib.sha256(requirements).digest())
    code_directory.extend(bytes(32))  # No bound Info.plist.
    if len(code_directory) != hash_offset:
        raise AssertionError("invalid CodeDirectory layout")
    for offset in range(0, code_limit, page_size):
        code_directory.extend(hashlib.sha256(
            content[offset:min(offset + page_size, code_limit)],
        ).digest())

    superblob_size = 36 + len(code_directory) + len(requirements) + len(cms_wrapper)
    superblob = bytearray(struct.pack(">III", 0xFADE0CC0, superblob_size, 3))
    blob_offset = 36
    for blob_type, blob in (
        (0, code_directory),
        (2, requirements),
        (0x10000, cms_wrapper),
    ):
        superblob.extend(struct.pack(">II", blob_type, blob_offset))
        blob_offset += len(blob)
    superblob.extend(code_directory)
    superblob.extend(requirements)
    superblob.extend(cms_wrapper)
    superblob.extend(b"\0" * (signature_size - len(superblob)))

    signed = content + superblob
    if len(signed) != slot_size:
        raise AssertionError("signed test executable does not fill its APFS slot")
    output.write_bytes(signed)
    return hashlib.sha256(code_directory).digest()[:20]


def merge_trust_cache(base_path: Path, output_path: Path, cdhash: bytes) -> None:
    data = base_path.read_bytes()
    if len(data) < 24:
        raise RunnerError("base trust cache is truncated")
    version, uuid, count = struct.unpack_from("<I16sI", data, 0)
    if version != 1 or len(data) != 24 + count * 22:
        raise RunnerError("base trust cache is not a version-1 trust cache")
    entries = {data[offset:offset + 22] for offset in range(24, len(data), 22)}
    entries.add(cdhash + bytes((2, 0)))
    ordered = sorted(entries)
    output_path.write_bytes(struct.pack("<I16sI", version, uuid, len(ordered)) + b"".join(ordered))


def _safe_extract_zip(archive: Path, destination: Path) -> None:
    root = destination.resolve()
    with zipfile.ZipFile(archive) as zip_file:
        for member in zip_file.infolist():
            target = (destination / member.filename).resolve()
            if target != root and root not in target.parents:
                raise RunnerError(f"unsafe path in test bundle ZIP: {member.filename}")
            unix_mode = member.external_attr >> 16
            if (unix_mode & 0o170000) == 0o120000:
                raise RunnerError("symlinks in zipped test bundles are not supported")
        zip_file.extractall(destination)


def locate_test_executable(bundle_artifact: Path, temporary_directory: Path) -> tuple[Path, Path]:
    if bundle_artifact.is_dir():
        bundle = bundle_artifact.resolve()
    else:
        extraction = temporary_directory / "bundle"
        extraction.mkdir()
        _safe_extract_zip(bundle_artifact, extraction)
        bundles = [path for path in extraction.rglob("*.xctest") if path.is_dir()]
        if len(bundles) != 1:
            raise RunnerError(f"expected one .xctest bundle, found {len(bundles)}")
        bundle = bundles[0]

    info_path = bundle / "Info.plist"
    if info_path.is_symlink() or not info_path.is_file():
        raise RunnerError("test bundle has no Info.plist")
    with info_path.open("rb") as info_file:
        executable_name = plistlib.load(info_file).get("CFBundleExecutable")
    if not executable_name:
        raise RunnerError("test bundle Info.plist has no CFBundleExecutable")
    if not isinstance(executable_name, str) or Path(executable_name).name != executable_name:
        raise RunnerError("test bundle CFBundleExecutable must be a file name")
    executable = bundle / executable_name
    if executable.is_symlink() or not executable.is_file():
        raise RunnerError(f"test bundle executable does not exist: {executable_name}")

    nested_machos = []
    unsupported_resources = []
    for path in bundle.rglob("*"):
        if path.is_symlink():
            raise RunnerError(
                "symlinks in test bundles are not supported: " +
                str(path.relative_to(bundle)),
            )
        if path == executable or not path.is_file():
            continue
        relative_path = path.relative_to(bundle)
        if relative_path == Path("Info.plist") or relative_path.parts[0] == "_CodeSignature":
            continue
        with path.open("rb") as candidate:
            magic = candidate.read(4)
        if magic in (b"\xcf\xfa\xed\xfe", b"\xca\xfe\xba\xbe", b"\xbe\xba\xfe\xca"):
            nested_machos.append(relative_path)
        else:
            unsupported_resources.append(relative_path)
    if nested_machos:
        raise RunnerError(
            "nested Mach-O files in test bundles are not supported yet: " +
            ", ".join(map(str, nested_machos)),
        )
    if unsupported_resources:
        raise RunnerError(
            "test bundle resources are not supported yet: " +
            ", ".join(map(str, unsupported_resources)),
        )
    return bundle, executable


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as input_file:
        for chunk in iter(lambda: input_file.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_slot_manifest(path: Path, inputs: dict[str, Path]) -> dict[str, object]:
    try:
        manifest = json.loads(path.read_text())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RunnerError(f"could not read slot manifest: {error}") from error
    if not isinstance(manifest, dict) or manifest.get("version") != 2:
        raise RunnerError("slot manifest must have version 2")
    if manifest.get("architecture") != "arm64":
        raise RunnerError("slot manifest must target arm64")

    for key in ("offset", "size"):
        if not isinstance(manifest.get(key), int):
            raise RunnerError(f"slot manifest {key} must be an integer")
    offset = manifest["offset"]
    size = manifest["size"]
    if offset < 0 or offset % 4096:
        raise RunnerError("slot manifest offset must be nonnegative and 4096-byte aligned")
    if size <= 0 or size % 4096:
        raise RunnerError("slot manifest size must be a positive multiple of 4096")

    placeholder_digest = manifest.get("placeholder_sha256")
    if not isinstance(placeholder_digest, str) or not _SHA256_RE.fullmatch(placeholder_digest):
        raise RunnerError("slot manifest has an invalid placeholder SHA-256")
    input_digests = manifest.get("input_sha256")
    if not isinstance(input_digests, dict):
        raise RunnerError("slot manifest has no input SHA-256 map")
    if set(input_digests) != set(inputs):
        raise RunnerError(
            "slot manifest input set does not match the runner inputs "
            f"(manifest: {sorted(input_digests)}, runner: {sorted(inputs)})",
        )
    for name, input_path in inputs.items():
        expected = input_digests.get(name)
        if not isinstance(expected, str) or not _SHA256_RE.fullmatch(expected):
            raise RunnerError(f"slot manifest has an invalid {name} SHA-256")
        if _sha256_file(input_path) != expected:
            raise RunnerError(f"{name} does not match the slot manifest")
    return manifest


def parse_environment(strings: list[str], combined: str) -> dict[str, str]:
    result: dict[str, str] = {}
    values = list(strings)
    if combined:
        values.extend(part for part in combined.split(",") if part)
    for value in values:
        name, separator, content = value.partition("=")
        if not separator or not _ENV_NAME_RE.fullmatch(name):
            raise RunnerError(f"invalid XCTest environment entry: {value!r}")
        result[name] = content
    return result


def test_filter(static_filter: str) -> str:
    filters = []
    for value in (static_filter, os.environ.get("TESTBRIDGE_TEST_ONLY", "")):
        filters.extend(part for part in value.split(",") if part)
    unsupported = [value for value in filters if value.startswith("-")]
    if unsupported:
        raise RunnerError("negative XCTest filters are not supported yet")
    return ",".join(filters) if filters else "All"


def test_arguments(values: list[str]) -> str:
    for value in values:
        if any(character in value for character in ("\0", "\r", "\n")):
            raise RunnerError("XCTest process arguments may not contain control characters")
    return " ".join(shlex.quote(value) for value in values)


class QemuConsole:
    def __init__(self, command: list[str]):
        master, slave = pty.openpty()
        self._master = master
        self.process = subprocess.Popen(
            command,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            close_fds=True,
        )
        os.close(slave)
        os.set_blocking(master, False)
        self.window = bytearray()
        self.output_tail = bytearray()

    def read_until(self, pattern: re.Pattern[bytes], timeout: int) -> re.Match[bytes] | None:
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                return None
            ready, _, _ = select.select([self._master], [], [], min(0.25, deadline - time.monotonic()))
            if not ready:
                continue
            try:
                chunk = os.read(self._master, 65536)
            except BlockingIOError:
                continue
            except OSError:
                return None
            if not chunk:
                return None
            sys.stdout.buffer.write(chunk)
            sys.stdout.buffer.flush()
            self.window.extend(chunk)
            self.output_tail.extend(chunk)
            del self.window[:-131072]
            del self.output_tail[:-1048576]
            match = pattern.search(self.window)
            if match:
                return match
        return None

    def has_panic(self) -> bool:
        return _PANIC_RE.search(self.window) is not None

    def write(self, content: str | bytes) -> None:
        if isinstance(content, str):
            content = content.encode()
        os.write(self._master, content)

    def write_slowly(self, content: str, delay: float = 0.002) -> None:
        for byte in content.encode():
            os.write(self._master, bytes((byte,)))
            time.sleep(delay)

    def stop(self) -> None:
        if self.process.poll() is None:
            try:
                self.write(b"\x01x")
                self.process.wait(timeout=2)
            except (OSError, subprocess.TimeoutExpired):
                try:
                    self.process.terminate()
                except ProcessLookupError:
                    pass
                try:
                    self.process.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    try:
                        self.process.kill()
                    except ProcessLookupError:
                        pass
                    self.process.wait()
        try:
            os.close(self._master)
        except OSError:
            pass


def _write_xml(path: str, exit_code: int, output: bytes) -> None:
    if not path:
        return
    suite = ET.Element("testsuite", name="ios_vm_xctest", tests="1", failures="1" if exit_code else "0")
    case = ET.SubElement(suite, "testcase", classname="darwin_vm", name="XCTest")
    if exit_code:
        failure = ET.SubElement(case, "failure", message=f"XCTest exited with {exit_code}")
        failure.text = output.decode("utf-8", "replace")
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    ET.ElementTree(suite).write(path, encoding="utf-8", xml_declaration=True)


def run(args: argparse.Namespace) -> int:
    for path in (
        args.qemu,
        args.bootkc,
        args.device_tree,
        args.trust_cache,
        args.ramdisk,
        args.slot_manifest,
        args.test_bundle,
    ):
        if not Path(path).exists():
            raise RunnerError(f"required input does not exist: {path}")
    if not Path(args.qemu).is_file() or not os.access(args.qemu, os.X_OK):
        raise RunnerError(f"QEMU is not executable: {args.qemu}")
    if bool(args.sptm) != bool(args.txm):
        raise RunnerError("sptm and txm must either both be provided or both be omitted")
    if args.boot_attempts <= 0 or args.boot_timeout <= 0 or args.test_timeout <= 0:
        raise RunnerError("attempt and timeout values must be positive")
    if args.sptm:
        print(
            "warning: SPTM/TXM XCTest execution is experimental; "
            "use non-SPTM firmware for Linux-built tests",
            file=sys.stderr,
        )

    environment = parse_environment(args.test_env, args.test_env_string)
    selected_tests = test_filter(args.test_filter)
    process_arguments = test_arguments(args.test_arg)

    with tempfile.TemporaryDirectory(prefix="ios-vm-test-") as temporary:
        temporary_directory = Path(temporary)
        _, test_executable = locate_test_executable(Path(args.test_bundle), temporary_directory)
        manifest_inputs = {
            "bootkc": Path(args.bootkc),
            "device_tree": Path(args.device_tree),
            "ramdisk": Path(args.ramdisk),
            "trust_cache": Path(args.trust_cache),
        }
        if args.sptm:
            manifest_inputs.update({
                "sptm": Path(args.sptm),
                "txm": Path(args.txm),
            })
        manifest = load_slot_manifest(Path(args.slot_manifest), manifest_inputs)
        slot_offset = manifest["offset"]
        slot_size = manifest["size"]
        guest_bundle = manifest.get("bundle_path")
        if (
            not isinstance(guest_bundle, str) or
            not guest_bundle.startswith("/") or
            any(part == ".." for part in guest_bundle.split("/")) or
            any(character in guest_bundle for character in ("\0", "\r", "\n"))
        ):
            raise RunnerError("slot manifest has an invalid guest bundle path")
        expected_digest = manifest["placeholder_sha256"]

        ramdisk = temporary_directory / "ramdisk.dmg"
        shutil.copyfile(args.ramdisk, ramdisk)
        with ramdisk.open("r+b") as image:
            image.seek(slot_offset)
            original_slot = image.read(slot_size)
            if len(original_slot) != slot_size:
                raise RunnerError("test slot extends beyond the ramdisk")
            if hashlib.sha256(original_slot).hexdigest() != expected_digest:
                raise RunnerError("ramdisk test slot does not match its manifest")

            signed_test = temporary_directory / "Test"
            cdhash = adhoc_sign_for_slot(test_executable, signed_test, slot_size, "Test")
            image.seek(slot_offset)
            image.write(signed_test.read_bytes())

        trust_cache = temporary_directory / "ramdisk.tc"
        merge_trust_cache(Path(args.trust_cache), trust_cache, cdhash)

        qemu_command = [
            args.qemu,
            "-M", "darwin",
            "-bootkc", args.bootkc,
            "-dtree", args.device_tree,
            "-tc", str(trust_cache),
            "-ramdisk", str(ramdisk),
            "-args", args.boot_args,
            "-nographic",
            "-serial", "mon:stdio",
            "-m", "8G",
            "-seed", "1",
        ]
        if args.sptm:
            qemu_command.extend(["-sptm", args.sptm, "-txm", args.txm])

        guest_environment = {"NSUnbufferedIO": "YES", **environment}
        env_prefix = " ".join(
            f"{name}={shlex.quote(value)}"
            for name, value in sorted(guest_environment.items())
        )
        xctest = "/System/Developer/Library/Xcode/Agents/xctest"
        command = (
            f"{env_prefix} {xctest} -XCTest {shlex.quote(selected_tests)} "
            f"{shlex.quote(guest_bundle)}"
            f"{' ' + process_arguments if process_arguments else ''}; "
            "test_status=$?; printf '\\n__DARWIN_VM_XCTEST_EXIT__%s\\n' \"$test_status\"\r"
        )
        last_output = b""
        for attempt in range(1, args.boot_attempts + 1):
            print(f"darwin-vm: attempt {attempt}/{args.boot_attempts}", file=sys.stderr)
            console = QemuConsole(qemu_command)
            try:
                boot_event = console.read_until(_BOOT_EVENT_RE, args.boot_timeout)
                if not (
                    boot_event and
                    _PROMPT_RE.search(boot_event.group(0)) and
                    not console.has_panic()
                ):
                    last_output = bytes(console.output_tail)
                    if console.has_panic() and attempt < args.boot_attempts:
                        print("darwin-vm: retrying after known SPTM/TXM panic", file=sys.stderr)
                        continue
                    reason = (
                        "known SPTM/TXM panic" if console.has_panic() else
                        "boot timeout or early QEMU exit"
                    )
                    _write_xml(args.xml_output, 1, last_output)
                    raise RunnerError(f"darwin-vm failed during boot: {reason}")

                console.write_slowly(command)
                console.read_until(_TEST_EVENT_RE, args.test_timeout)
                last_output = bytes(console.output_tail)
                exit_match = _EXIT_RE.search(console.window)
                if exit_match and not console.has_panic():
                    exit_code = int(exit_match.group(1))
                    _write_xml(args.xml_output, exit_code, last_output)
                    return exit_code

                if console.has_panic() and attempt < args.boot_attempts:
                    print("darwin-vm: retrying after known SPTM/TXM panic", file=sys.stderr)
                    continue
                reason = (
                    "known SPTM/TXM panic" if console.has_panic() else
                    "XCTest timeout or early QEMU exit"
                )
                _write_xml(args.xml_output, 1, last_output)
                raise RunnerError(f"darwin-vm failed while running XCTest: {reason}")
            finally:
                console.stop()

        _write_xml(args.xml_output, 1, last_output)
        raise RunnerError(
            f"darwin-vm hit a known SPTM/TXM panic on all {args.boot_attempts} attempts",
        )


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--qemu", required=True)
    parser.add_argument("--bootkc", required=True)
    parser.add_argument("--device-tree", required=True)
    parser.add_argument("--trust-cache", required=True)
    parser.add_argument("--ramdisk", required=True)
    parser.add_argument("--slot-manifest", required=True)
    parser.add_argument("--sptm")
    parser.add_argument("--txm")
    parser.add_argument("--boot-args", required=True)
    parser.add_argument("--boot-attempts", type=int, default=1)
    parser.add_argument("--boot-timeout", type=int, default=30)
    parser.add_argument("--test-timeout", type=int, default=300)
    parser.add_argument("--test-bundle", required=True)
    parser.add_argument("--test-filter", default="")
    parser.add_argument("--test-env-string", default="")
    parser.add_argument("--test-env", action="append", default=[])
    parser.add_argument("--test-arg", action="append", default=[])
    parser.add_argument("--xml-output", default="")
    return parser


def _terminate(signum: int, _frame: object) -> None:
    raise SystemExit(128 + signum)


def main() -> int:
    args = _parser().parse_args()
    previous_sigterm = signal.signal(signal.SIGTERM, _terminate)
    try:
        return run(args)
    except RunnerError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    finally:
        signal.signal(signal.SIGTERM, previous_sigterm)


if __name__ == "__main__":
    raise SystemExit(main())
