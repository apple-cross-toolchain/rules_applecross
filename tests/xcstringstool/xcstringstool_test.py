"""xcstringstool compile off a Mac, against Xcode 27.0's own output for the
same String Catalogs (see docs/clean-room/xcstringstool.md)."""

import os
import subprocess
import sys
import tempfile
import unittest

from python.runfiles import runfiles

RUNFILES = runfiles.Create()
TOOL = RUNFILES.Rlocation("rules_applecross/toolchain/stubs/xcstringstool.py")
HERE = os.path.dirname(RUNFILES.Rlocation("rules_applecross/tests/xcstringstool/BUILD"))


def compile_catalog(path, out, relaxed=False):
    env = dict(os.environ)
    env.pop("RULES_APPLECROSS_RESOURCE_OUTPUT_MODE", None)
    if relaxed:
        env["RULES_APPLECROSS_RESOURCE_OUTPUT_MODE"] = "relaxed"
    return subprocess.run([sys.executable, TOOL, "compile", path, "--output-directory", out],
                          env=env, capture_output=True, text=True)


def tree(root):
    """{relative path: bytes} of files, and each empty directory as path/."""
    out = {}
    if not os.path.isdir(root):
        return out
    for directory, dirs, files in os.walk(root):
        rel = os.path.relpath(directory, root)
        names = [f for f in files if f != ".empty"]
        if ".empty" in files or (not dirs and not names and rel != "."):
            out[rel + "/"] = b""
        for name in names:
            with open(os.path.join(directory, name), "rb") as f:
                out[os.path.normpath(os.path.join(rel, name))] = f.read()
    return out


class XcstringstoolTest(unittest.TestCase):

    def test_matches_xcode(self):
        cases = sorted(f[:-len(".xcstrings")] for f in os.listdir(os.path.join(HERE, "cases")))
        self.assertGreater(len(cases), 300)
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as tmp:
                out = os.path.join(tmp, "out")
                result = compile_catalog(os.path.join(HERE, "cases", case + ".xcstrings"), out)
                if os.path.exists(os.path.join(HERE, "expected", case + ".fails")):
                    self.assertNotEqual(result.returncode, 0, result.stderr)
                    continue
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(tree(out), tree(os.path.join(HERE, "expected", case)))

    def test_refuses_unestablished_input_unless_relaxed(self):
        catalog = os.path.join(HERE, "refused", "manual_two_arguments.xcstrings")
        with tempfile.TemporaryDirectory() as tmp:
            strict = compile_catalog(catalog, os.path.join(tmp, "strict"))
            self.assertNotEqual(strict.returncode, 0)
            self.assertIn("--config=relaxed", strict.stderr)
            relaxed = compile_catalog(catalog, os.path.join(tmp, "relaxed"), relaxed=True)
            self.assertEqual(relaxed.returncode, 0, relaxed.stderr)
            self.assertIn("relaxed", relaxed.stderr)
            self.assertIn("de.lproj/manual_two_arguments.stringsdict", tree(os.path.join(tmp, "relaxed")))


if __name__ == "__main__":
    unittest.main()
