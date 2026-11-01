"""coremlc off a Mac, against Xcode 27.2's own output for the same models (see
docs/clean-room/coremlc.md)."""

import glob
import os
import plistlib
import subprocess
import sys
import tempfile
import unittest

from python.runfiles import runfiles
from xcode_equivalence import differences

RUNFILES = runfiles.Create()
TOOL = RUNFILES.Rlocation("rules_applecross/toolchain/stubs/coremlc.py")
HERE = os.path.dirname(RUNFILES.Rlocation("rules_applecross/tests/coremlc/BUILD"))


# The Xcode the expected outputs are from: coremlc records its version in a
# model's buildInfo, by the version.plist beside DEVELOPER_DIR, which off a
# Mac is this stand-in.
def stand_in_xcode(version, build):
    root = tempfile.mkdtemp(prefix="xcode-identity-")
    os.makedirs(os.path.join(root, "Contents", "Developer"))
    with open(os.path.join(root, "Contents", "version.plist"), "wb") as f:
        plistlib.dump({"CFBundleShortVersionString": version, "ProductBuildVersion": build}, f)
    return root


XCODE_ROOT = stand_in_xcode("27.2", "27B52")


def run(*args, xcode_root=XCODE_ROOT, env=None):
    full = dict(os.environ, DEVELOPER_DIR=os.path.join(xcode_root, "Contents", "Developer"))
    for name in [n for n in full if n.startswith("RULES_APPLECROSS_RELAX") or n == "RULES_APPLECROSS_RESOURCE_OUTPUT_MODE"]:
        del full[name]
    full.update(env or {})
    return subprocess.run([sys.executable, TOOL] + list(args), env=full, capture_output=True, text=True)


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

    def test_refuses_encodings_xcode_would_change(self):
        # Xcode re-encodes the specification it copies; one whose fields are
        # out of order would come out different, so it is refused.
        source = glob.glob(os.path.join(HERE, "cases", "classic_Afe", "*.mlmodel"))[0]
        with open(source, "rb") as f:
            data = f.read()
        self.assertEqual(data[0], 0x08)  # field 1, the specification version
        with tempfile.TemporaryDirectory() as tmp:
            self.assertEqual(run("compile", source, tmp).returncode, 0)
        end = 2
        while data[end - 1] & 0x80:
            end += 1
        with tempfile.TemporaryDirectory() as tmp:
            model = os.path.join(tmp, "Reordered.mlmodel")
            with open(model, "wb") as f:
                f.write(data[end:] + data[:end])
            result = run("compile", model, tmp)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("encoded", result.stderr)

    def test_unknown_xcode_refused_unless_relaxed(self):
        # Xcode 26.6's buildInfo isn't known: refused, or relaxed recorded
        # with the nearest known Xcode's, 27.0's.
        package = glob.glob(os.path.join(HERE, "cases", "cmp_A16_ret", "*.mlpackage"))[0]
        xcode_26 = stand_in_xcode("26.6", "17F113")
        with tempfile.TemporaryDirectory() as tmp:
            result = run("compile", package, tmp, xcode_root=xcode_26)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Xcode 26.6 (17F113) is not supported", result.stderr)
            self.assertEqual(os.listdir(tmp), [])
        for env in ({"RULES_APPLECROSS_RELAX_XCODE_VERSION": "1"}, {"RULES_APPLECROSS_RESOURCE_OUTPUT_MODE": "relaxed"}):
            with self.subTest(env=env), tempfile.TemporaryDirectory() as tmp:
                result = run("compile", package, tmp, xcode_root=xcode_26, env=env)
                self.assertEqual(result.returncode, 0, result.stderr)
                with open(glob.glob(os.path.join(tmp, "*.mlmodelc", "model.mil"))[0]) as f:
                    mil = f.read()
                self.assertIn('"coremlc-component-MIL", "3600.16.1"', mil)

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
