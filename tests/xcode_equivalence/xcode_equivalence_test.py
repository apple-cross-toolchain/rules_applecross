"""Xcode's own run-to-run variations are equivalent; any other change isn't."""

import struct
import unittest

from python.runfiles import runfiles
from xcode_equivalence import _appearance_blocks, _bom, equivalent

RUNFILES = runfiles.Create()


def read(name):
    with open(RUNFILES.Rlocation("rules_applecross/tests/xcode_equivalence/data/" + name), "rb") as f:
        return f.read()


class AssetsCarTest(unittest.TestCase):

    def setUp(self):
        self.first, self.second = read("Assets.1.car"), read("Assets.2.car")

    def test_two_xcode_runs_are_equivalent(self):
        self.assertNotEqual(self.first, self.second)
        self.assertTrue(equivalent("App.app/Assets.car", self.first, self.second))

    def test_any_changed_byte_is_not(self):
        blocks, variables, _ = _bom(self.second)
        leaf, owned, _ = _appearance_blocks(self.second, blocks, variables)
        # Every byte of the appearance blocks and the leaf, and a stride
        # through the rest of the file.
        positions = set(range(0, len(self.second), 97))
        for i in owned | {leaf}:
            at, length = blocks[i]
            positions |= set(range(at, at + length))
        for at in sorted(positions):
            changed = bytearray(self.second)
            changed[at] ^= 1
            self.assertFalse(equivalent("App.app/Assets.car", self.first, bytes(changed)), at)

    def test_swapped_appearance_ids_are_not(self):
        blocks, variables, _ = _bom(self.second)
        _, _, entries = _appearance_blocks(self.second, blocks, variables)
        self.assertEqual([name for name, _ in entries], [b"UIAppearanceAny", b"UIAppearanceDark"])
        # The two ids, exchanged in place.
        changed = bytearray(self.second)
        values = []
        leaf = struct.unpack_from(">I", self.second, blocks[variables[b"APPEARANCEKEYS"]][0] + 8)[0]
        for k in range(2):
            value = struct.unpack_from(">I", self.second, blocks[leaf][0] + 12 + 8 * k)[0]
            values.append(blocks[value])
        (a, n), (b, _) = values
        changed[a:a + n], changed[b:b + n] = self.second[b:b + n], self.second[a:a + n]
        self.assertFalse(equivalent("App.app/Assets.car", self.first, bytes(changed)))


class CoremldataTest(unittest.TestCase):
    """coremlc writes the maps of a specification it copies (a categorical
    mapping's, a custom model's parameters) and a model's user-defined
    metadata (here in a k-nearest neighbors archive) in a random order."""

    def test_two_xcode_runs_are_equivalent(self):
        for name in ("CatMany", "Custom", "Knn"):
            first, second = read(name + ".1.coremldata.bin"), read(name + ".2.coremldata.bin")
            self.assertNotEqual(first, second, name)
            self.assertTrue(equivalent("M.mlmodelc/coremldata.bin", first, second), name)

    def test_any_changed_byte_is_not(self):
        for name in ("CatMany", "Custom", "Knn"):
            first, second = read(name + ".1.coremldata.bin"), read(name + ".2.coremldata.bin")
            for at in range(len(second)):
                changed = bytearray(second)
                changed[at] ^= 1
                self.assertFalse(equivalent("M.mlmodelc/coremldata.bin", first, bytes(changed)), (name, at))


if __name__ == "__main__":
    unittest.main()
