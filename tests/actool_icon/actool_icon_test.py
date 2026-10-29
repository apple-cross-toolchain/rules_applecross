#!/usr/bin/env python3
"""Icon Composer documents through this repository's actool, against Xcode
27.0's output (expected/, from update_expected.py): every file and every
rendition the same, the flattened images' pixels aside, with
--config=relax_actool_icon_composer; refused without it.

    python3 tests/actool_icon/actool_icon_test.py PATH/TO/actool
"""

import json
import os
import sys
import tempfile

from common import HERE, cases, compile_icon, digests


def main(argv):
    if len(argv) != 1:
        sys.exit(__doc__)
    tool = [os.path.abspath(argv[0])]
    relaxed = dict(os.environ, RULES_APPLECROSS_RELAX_ACTOOL_ICON_COMPOSER="1")
    strict = {k: v for k, v in os.environ.items() if not k.startswith("RULES_APPLECROSS_RELAX") and k != "RULES_APPLECROSS_RESOURCE_OUTPUT_MODE"}
    failures, count = [], 0
    for case, icon, catalogs, platforms in cases():
        expected = {}
        with open(os.path.join(HERE, "expected", case + ".txt")) as f:
            for line in f:
                key, digest = line.split(": ", 1)
                platform, path = json.loads(key)
                expected.setdefault(platform, {})[path] = json.loads(digest)
        for platform in platforms:
            count += 1
            with tempfile.TemporaryDirectory() as out:
                status, log = compile_icon(tool, icon, catalogs, platform, out, relaxed)
                if status:
                    failures.append("%s %s: refused:\n%s" % (case, platform, log))
                    continue
                ours = json.loads(json.dumps(digests(out)))
            want = expected[platform]
            different = sorted(p for p in set(want) | set(ours) if want.get(p) != ours.get(p))
            if different:
                failures.append("%s %s: different from Xcode's: %s" % (case, platform, ", ".join(different)))
        with tempfile.TemporaryDirectory() as out:
            status, _ = compile_icon(tool, icon, catalogs, "sim", out, strict)
            if status == 0:
                failures.append("%s: compiled strict, where its rasters can't be Xcode's" % case)
    for f in failures:
        print(f)
    print("%d of %d compiled as Xcode does" % (count - len([f for f in failures if "strict" not in f]), count))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
