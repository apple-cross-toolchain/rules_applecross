"""extract_xip.py and link_local_xcode.py --move on a small synthetic .xip,
laid out as Apple's are: a xar archive whose Content is a pbzx stream of a cpio
(odc) archive, with identical files stored once as hard links."""

import lzma
import os
import shutil
import stat
import struct
import sys
import tempfile
import unittest
import zlib

from python.runfiles import runfiles

sys.path.insert(0, os.path.dirname(
    runfiles.Create().Rlocation("rules_applecross/toolchain/repo_tools/extract_xip.py")))

import extract_xip  # noqa: E402
import link_local_xcode  # noqa: E402

DEV = "./Xcode.app/Contents/Developer/"
SDK = DEV + "Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/"
DRIVERKIT = DEV + "Platforms/DriverKit.platform/Developer/SDKs/DriverKit.sdk/"
TOOLCHAIN = DEV + "Toolchains/XcodeDefault.xctoolchain/"


def placeholder(size):
    return (b"NULLcanary" + b"\0" * size)[:size]


class Cpio(object):
    def __init__(self):
        self.data = bytearray()
        self.next_ino = 1

    def entry(self, path, mode, data=b"", ino=None, nlink=1):
        if ino is None:
            ino = self.next_ino
            self.next_ino += 1
        name = path.encode() + b"\0"
        self.data += b"070707" + "".join([
            "%06o" % 0, "%06o" % ino, "%06o" % mode, "%06o" % 0, "%06o" % 0,
            "%06o" % nlink, "%06o" % 0, "%011o" % 0, "%06o" % len(name),
            "%011o" % len(data),
        ]).encode() + name + data
        return ino

    def directory(self, path):
        self.entry(path, stat.S_IFDIR | 0o755)

    def file(self, path, data, mode=0o644, ino=None, nlink=1):
        return self.entry(path, stat.S_IFREG | mode, data, ino, nlink)

    def symlink(self, path, target):
        self.entry(path, stat.S_IFLNK | 0o755, target.encode())

    def finish(self):
        self.entry("TRAILER!!!", 0)
        return bytes(self.data)


def pbzx(payload):
    """Two chunks: the first stored, the second XZ-compressed."""
    half = len(payload) // 2
    out = b"pbzx" + struct.pack(">Q", 1 << 24)
    first, second = payload[:half], payload[half:]
    out += struct.pack(">QQ", len(first), len(first)) + first
    compressed = lzma.compress(second, format=lzma.FORMAT_XZ)
    out += struct.pack(">QQ", len(second), len(compressed)) + compressed
    return out


def xar(content):
    metadata = b"metadata"
    toc = """<?xml version="1.0" encoding="UTF-8"?>
<xar><toc>
<file id="1"><data><length>{0}</length><offset>0</offset><size>{0}</size>
<encoding style="application/octet-stream"/></data><name>Metadata</name></file>
<file id="2"><data><length>{1}</length><offset>{0}</offset><size>{1}</size>
<encoding style="application/octet-stream"/></data><name>Content</name></file>
</toc></xar>""".format(len(metadata), len(content)).encode()
    compressed = zlib.compress(toc)
    header = b"xar!" + struct.pack(">HHQQI", 28, 1, len(compressed), len(toc), 1)
    return header + compressed + metadata + content


def skeleton(cpio):
    for path in (".", "./Xcode.app", "./Xcode.app/Contents", DEV[:-1],
                 DEV + "Platforms", DEV + "Platforms/iPhoneOS.platform",
                 DEV + "Platforms/iPhoneOS.platform/Developer",
                 DEV + "Platforms/iPhoneOS.platform/Developer/SDKs",
                 SDK[:-1], SDK + "usr", SDK + "usr/include",
                 DEV + "Platforms/DriverKit.platform",
                 DEV + "Platforms/DriverKit.platform/Developer",
                 DEV + "Platforms/DriverKit.platform/Developer/SDKs",
                 DRIVERKIT[:-1], DEV + "Toolchains", TOOLCHAIN[:-1]):
        cpio.directory(path)
    cpio.file("./Xcode.app/Contents/Info.plist", b"info")
    cpio.file("./Xcode.app/Contents/version.plist", b"version")
    cpio.file(DEV + "Platforms/iPhoneOS.platform/Info.plist", b"platform")
    cpio.file(TOOLCHAIN + "ToolchainInfo.plist", b"toolchain")


