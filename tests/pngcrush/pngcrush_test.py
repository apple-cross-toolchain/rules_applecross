#!/usr/bin/env python3
"""PNGs through this repository's pngcrush as rules_apple's copypng runs it,
against Xcode 27's (expected/, from update_expected.py): byte for byte,
strict. Those in refused/ are refused strict and copied as they are with
RULES_APPLECROSS_RELAX_PNGCRUSH=1.

    python3 tests/pngcrush/pngcrush_test.py PATH/TO/pngcrush
"""

import os
import sys
import tempfile

from common import HERE, cases, crush, digest


def main(argv):
    if len(argv) != 1:
        sys.exit(__doc__)
    tool = [os.path.abspath(argv[0])]
    strict = {k: v for k, v in os.environ.items() if not k.startswith("RULES_APPLECROSS_RELAX") and k != "RULES_APPLECROSS_RESOURCE_OUTPUT_MODE"}
    failures, count = [], 0
    for case, png in cases():
        count += 1
        with open(os.path.join(HERE, "expected", case + ".txt")) as f:
            want = f.read().strip()
        with tempfile.TemporaryDirectory() as out:
            path = os.path.join(out, case + ".png")
            status, log = crush(tool, png, path, strict)
            if status:
                failures.append("%s: refused:\n%s" % (case, log))
            elif digest(path) != want:
                failures.append("%s: different from Xcode's" % case)
    for case, png in cases("refused"):
        count += 1
        with tempfile.TemporaryDirectory() as out:
            path = os.path.join(out, case + ".png")
            status, _ = crush(tool, png, path, strict)
            if status == 0:
                failures.append("%s: compiled strict" % case)
                continue
            status, log = crush(tool, png, path, dict(strict, RULES_APPLECROSS_RELAX_PNGCRUSH="1"))
            if status or digest(path) != digest(png):
                failures.append("%s: not copied as it is when relaxed:\n%s" % (case, log))
    for f in failures:
        print(f)
    print("%d of %d as expected" % (count - len(failures), count))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
