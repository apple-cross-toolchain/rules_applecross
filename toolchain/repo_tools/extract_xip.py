#!/usr/bin/env python3
"""Extract the parts of an Xcode .xip that the cross toolchain reads.

Usage: extract_xip.py <Xcode.xip> <destination>

Writes <destination>/Xcode.app/Contents/{Info.plist,version.plist} and, under
Contents/Developer, the subtrees link_local_xcode.py selects from, so that
link_local_xcode.py --move can turn the result into the same tree a local Xcode
or a packaged archive gives. Everything else in the archive is decompressed and
discarded without touching the disk.

A .xip is a xar archive whose Content member is a pbzx stream: chunks that are
each either stored or a complete XZ stream, which together hold a cpio archive
in the portable (odc) format. The archive is read as a stream, so memory stays
at a few chunks however large the Xcode.

Identical files are stored once: entries sharing a (dev, ino) pair are hard
links, the first holding the data and each later one a placeholder of the same
size, "NULLcanary" and then zeros. A later link becomes a hard link to the
first; its placeholder is checked and discarded. A first copy outside the
selection is kept aside when its link count says more links follow.

The signature in the xar table of contents is not checked; the repository rule
pins the archive's SHA-256 instead.
"""

import lzma
import os
import shutil
import stat
import struct
import sys
import xml.etree.ElementTree as ET
import zlib

from link_local_xcode import PLATFORMS, _TOOLCHAIN

_APP = "Xcode.app/Contents/"
_DEVELOPER = _APP + "Developer/"

# The include roots of link_local_xcode._includes, as path prefixes under the
# Developer directory. The back-deployment runtimes (usr/lib/swift-5.5, ...)
# are matched by the swift- prefix.
_ROOTS = tuple(
    ["Platforms/{}.platform/{}".format(p, sub) for p in PLATFORMS for sub in (
        "Info.plist",
        "Developer/SDKs/",
        "usr/lib/",
        "Developer/usr/lib/",
        "Developer/Library/Frameworks/",
        "Developer/Library/PrivateFrameworks/",
    )] + [_TOOLCHAIN + "/" + sub for sub in (
        "ToolchainInfo.plist",
        "usr/include/",
        "usr/lib/arc/",
        "usr/lib/clang/",
        "usr/lib/swift/",
        "usr/lib/swift-",
    )]
)

_CPIO_HEADER = 76
_CANARY = b"NULLcanary"


class XipError(Exception):
    pass


def _selected(path):
    """Whether a path relative to the archive root is extracted."""
    if path in (_APP + "Info.plist", _APP + "version.plist"):
        return True
    if not path.startswith(_DEVELOPER):
        return False
    rel = path[len(_DEVELOPER):]
    # A directory entry precedes its contents, so a root's own parents are
    # selected too, which keeps their modes.
    return any(rel.startswith(root) or root.startswith(rel + "/") for root in _ROOTS)


def _content_offset(f):
    """The absolute offset of the xar Content member's data."""
    header = f.read(28)
    if len(header) != 28 or header[:4] != b"xar!":
        raise XipError("not a xar archive")
    header_size, _, toc_compressed, _, _ = struct.unpack(">HHQQI", header[4:])
    f.seek(header_size)
    toc = ET.fromstring(zlib.decompress(f.read(toc_compressed)))
    heap = header_size + toc_compressed
    for entry in toc.iter("file"):
        if entry.findtext("name") != "Content":
            continue
        data = entry.find("data")
        encoding = data.find("encoding").get("style") if data is not None else None
        if encoding != "application/octet-stream":
            raise XipError("unexpected Content encoding {}".format(encoding))
        return heap + int(data.findtext("offset"))
    raise XipError("the xar archive has no Content member")


def _pbzx_chunks(f):
    if f.read(4) != b"pbzx":
        raise XipError("Content is not a pbzx stream")
    f.read(8)
    while True:
        header = f.read(16)
        if not header:
            return
        if len(header) != 16:
            raise XipError("truncated pbzx chunk header")
        uncompressed, compressed = struct.unpack(">QQ", header)
        data = f.read(compressed)
        if len(data) != compressed:
            raise XipError("truncated pbzx chunk")
        if compressed != uncompressed:
            data = lzma.decompress(data, format=lzma.FORMAT_XZ)
            if len(data) != uncompressed:
                raise XipError("pbzx chunk decompressed to the wrong size")
        yield data


class _Stream(object):
    def __init__(self, chunks):
        self.chunks = chunks
        self.buffer = bytearray()

    def read(self, size):
        while len(self.buffer) < size:
            chunk = next(self.chunks, None)
            if chunk is None:
                raise XipError("the cpio archive ends early")
            self.buffer += chunk
        data = bytes(self.buffer[:size])
        del self.buffer[:size]
        return data

    def copy(self, size, out):
        """Pass size bytes to out, or discard them when out is None."""
        while size:
            if not self.buffer:
                chunk = next(self.chunks, None)
                if chunk is None:
                    raise XipError("the cpio archive ends early")
                self.buffer += chunk
            n = min(size, len(self.buffer))
            if out is not None:
                out.write(memoryview(self.buffer)[:n])
            del self.buffer[:n]
            size -= n

    def placeholder(self, size):
        """Consume size bytes; whether they are a hard link's placeholder."""
        head = self.read(min(size, len(_CANARY)))
        checker = _ZeroCheck()
        self.copy(size - len(head), checker)
        return head == _CANARY[:size] and checker.zero


