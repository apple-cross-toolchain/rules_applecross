"""The app's default.metallib, as rules_apple builds it off a Mac with relax_metal:
the container Xcode writes (docs: .research/metal-air/FORMAT.md on metal-air-research)."""

import hashlib
import os
import struct
import sys
import unittest
import zipfile

APP = None


def tags(data, offset):
    out = {}
    while True:
        name = data[offset : offset + 4].decode()
        offset += 4
        if name == "ENDT":
            return out, offset
        (size,) = struct.unpack_from("<H", data, offset)
        out[name] = data[offset + 2 : offset + 2 + size]
        offset += 2 + size


def parse(data):
    assert data[:4] == b"MTLB"
    header = struct.unpack_from("<6H", data, 4)
    total, flist, _, public, _, _, _, modules, _ = struct.unpack_from("<9Q", data, 16)
    (count,) = struct.unpack_from("<I", data, flist)
    offset, functions = flist + 4, []
    for _ in range(count):
        (size,) = struct.unpack_from("<I", data, offset)
        entry, _ = tags(data, offset + 4)
        pub_offset, _, module_offset = struct.unpack("<3Q", entry["OFFT"])
        pub, _ = tags(data, public + pub_offset + 4)
        (module_size,) = struct.unpack("<Q", entry["MDSZ"])
        module = data[modules + module_offset : modules + module_offset + module_size]
        functions.append(dict(name=entry["NAME"][:-1].decode(), type=entry["TYPE"][0], vers=struct.unpack("<4H", entry["VERS"]),
                              hash=entry["HASH"], module=module, public=pub))
        offset += size
    return header, total, functions


class MetalTest(unittest.TestCase):
    def setUp(self):
        with zipfile.ZipFile(APP) as archive:
            name = next(n for n in archive.namelist() if n.endswith(".app/default.metallib"))
            self.data = archive.read(name)
        self.header, self.total, self.functions = parse(self.data)

    def test_header(self):
        # iOS family, file version 7, iOS Simulator (0x87), deployment target 17.0
        self.assertEqual(self.header, (1, 2, 7, 0x8700, 17, 0))
        self.assertEqual(self.total, len(self.data))

    def test_functions(self):
        # kernels, then vertex, then fragment functions, as Xcode orders them
        self.assertEqual([(f["name"], f["type"]) for f in self.functions], [("scale", 2), ("quad", 0), ("textured", 1)])
        for f in self.functions:
            self.assertEqual(f["vers"], (2, 6, 3, 1))  # AIR 2.6, Metal 3.1 for iOS 17
            self.assertEqual(f["module"][:4], bytes.fromhex("dec0170b"))
            self.assertEqual(f["hash"], hashlib.sha256(f["module"]).digest())

    def test_public_metadata(self):
        quad = next(f for f in self.functions if f["name"] == "quad")
        # both attributes, used, as float2 (MTLDataType 4)
        self.assertEqual(quad["public"]["VATT"], b"\x02\x00position\x00\x00\x80uv\x00\x01\x80")
        self.assertEqual(quad["public"]["VATY"], b"\x02\x00\x04\x04")
        scale = next(f for f in self.functions if f["name"] == "scale")
        # use_square: bool (53), index 0, required
        self.assertEqual(scale["public"]["CNST"], b"\x01\x00use_square\x00\x35\x00\x00\x01")


if __name__ == "__main__":
    APP = os.path.join(os.environ.get("TEST_SRCDIR", ""), os.environ.get("TEST_WORKSPACE", ""), sys.argv[1])
    unittest.main(argv=sys.argv[:1], verbosity=2)
