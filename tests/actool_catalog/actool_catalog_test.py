#!/usr/bin/env python3
"""Asset catalogs through this repository's actool, against Xcode 27.0's
output (expected/, from update_expected.py): every file the same, up to
what Xcode itself varies (tools/xcode_equivalence.py).

    python3 tests/actool_catalog/actool_catalog_test.py PATH/TO/actool
"""

import os
import sys
import tempfile

from common import HERE, cases, compile_catalog, differences


def main(argv):
    if len(argv) != 1:
        sys.exit(__doc__)
    tool = [os.path.abspath(argv[0])]
    failures, count = [], 0
    for case, catalog, icon, platforms in cases():
        for platform in platforms:
            count += 1
            with tempfile.TemporaryDirectory() as out:
                status, log = compile_catalog(tool, catalog, icon, platform, out)
                if status:
                    failures.append("%s %s: refused:\n%s" % (case, platform, log))
                    continue
                different = differences(os.path.join(HERE, "expected", case, platform), out)
                if different:
                    failures.append("%s %s: %s" % (case, platform, "; ".join(different)))
    for f in failures:
        print(f)
    print("%d of %d compiled as Xcode does" % (count - len(failures), count))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
