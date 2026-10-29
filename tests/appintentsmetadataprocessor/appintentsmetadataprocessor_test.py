"""appintentsmetadataprocessor off a Mac, against Xcode 27.0's own output for
the same Swift constant values (see
docs/clean-room/appintentsmetadataprocessor.md)."""

import json
import os
import subprocess
import sys
import tempfile
import unittest

from python.runfiles import runfiles

RUNFILES = runfiles.Create()
TOOL = RUNFILES.Rlocation("rules_applecross/toolchain/stubs/appintentsmetadataprocessor.py")
HERE = os.path.dirname(RUNFILES.Rlocation("rules_applecross/tests/appintentsmetadataprocessor/BUILD"))


def process(consts, out):
    """Runs the tool as rules_apple does, with consts as the module's only
    constant values file."""
    with open(os.path.join(out, "consts.txt"), "w") as f:
        f.write(consts + "\n")
    with open(os.path.join(out, "sources.txt"), "w") as f:
        f.write("MyApp.swift\n")
    env = dict(os.environ, SWIFT_DETERMINISTIC_HASHING="1")
    for name in [n for n in env if n.startswith("RULES_APPLECROSS_RELAX") or n == "RULES_APPLECROSS_RESOURCE_OUTPUT_MODE"]:
        del env[name]
    return subprocess.run([
        sys.executable, TOOL,
        "--binary-file", "/bazel_rules_apple/fakepath",
        "--module-name", "MyApp",
        "--output", out,
        "--sdk-root", "/sdk",
        "--platform-family", "iOS",
        "--deployment-target", "17.0",
        "--target-triple", "arm64-apple-ios17.0-simulator",
        "--compile-time-extraction",
        "--xcode-version", "27A266a",
        "--source-file-list", os.path.join(out, "sources.txt"),
        "--swift-const-vals-list", os.path.join(out, "consts.txt"),
        "--sdk-root", "/sdk",
        "--toolchain-dir", "/toolchain",
    ], env=env, capture_output=True, text=True)


class AppIntentsMetadataProcessorTest(unittest.TestCase):

    def test_matches_xcode(self):
        cases = sorted(f[:-len(".json")] for f in os.listdir(os.path.join(HERE, "cases")))
        self.assertGreater(len(cases), 70)
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as tmp:
                result = process(os.path.join(HERE, "cases", case + ".json"), tmp)
                fails = os.path.join(HERE, "expected", case + ".fails")
                if os.path.exists(fails):
                    self.assertNotEqual(result.returncode, 0)
                    with open(fails) as f:
                        self.assertIn(f.read().strip(), result.stderr)
                    continue
                self.assertEqual(result.returncode, 0, result.stderr)
                with open(os.path.join(tmp, "Metadata.appintents", "extract.actionsdata")) as f:
                    actual = json.load(f)
                with open(os.path.join(HERE, "expected", case + ".json")) as f:
                    expected = json.load(f)
                self.assertEqual(actual, expected)
                with open(os.path.join(tmp, "Metadata.appintents", "version.json")) as f:
                    self.assertEqual(json.load(f), {"toolsVersion": "27A266a", "version": "3.0"})


if __name__ == "__main__":
    unittest.main()
