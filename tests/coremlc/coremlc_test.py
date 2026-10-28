"""coremlc off a Mac, against Xcode 27.0's own output for the same models (see
docs/clean-room/coremlc.md)."""

import glob
import os
import subprocess
import sys
import tempfile
import unittest

from python.runfiles import runfiles
from xcode_equivalence import differences

RUNFILES = runfiles.Create()
TOOL = RUNFILES.Rlocation("rules_applecross/toolchain/stubs/coremlc.py")
HERE = os.path.dirname(RUNFILES.Rlocation("rules_applecross/tests/coremlc/BUILD"))


def run(*args):
    env = dict(os.environ)
    env.pop("RULES_APPLECROSS_RESOURCE_OUTPUT_MODE", None)
    return subprocess.run([sys.executable, TOOL] + list(args), env=env, capture_output=True, text=True)


def tree(root):
    out = {}
    for directory, _, files in os.walk(root):
        for name in files:
            path = os.path.join(directory, name)
            with open(path, "rb") as f:
                out[os.path.relpath(path, root)] = f.read()
    return out


class CoremlcTest(unittest.TestCase):

    def cases(self):
        for case in sorted(os.listdir(os.path.join(HERE, "cases"))):
            package = (glob.glob(os.path.join(HERE, "cases", case, "*.mlpackage")) + glob.glob(os.path.join(HERE, "cases", case, "*.mlmodel")))[0]
            yield case, package, os.path.join(HERE, "expected", case)

    def test_compile_matches_xcode(self):
        count = 0
        for case, package, expected in self.cases():
            if not os.path.isdir(os.path.join(expected, "compiled")):
                continue
            count += 1
            with self.subTest(case=case), tempfile.TemporaryDirectory() as tmp:
                result = run("compile", package, tmp, "--output-partial-info-plist", os.path.join(tmp, "partial.plist"))
                self.assertEqual(result.returncode, 0, result.stderr)
                os.remove(os.path.join(tmp, "partial.plist"))
                # Xcode writes some models' metadata in a random order.
                self.assertEqual(differences(os.path.join(expected, "compiled"), tmp), [])
        self.assertGreater(count, 50)

    def test_generate_matches_xcode(self):
        count = 0
        for case, package, expected in self.cases():
            for language, args in (("swift", ["--public-access", "--language", "Swift"]), ("objc", [])):
                if not os.path.isdir(os.path.join(expected, language)):
                    continue
                count += 1
                with self.subTest(case=case, language=language), tempfile.TemporaryDirectory() as tmp:
                    result = run("generate", package, tmp, *args)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertEqual(tree(tmp), tree(os.path.join(expected, language)))
        self.assertGreater(count, 30)

    def test_refuses_what_xcode_refuses(self):
        # expected/X/refused.txt holds Xcode's error for a model it refuses.
        count = 0
        for case, package, expected in self.cases():
            if not os.path.isfile(os.path.join(expected, "refused.txt")):
                continue
            count += 1
            with self.subTest(case=case), tempfile.TemporaryDirectory() as tmp:
                result = run("compile", package, tmp)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(os.listdir(tmp), [])
        self.assertGreater(count, 0)

    def test_refuses_neural_networks(self):
        with tempfile.TemporaryDirectory() as tmp:
            # A neural network: the Model message's field 500.
            model = os.path.join(tmp, "Old.mlmodel")
            with open(model, "wb") as f:
                f.write(bytes([0x08, 0x04, 0xa2, 0x1f, 0x00]))
            result = run("compile", model, tmp)
            self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