class ExtractXipTest(unittest.TestCase):
    def setUp(self):
        self.work = tempfile.mkdtemp()

    def tearDown(self):
        shutil.rmtree(self.work)

    def extract(self, cpio):
        path = os.path.join(self.work, "Xcode.xip")
        with open(path, "wb") as f:
            f.write(xar(pbzx(cpio.finish())))
        destination = os.path.join(self.work, "stage")
        extract_xip.extract(path, destination)
        return destination

    def read(self, root, rel):
        with open(os.path.join(root, rel), "rb") as f:
            return f.read()

    def test_selects_sdks_and_resolves_hard_links(self):
        cpio = Cpio()
        skeleton(cpio)
        # An unselected first copy, then a selected link to it.
        shared = b"#define SHARED 1\n" * 50
        ino = cpio.file(DRIVERKIT + "shared.h", shared, nlink=2)
        cpio.file(SDK + "usr/include/shared.h", placeholder(len(shared)), ino=ino, nlink=2)
        # A selected first copy, then a selected link to it.
        own = b"int own;\n"
        ino = cpio.file(SDK + "usr/include/own.h", own, nlink=2)
        cpio.file(SDK + "usr/include/own_link.h", placeholder(len(own)), ino=ino, nlink=2)
        cpio.file(SDK + "usr/include/tool", b"#!/bin/sh\n", mode=0o755)
        cpio.symlink(SDK + "usr/include/alias.h", "./own.h")
        cpio.symlink(SDK + "usr/include/up.h", "..//include/own.h")
        cpio.symlink(SDK + "usr/include/self", "./")
        # Never selected, never written.
        cpio.directory("./Xcode.app/Contents/MacOS")
        cpio.file("./Xcode.app/Contents/MacOS/Xcode", b"\0" * 4096)

        stage = self.extract(cpio)
        sdk = os.path.join(stage, SDK[2:])
        self.assertEqual(self.read(sdk, "usr/include/shared.h"), shared)
        self.assertEqual(self.read(sdk, "usr/include/own_link.h"), own)
        self.assertEqual(os.readlink(os.path.join(sdk, "usr/include/alias.h")), "own.h")
        self.assertEqual(os.readlink(os.path.join(sdk, "usr/include/up.h")), "../include/own.h")
        self.assertEqual(os.readlink(os.path.join(sdk, "usr/include/self")), ".")
        self.assertTrue(os.stat(os.path.join(sdk, "usr/include/tool")).st_mode & 0o111)
        self.assertFalse(os.stat(os.path.join(sdk, "usr/include/own.h")).st_mode & 0o111)
        self.assertFalse(os.path.exists(os.path.join(stage, "Xcode.app/Contents/MacOS")))
        self.assertFalse(os.path.exists(os.path.join(stage, DRIVERKIT[2:])))
        self.assertEqual(sorted(os.listdir(stage)), ["Xcode.app"])

        # --move gives a self-contained tree with the same content.
        xcode_app = os.path.join(self.work, "out", "Xcode.app")
        link_local_xcode.Linker(
            os.path.join(stage, DEV[2:-1]), xcode_app, readtapi="/nonexistent", move=True,
        ).run()
        out_sdk = os.path.join(xcode_app, "Contents", SDK[len("./Xcode.app/Contents/"):])
        self.assertEqual(self.read(out_sdk, "usr/include/shared.h"), shared)
        self.assertEqual(self.read(xcode_app, "Contents/version.plist"), b"version")
        self.assertEqual(self.read(
            xcode_app, "Contents/Developer/Toolchains/XcodeDefault.xctoolchain/ToolchainInfo.plist"),
            b"toolchain")
        for dirpath, dirnames, filenames in os.walk(xcode_app):
            for name in dirnames + filenames:
                path = os.path.join(dirpath, name)
                if os.path.islink(path):
                    self.assertFalse(os.path.isabs(os.readlink(path)), path)

    def test_refuses_data_in_a_later_hard_link(self):
        cpio = Cpio()
        skeleton(cpio)
        ino = cpio.file(SDK + "usr/include/a.h", b"a", nlink=2)
        cpio.file(SDK + "usr/include/b.h", b"b", ino=ino, nlink=2)
        with self.assertRaisesRegex(extract_xip.XipError, "later hard link carries data"):
            self.extract(cpio)

    def test_refuses_a_placeholder_without_its_first_copy(self):
        cpio = Cpio()
        skeleton(cpio)
        cpio.file(SDK + "usr/include/a.h", placeholder(64), nlink=2)
        with self.assertRaisesRegex(extract_xip.XipError, "placeholder"):
            self.extract(cpio)

    def test_refuses_paths_that_leave_the_destination(self):
        cpio = Cpio()
        skeleton(cpio)
        cpio.file(DEV + "Platforms/../../../../escape", b"x")
        with self.assertRaisesRegex(extract_xip.XipError, "unsafe path"):
            self.extract(cpio)

    def test_refuses_writing_through_a_symlink(self):
        cpio = Cpio()
        skeleton(cpio)
        cpio.symlink(SDK + "usr/lib", "/tmp")
        cpio.file(SDK + "usr/lib/escape.h", b"x")
        with self.assertRaisesRegex(extract_xip.XipError, "through a symlink"):
            self.extract(cpio)

    def test_refuses_an_archive_without_xcode(self):
        cpio = Cpio()
        cpio.directory(".")
        with self.assertRaisesRegex(extract_xip.XipError, "no Xcode.app"):
            self.extract(cpio)


if __name__ == "__main__":
    unittest.main()
