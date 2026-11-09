#!/usr/bin/env python3
"""iOS xibs and storyboards through this repository's ibtool, strict, against Xcode 27's
nibs (expected/, from update_expected.py): byte for byte.

AppKit colours embed the Mac's colour profiles, which this ibtool reads from
the SDK archive's DEVELOPER_DIR (tools/package-sdks.sh), or on a Mac from
/System/Library/ColorSync. Without them, the cases needing them are listed
as skipped.

    python3 tests/ibtool_xib/ibtool_xib_test.py PATH/TO/ibtool
"""

import os
import shutil
import sys
import tempfile

from common import HERE, cases, compile_xib, digest


def main(argv):
    if len(argv) != 1:
        sys.exit(__doc__)
    tool = [os.path.abspath(argv[0])]
    strict = {k: v for k, v in os.environ.items() if not k.startswith("RULES_APPLECROSS_RELAX") and k != "RULES_APPLECROSS_RESOURCE_OUTPUT_MODE"}
    with tempfile.TemporaryDirectory() as developer:
        if not profiles(strict.get("DEVELOPER_DIR")) and os.path.isdir(_SYSTEM_PROFILES):
            os.makedirs(os.path.join(developer, _PROFILES))
            for name, system in _PROFILE_NAMES.items():
                shutil.copyfile(os.path.join(_SYSTEM_PROFILES, system), os.path.join(developer, _PROFILES, name))
            strict["DEVELOPER_DIR"] = developer
        return run(tool, strict)


_PROFILES = "Toolchains/XcodeDefault.xctoolchain/usr/share/colorsync"
_SYSTEM_PROFILES = "/System/Library/ColorSync/Profiles"
_PROFILE_NAMES = {"srgb.icc": "sRGB Profile.icc", "generic_gray_gamma_2_2.icc": "Generic Gray Gamma 2.2 Profile.icc"}


def profiles(developer):
    """Whether DEVELOPER_DIR has the colour profiles."""
    return bool(developer) and all(os.path.isfile(os.path.join(developer, _PROFILES, n)) for n in _PROFILE_NAMES)


def run(tool, strict):
    failures, skipped, count = [], [], 0
    have_profiles = profiles(strict.get("DEVELOPER_DIR"))
    for case, xib in cases():
        count += 1
        with open(os.path.join(HERE, "expected", case + ".txt")) as f:
            want = f.read().strip()
        with tempfile.TemporaryDirectory() as out:
            nib = os.path.join(out, case + ".nib")
            status, log = compile_xib(tool, xib, nib, strict)
            if status and not have_profiles and "colour profiles" in log:
                skipped.append(case)
            elif status:
                failures.append("%s: refused:\n%s" % (case, log))
            elif digest(nib) != want:
                failures.append("%s: different from Xcode's" % case)
    for f in failures:
        print(f)
    if skipped:
        print("skipped, without the Mac's colour profiles: " + ", ".join(skipped))
    print("%d of %d compiled as Xcode does" % (count - len(failures) - len(skipped), count))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
