#!/usr/bin/env python3
"""iOS xibs through this repository's ibtool, strict, against Xcode 27's
nibs (expected/, from update_expected.py): byte for byte.

    python3 tests/ibtool_xib/ibtool_xib_test.py PATH/TO/ibtool
"""

import os
import sys
import tempfile

from common import HERE, cases, compile_xib, digest


def main(argv):
    if len(argv) != 1:
        sys.exit(__doc__)
    tool = [os.path.abspath(argv[0])]
    strict = {k: v for k, v in os.environ.items() if not k.startswith("RULES_APPLECROSS_RELAX") and k != "RULES_APPLECROSS_RESOURCE_OUTPUT_MODE"}
    failures, count = [], 0
    for case, xib in cases():
        count += 1
        with open(os.path.join(HERE, "expected", case + ".txt")) as f:
            want = f.read().strip()
        with tempfile.TemporaryDirectory() as out:
            nib = os.path.join(out, case + ".nib")
            status, log = compile_xib(tool, xib, nib, strict)
            if status:
                failures.append("%s: refused:\n%s" % (case, log))
            elif digest(nib) != want:
                failures.append("%s: different from Xcode's" % case)
    for f in failures:
        print(f)
    print("%d of %d compiled as Xcode does" % (count - len(failures), count))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
