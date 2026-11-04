#!/usr/bin/env python3
"""Writes expected/<case>/ from Xcode's own momc (on a Mac with the Xcode the
tests are against): each model version's .mom, as `momc` compiles
cases/<case>/Model.xcdatamodeld for the iOS simulator. VersionInfo.plist and
the .omo beside a versioned model's current version aren't kept: off a Mac
they're written without what isn't established (docs/clean-room/momc.md).
Run from anywhere:

    python3 tests/momc/update_expected.py
"""

import glob
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))


def main():
    sdk = subprocess.run(["xcrun", "--sdk", "iphonesimulator", "--show-sdk-path"],
                         capture_output=True, text=True, check=True).stdout.strip()
    for case in sorted(os.listdir(os.path.join(HERE, "cases"))):
        model = glob.glob(os.path.join(HERE, "cases", case, "*.xcdatamodeld"))[0]
        expected = os.path.join(HERE, "expected", case)
        shutil.rmtree(expected, ignore_errors=True)
        os.makedirs(expected)
        with tempfile.TemporaryDirectory() as tmp:
            result = subprocess.run(["xcrun", "momc", "--sdkroot", sdk, "--iphonesimulator-deployment-target", "18.0",
                                     "--module", "App", model, tmp], capture_output=True, text=True)
            if result.returncode != 0:
                sys.exit("%s: Xcode's momc failed:\n%s" % (case, result.stderr))
            for mom in glob.glob(os.path.join(tmp, "*.momd", "*.mom")):
                shutil.copy(mom, expected)


if __name__ == "__main__":
    main()
