"""momc off a Mac, against Xcode 27.2's own output for the same models (see
docs/clean-room/momc.md)."""

import glob
import os
import plistlib
import subprocess
import sys
import tempfile
import unittest

from python.runfiles import runfiles

RUNFILES = runfiles.Create()
TOOL = RUNFILES.Rlocation("rules_applecross/toolchain/stubs/momc.py")
HERE = os.path.dirname(RUNFILES.Rlocation("rules_applecross/tests/momc/BUILD"))


# The Xcode the expected outputs are from: momc records its version in a
# model, by the version.plist beside DEVELOPER_DIR, which off a Mac is this
# stand-in.
def stand_in_xcode(version, build):
    root = tempfile.mkdtemp(prefix="xcode-identity-")
    os.makedirs(os.path.join(root, "Contents", "Developer"))
    with open(os.path.join(root, "Contents", "version.plist"), "wb") as f:
        plistlib.dump({"CFBundleShortVersionString": version, "ProductBuildVersion": build}, f)
    return root


XCODE_ROOT = stand_in_xcode("27.2", "27B52")
RELAXED = {"RULES_APPLECROSS_RELAX_MOMC": "1"}


def run(*args, xcode_root=XCODE_ROOT, env=None):
    full = dict(os.environ, DEVELOPER_DIR=os.path.join(xcode_root, "Contents", "Developer"))
    for name in [n for n in full if n.startswith("RULES_APPLECROSS_RELAX") or n == "RULES_APPLECROSS_RESOURCE_OUTPUT_MODE"]:
        del full[name]
    full.update(env or {})
    return subprocess.run([sys.executable, TOOL] + list(args), env=full, capture_output=True, text=True)


def model(directory):
    return glob.glob(os.path.join(directory, "*.xcdatamodeld"))[0]


class MomcTest(unittest.TestCase):

    def test_models_match_xcode(self):
        cases = sorted(os.listdir(os.path.join(HERE, "cases")))
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as tmp:
                out = os.path.join(tmp, "Model.momd")
                result = run("--iphonesimulator-deployment-target", "18.0", "--module", "App",
                             model(os.path.join(HERE, "cases", case)), out, env=RELAXED)
                self.assertEqual(result.returncode, 0, result.stderr)
                expected = os.path.join(HERE, "expected", case)
                self.assertEqual(sorted(n for n in os.listdir(out) if n.endswith(".mom")), sorted(os.listdir(expected)))
                for name in os.listdir(expected):
                    with open(os.path.join(expected, name), "rb") as f, open(os.path.join(out, name), "rb") as g:
                        self.assertTrue(f.read() == g.read(), "%s/%s differs from Xcode's" % (case, name))
        self.assertGreater(len(cases), 70)

    def test_version_info_names_only_the_current_version(self):
        with tempfile.TemporaryDirectory() as tmp:
            for case, info in (("versioned", {"NSManagedObjectModel_CurrentVersionName": "Model2"}), ("two_entities", {})):
                out = os.path.join(tmp, case + ".momd")
                result = run(model(os.path.join(HERE, "cases", case)), out, env=RELAXED)
                self.assertEqual(result.returncode, 0, result.stderr)
                with open(os.path.join(out, "VersionInfo.plist"), "rb") as f:
                    self.assertEqual(plistlib.load(f), info)

    def test_one_version_compiles_to_a_mom(self):
        version = os.path.join(model(os.path.join(HERE, "cases", "two_entities")), "Model.xcdatamodel")
        with tempfile.TemporaryDirectory() as tmp:
            result = run(version, os.path.join(tmp, "Model.mom"), env=RELAXED)
            self.assertEqual(result.returncode, 0, result.stderr)
            with open(os.path.join(HERE, "expected", "two_entities", "Model.mom"), "rb") as f, \
                    open(os.path.join(tmp, "Model.mom"), "rb") as g:
                self.assertEqual(f.read(), g.read())

    def test_refused_without_relax_momc(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = run(model(os.path.join(HERE, "cases", "two_entities")), os.path.join(tmp, "Model.momd"))
            self.assertEqual(result.returncode, 1)
            self.assertIn("--config=relax_momc", result.stderr)
            self.assertEqual(os.listdir(tmp), [])

    def test_refused_models(self):
        cases = sorted(os.listdir(os.path.join(HERE, "refused")))
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as tmp:
                result = run(model(os.path.join(HERE, "refused", case)), os.path.join(tmp, "Model.momd"), env=RELAXED)
                self.assertEqual(result.returncode, 1, result.stdout)
                self.assertTrue(result.stderr.startswith("error: momc: "), result.stderr)
                self.assertNotIn("Traceback", result.stderr)
        self.assertGreater(len(cases), 5)

    def test_unknown_xcode(self):
        case = model(os.path.join(HERE, "cases", "two_entities"))
        with tempfile.TemporaryDirectory() as tmp:
            result = run(case, os.path.join(tmp, "a.momd"), xcode_root=stand_in_xcode("28.0", "28A1"), env=RELAXED)
            self.assertEqual(result.returncode, 1)
            self.assertIn("relax_xcode_version", result.stderr)
            result = run(case, os.path.join(tmp, "b.momd"), xcode_root=stand_in_xcode("28.0", "28A1"),
                         env=dict(RELAXED, RULES_APPLECROSS_RELAX_XCODE_VERSION="1"))
            self.assertEqual(result.returncode, 0, result.stderr)
            with open(os.path.join(HERE, "expected", "two_entities", "Model.mom"), "rb") as f, \
                    open(os.path.join(tmp, "b.momd", "Model.mom"), "rb") as g:
                self.assertEqual(f.read(), g.read())

    def test_generate_matches_xcode(self):
        cases = sorted(os.listdir(os.path.join(HERE, "generate")))
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as tmp:
                out = os.path.join(tmp, "out")
                result = run("--action", "generate", "--iphonesimulator-deployment-target", "18.0",
                             model(os.path.join(HERE, "generate", case)), out, env=RELAXED)
                self.assertEqual(result.returncode, 0, result.stderr)
                expected = os.path.join(HERE, "generate_expected", case)
                names = sorted(n for n in os.listdir(expected) if n != ".empty")
                self.assertEqual(sorted(os.listdir(out)) if os.path.isdir(out) else [], names)
                for name in names:
                    with open(os.path.join(expected, name), "rb") as f, open(os.path.join(out, name), "rb") as g:
                        self.assertTrue(f.read() == g.read(), "%s/%s differs from Xcode's" % (case, name))
        self.assertGreater(len(cases), 5)

    def test_generate_refused_without_relax_momc(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = run("--action", "generate", model(os.path.join(HERE, "generate", "types")), tmp)
            self.assertEqual(result.returncode, 1)
            self.assertIn("--config=relax_momc", result.stderr)


if __name__ == "__main__":
    unittest.main()
