#!/usr/bin/env python3
"""Writes expected/<case>.txt from Xcode's own ibtool (on a Mac with Xcode
27): the digest of the nib it compiles (or of each nib directory's files,
common.digest). Each case is compiled twice, and the digests must agree."""

import os
import sys
import tempfile

from common import HERE, cases, compile_xib, digest


def main():
    os.makedirs(os.path.join(HERE, "expected"), exist_ok=True)
    for case, xib in cases():
        runs = []
        for _ in range(2):
            with tempfile.TemporaryDirectory() as out:
                nib = os.path.join(out, case + ".nib")
                status, log = compile_xib(["xcrun", "ibtool"], xib, nib)
                if status or "<key>com.apple.ibtool.errors</key>" in log:
                    sys.exit("%s: Xcode refused it:\n%s" % (case, log))
                runs.append(digest(nib))
        if runs[0] != runs[1] or runs[0] is None:
            sys.exit("%s: Xcode's two runs differ, or it wrote nothing" % case)
        with open(os.path.join(HERE, "expected", case + ".txt"), "w") as f:
            f.write(runs[0] + "\n")
        print(case)


if __name__ == "__main__":
    main()
