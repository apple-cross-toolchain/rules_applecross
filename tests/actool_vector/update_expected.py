#!/usr/bin/env python3
"""Writes expected/<case>.txt from Xcode's own actool (on a Mac with Xcode
27.2): each platform's output digests. Each case is compiled twice, and its
digests must agree, so what's kept is what Xcode keeps from run to run."""

import json
import os
import sys
import tempfile

from common import HERE, cases, compile_catalog, digests, platforms
from xcode_equivalence import actool_diagnostics as diagnostics


def main():
    os.makedirs(os.path.join(HERE, "expected"), exist_ok=True)
    for case, catalog in cases():
        expected = {}
        for platform in platforms(case):
            runs = []
            for _ in range(2):
                with tempfile.TemporaryDirectory() as out:
                    status, log = compile_catalog(["xcrun", "actool"], catalog, platform, out)
                    if status or diagnostics(log):
                        sys.exit("%s %s: Xcode refused it, or reports errors that fail rules_apple's build:\n%s" % (case, platform, log))
                    runs.append(digests(out))
            if runs[0] != runs[1]:
                sys.exit("%s %s: Xcode's two runs differ beyond what's known to vary" % (case, platform))
            expected[platform] = runs[0]
        # A line a file, so a change shows which.
        lines = ["%s: %s" % (json.dumps([platform, path]), json.dumps(digest, separators=(",", ":")))
                 for platform in sorted(expected) for path, digest in sorted(expected[platform].items())]
        with open(os.path.join(HERE, "expected", case + ".txt"), "w") as f:
            f.write("\n".join(lines) + "\n")
        print(case)


if __name__ == "__main__":
    main()
