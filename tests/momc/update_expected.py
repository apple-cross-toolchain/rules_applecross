#!/usr/bin/env python3
"""Writes expected/<case>/ from Xcode's own momc (on a Mac with the Xcode the
tests are against): each model version's .mom, as `momc` compiles
cases/<case>/Model.xcdatamodeld for the iOS simulator; and
generate_expected/<case>/, the Swift classes `momc --action generate`
writes for generate/<case>/Model.xcdatamodeld, with the header's line
naming the user and the date replaced by the one this momc writes. A single
version's VersionInfo.plist is kept, but not one with a derived attribute
(whose version hash isn't reproduced); several versions' VersionInfo.plist
and the .omo beside the current one aren't: off a Mac they're refused, or
written without what isn't established (docs/clean-room/momc.md).
Run from anywhere:

    python3 tests/momc/update_expected.py
"""

import glob
import os
import re
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
            versions = glob.glob(os.path.join(model, "*.xcdatamodel"))
            derived = any('derived="YES"' in open(os.path.join(v, "contents")).read() for v in versions)
            if len(versions) == 1 and not derived:
                shutil.copy(glob.glob(os.path.join(tmp, "*.momd", "VersionInfo.plist"))[0], expected)
    for case in sorted(os.listdir(os.path.join(HERE, "generate"))):
        model = os.path.join(HERE, "generate", case, "Model.xcdatamodeld")
        expected = os.path.join(HERE, "generate_expected", case)
        shutil.rmtree(expected, ignore_errors=True)
        os.makedirs(expected)
        with tempfile.TemporaryDirectory() as tmp:
            result = subprocess.run(["xcrun", "momc", "--action", "generate", "--iphonesimulator-deployment-target", "18.0",
                                     model, tmp], capture_output=True, text=True)
            if result.returncode != 0:
                sys.exit("%s: Xcode's momc failed to generate:\n%s" % (case, result.stderr))
            for name in os.listdir(tmp):
                with open(os.path.join(tmp, name)) as f:
                    text = re.sub(r"(?m)^//  Created by .*$", "//  Created by momc.", f.read())
                with open(os.path.join(expected, name), "w") as f:
                    f.write(text)
        # Kept in git when Xcode writes nothing.
        if not os.listdir(expected):
            open(os.path.join(expected, ".empty"), "w").close()


if __name__ == "__main__":
    main()
