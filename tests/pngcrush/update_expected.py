#!/usr/bin/env python3
"""Writes expected/<case>.txt from Xcode's own pngcrush (on a Mac with Xcode
27): the digest of the PNG it writes for each case, run twice."""

import os
import subprocess
import sys
import tempfile

from common import HERE, cases, crush, digest


def main():
    tool = [subprocess.run(["xcrun", "-f", "pngcrush"], capture_output=True, text=True, check=True).stdout.strip()]
    os.makedirs(os.path.join(HERE, "expected"), exist_ok=True)
    for case, png in cases():
        runs = []
        for _ in range(2):
            with tempfile.TemporaryDirectory() as out:
                path = os.path.join(out, case + ".png")
                status, log = crush(tool, png, path)
                if status:
                    sys.exit("%s: Xcode's pngcrush failed:\n%s" % (case, log))
                runs.append(digest(path))
        if runs[0] != runs[1]:
            sys.exit("%s: Xcode's two runs differ" % case)
        with open(os.path.join(HERE, "expected", case + ".txt"), "w") as f:
            f.write(runs[0] + "\n")
        print(case)


if __name__ == "__main__":
    main()
