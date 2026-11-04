#!/usr/bin/env python3
"""Vector image sets (PDF, SVG) through this repository's actool, against
Xcode 27.2's output (expected/, from update_expected.py): every file and
every rendition the same, but for the pixels of the bitmaps it draws from
them and a PDF's stored bytes (tools/xcode_equivalence.py
vector_car_digest), with --config=relax_actool_vector; refused without it.

    python3 tests/actool_vector/actool_vector_test.py PATH/TO/actool
"""

import json
import os
import sys
import tempfile

from common import HERE, cases, compile_catalog, digests, platforms


def main(argv):
    if len(argv) != 1:
        sys.exit(__doc__)
    tool = [os.path.abspath(argv[0])]
    relaxed = dict(os.environ, RULES_APPLECROSS_RELAX_ACTOOL_VECTOR="1")
    strict = {k: v for k, v in os.environ.items() if not k.startswith("RULES_APPLECROSS_RELAX") and k != "RULES_APPLECROSS_RESOURCE_OUTPUT_MODE"}
    failures, count = [], 0
    for case, catalog in cases():
        expected = {}
        with open(os.path.join(HERE, "expected", case + ".txt")) as f:
            for line in f:
                key, digest = line.split(": ", 1)
                platform, path = json.loads(key)
                expected.setdefault(platform, {})[path] = json.loads(digest)
        for platform in platforms(case):
            count += 1
            with tempfile.TemporaryDirectory() as out:
                status, log = compile_catalog(tool, catalog, platform, out, relaxed)
                if status:
                    failures.append("%s %s: refused:\n%s" % (case, platform, log))
                    continue
                ours = json.loads(json.dumps(digests(out)))
            want = expected[platform]
            different = sorted(p for p in set(want) | set(ours) if want.get(p) != ours.get(p))
            if different:
                failures.append("%s %s: different from Xcode's: %s" % (case, platform, ", ".join(different)))
        with tempfile.TemporaryDirectory() as out:
            status, _ = compile_catalog(tool, catalog, platforms(case)[0], out, strict)
            if status == 0:
                failures.append("%s: compiled strict, where its bitmaps can't be Xcode's" % case)
    for f in failures:
        print(f)
    print("%d of %d compiled as Xcode does" % (count - len([f for f in failures if "strict" not in f]), count))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