class _ZeroCheck(object):
    def __init__(self):
        self.zero = True

    def write(self, data):
        if self.zero and data.tobytes().strip(b"\0"):
            self.zero = False


def _entries(stream):
    """Yield (path, mode, dev, ino, nlink, size) for each cpio entry; the caller
    reads or skips its data before asking for the next."""
    while True:
        header = stream.read(_CPIO_HEADER)
        if header[:6] != b"070707":
            raise XipError("bad cpio header")
        try:
            dev = int(header[6:12], 8)
            ino = int(header[12:18], 8)
            mode = int(header[18:24], 8)
            nlink = int(header[36:42], 8)
            name_size = int(header[59:65], 8)
            size = int(header[65:76], 8)
        except ValueError:
            raise XipError("bad cpio header")
        name = stream.read(name_size)
        if not name.endswith(b"\0"):
            raise XipError("bad cpio entry name")
        path = name[:-1].decode("utf-8")
        if path == "TRAILER!!!":
            return
        yield path, mode, dev, ino, nlink, size


def _safe_relative(path):
    if path.startswith("./"):
        path = path[2:]
    parts = path.split("/")
    if path.startswith("/") or any(p in ("", ".", "..") for p in parts):
        raise XipError("unsafe path in archive: " + path)
    return path


def _link_target(target):
    """A link's target without "." or empty segments, as Bazel extracts an
    archive's links, so a tree from a .xip and one from the SDK archive made of
    it agree. Both spellings resolve alike; ".." is kept."""
    parts = [p for p in target.split("/") if p not in ("", ".")]
    return ("/" if target.startswith("/") else "") + ("/".join(parts) or ".")


def _write_first(stream, size, out, path):
    """Write the data of a file's first entry, which is never a placeholder."""
    head = stream.read(min(size, len(_CANARY)))
    if size and head == _CANARY[:size]:
        raise XipError("the first entry of a file is a hard link's placeholder: " + path)
    with open(out, "wb") as w:
        w.write(head)
        stream.copy(size - len(head), w)


def extract(xip, destination):
    destination = os.path.realpath(destination)
    os.makedirs(destination, exist_ok=True)
    # Unselected first copies of hard-linked files, for links that are selected.
    aside = os.path.join(destination, ".hardlinks")
    os.makedirs(aside, exist_ok=True)
    # (dev, ino) to the path holding the data.
    links = {}
    count = 0
    with open(xip, "rb") as f:
        f.seek(_content_offset(f))
        stream = _Stream(_pbzx_chunks(f))
        for path, mode, dev, ino, nlink, size in _entries(stream):
            if path == ".":
                stream.copy(size, None)
                continue
            rel = _safe_relative(path)
            kind = stat.S_IFMT(mode)
            key = (dev, ino)
            if kind == stat.S_IFREG and key in links:
                if not stream.placeholder(size):
                    raise XipError("a later hard link carries data: " + path)
            if not _selected(rel):
                if kind == stat.S_IFREG and key not in links and nlink > 1:
                    links[key] = os.path.join(aside, "{}-{}".format(dev, ino))
                    _write_first(stream, size, links[key], path)
                elif kind != stat.S_IFREG or key not in links:
                    stream.copy(size, None)
                continue
            out = os.path.join(destination, rel)
            parent = os.path.dirname(out)
            # Never write through a link the archive made.
            if os.path.realpath(parent) != parent:
                raise XipError("archive path passes through a symlink: " + path)
            if kind == stat.S_IFDIR:
                stream.copy(size, None)
                os.makedirs(out, exist_ok=True)
                os.chmod(out, 0o755)
            elif kind == stat.S_IFLNK:
                os.symlink(_link_target(stream.read(size).decode("utf-8")), out)
            elif kind == stat.S_IFREG:
                if key in links:
                    os.link(links[key], out)
                else:
                    _write_first(stream, size, out, path)
                    links[key] = out
                os.chmod(out, 0o755 if mode & 0o111 else 0o644)
            else:
                raise XipError("unsupported cpio entry type {:o}: {}".format(kind, path))
            count += 1
    shutil.rmtree(aside)
    if not os.path.isfile(os.path.join(destination, _APP, "version.plist")):
        raise XipError("the archive holds no Xcode.app")
    return count


def main():
    if len(sys.argv) != 3:
        print("usage: extract_xip.py <Xcode.xip> <destination>", file=sys.stderr)
        return 2
    try:
        count = extract(sys.argv[1], sys.argv[2])
    except (XipError, OSError, lzma.LZMAError, zlib.error, ET.ParseError) as e:
        print("error: {}: {}".format(sys.argv[1], e), file=sys.stderr)
        return 1
    print("extracted {} entries from {}".format(count, sys.argv[1]), file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
