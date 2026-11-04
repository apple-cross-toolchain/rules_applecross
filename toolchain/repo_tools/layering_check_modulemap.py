#!/usr/bin/env python3
"""Writes the layering check's `crosstool` module map, as apple_support's
generate-modulemap.sh writes it from an Xcode: every header (`*.h`, `*.def`,
anything under a `c++` directory) of a developer directory, and of any other
directories given (a macOS host's own LLVM, whose clang headers its
compiles use), as a textual header, sorted bytewise. Its paths are relative
to the map's own directory (OUT's), which is where clang resolves them.

    layering_check_modulemap.py OUT DIR...
"""

import os
import sys


def main(out, *roots):
    headers = set()
    for root in roots:
        if os.path.isdir(root):
            add(headers, root, out)
    with open(out, "w") as f:
        f.write('module "crosstool" [system] {\n')
        for header in sorted(headers, key=lambda h: h.encode()):
            f.write('  textual header "{}"\n'.format(header))
        f.write("}\n")


def add(headers, developer_dir, out):
    base = os.path.relpath(developer_dir, os.path.dirname(os.path.abspath(out)))
    for directory, _, files in os.walk(developer_dir, followlinks=True):
        rel = os.path.relpath(directory, developer_dir)
        in_cxx = "c++" in rel.split(os.sep)
        for name in files:
            if name.endswith(".h") or name.endswith(".def") or in_cxx:
                path = os.path.join(directory, name)
                if os.path.isfile(path):
                    headers.add(os.path.normpath(os.path.join(base, rel, name)))


if __name__ == "__main__":
    main(*sys.argv[1:])
