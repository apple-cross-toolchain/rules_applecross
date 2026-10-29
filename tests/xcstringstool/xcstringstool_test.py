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


def compile_catalog(path, out, **relax):
    """`relax`: the environment's relaxations, such as
    RULES_APPLECROSS_RELAX_XCSTRINGS="1"; none by default."""
    env = dict(os.environ)
    for name in [n for n in env if n.startswith("RULES_APPLECROSS_RELAX") or n == "RULES_APPLECROSS_RESOURCE_OUTPUT_MODE"]:
        del env[name]
    env.update(relax)
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
            self.assertIn("--config=relax_xcstrings", strict.stderr)
            # Another tool's relaxation leaves this one strict.
            other = compile_catalog(catalog, os.path.join(tmp, "other"), RULES_APPLECROSS_RELAX_METAL="1")
            self.assertNotEqual(other.returncode, 0)
            # Its own, or all of them (--config=relaxed's older form), relaxes it.
            for name, relax in (("own", {"RULES_APPLECROSS_RELAX_XCSTRINGS": "1"}),
                                ("all", {"RULES_APPLECROSS_RESOURCE_OUTPUT_MODE": "relaxed"})):
                relaxed = compile_catalog(catalog, os.path.join(tmp, name), **relax)
                self.assertEqual(relaxed.returncode, 0, relaxed.stderr)
                self.assertIn("relaxed", relaxed.stderr)
                self.assertIn("de.lproj/manual_two_arguments.stringsdict", tree(os.path.join(tmp, name)))


if __name__ == "__main__":
    unittest.main()
