#!/usr/bin/env python3
"""Writes expected/<case>/ from Xcode's own xcstringstool (on a Mac with the
Xcode the tests are against): what `xcstringstool compile <case>.xcstrings
--output-directory <dir>` writes, each empty directory kept with a `.empty`
file, or expected/<case>.fails for a catalog Xcode refuses. Run from
anywhere:

    python3 tests/xcstringstool/update_expected.py
"""

import os
import shutil
import subprocess
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))


def main():
    expected_root = os.path.join(HERE, "expected")
    for case in sorted(f[:-len(".xcstrings")] for f in os.listdir(os.path.join(HERE, "cases"))):
        expected = os.path.join(expected_root, case)
        shutil.rmtree(expected, ignore_errors=True)
        fails = expected + ".fails"
        if os.path.exists(fails):
            os.remove(fails)
        with tempfile.TemporaryDirectory() as tmp:
            out = os.path.join(tmp, "out")
            result = subprocess.run(["xcrun", "xcstringstool", "compile", os.path.join(HERE, "cases", case + ".xcstrings"),
                                     "--output-directory", out], capture_output=True, text=True)
            if result.returncode != 0:
                open(fails, "w").close()
                print(case, "fails")
                continue
            shutil.copytree(out, expected) if os.path.isdir(out) else os.makedirs(expected)
        for directory, dirs, files in os.walk(expected):
            if not dirs and not files and directory != expected:
                open(os.path.join(directory, ".empty"), "w").close()
        print(case)


if __name__ == "__main__":
    main()
