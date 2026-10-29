#!/usr/bin/env python3
"""Writes expected/<case>/<platform>/ from Xcode's own actool (on a Mac with
Xcode 27.0). Each case is compiled twice, and the runs must agree up to what
Xcode is known to vary (tools/xcode_equivalence.py)."""

import os
import shutil
import sys
import tempfile

from common import HERE, cases, compile_catalog, differences


def main():
    for case, catalog, icon, platforms in cases():
        for platform in platforms:
            out = os.path.join(HERE, "expected", case, platform)
            shutil.rmtree(out, ignore_errors=True)
            os.makedirs(out)
            status, log = compile_catalog(["xcrun", "actool"], catalog, icon, platform, out)
            if status:
                sys.exit("%s %s: Xcode refused it:\n%s" % (case, platform, log))
            with tempfile.TemporaryDirectory() as again:
                compile_catalog(["xcrun", "actool"], catalog, icon, platform, again)
                if differences(out, again):
                    sys.exit("%s %s: Xcode's two runs differ beyond what's known to vary" % (case, platform))
        print(case, ", ".join(platforms))


if __name__ == "__main__":
    main()
