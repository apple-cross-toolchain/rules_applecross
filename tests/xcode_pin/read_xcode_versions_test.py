"""read_xcode_versions.py reports the Swift that built the SDKs, from the
macOS SDK's standard library interface."""

import os
import plistlib
import subprocess
import sys
import tempfile
import unittest

SCRIPT = sys.argv.pop(1)


class ReadXcodeVersionsTest(unittest.TestCase):
    def test_swift_version(self):
        with tempfile.TemporaryDirectory() as tmp:
            app = os.path.join(tmp, "Xcode.app")
            os.makedirs(os.path.join(app, "Contents"))
            with open(os.path.join(app, "Contents", "version.plist"), "wb") as f:
                plistlib.dump({"CFBundleShortVersionString": "27.2", "ProductBuildVersion": "27B48"}, f)
            swiftmodule = os.path.join(app, "Contents", "Developer", "Platforms", "MacOSX.platform", "Developer",
                                       "SDKs", "MacOSX.sdk", "usr", "lib", "swift", "Swift.swiftmodule")
            os.makedirs(swiftmodule)
            with open(os.path.join(swiftmodule, "arm64e-apple-macos.swiftinterface"), "w") as f:
                f.write("// swift-interface-format-version: 1.0\n"
                        "// swift-compiler-version: Apple Swift version 6.4.2 effective-5.10 (swiftlang-6.4.2.1.9 clang-2100.4.1.1)\n")
            out = subprocess.run([sys.executable, SCRIPT, app], capture_output=True, text=True, check=True).stdout
            self.assertEqual(["xcode 27.2", "build 27B48", "swift 6.4.2"], out.splitlines()[:3])


if __name__ == "__main__":
    unittest.main()
