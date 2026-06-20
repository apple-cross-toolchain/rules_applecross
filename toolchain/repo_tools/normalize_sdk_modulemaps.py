#!/usr/bin/env python3
"""Normalize SDK module maps after extraction."""

import os
import sys


def main() -> int:
    root = sys.argv[1] if len(sys.argv) > 1 else "Xcode.app"
    suffix = os.path.join("usr", "include", "libxml2", "module.modulemap")

    for dirpath, _, filenames in os.walk(root, followlinks=False):
        if "module.modulemap" not in filenames:
            continue
        modulemap = os.path.join(dirpath, "module.modulemap")
        if not modulemap.endswith(suffix):
            continue
        include_dir = modulemap[: -len(os.path.join("libxml2", "module.modulemap"))]
        duplicate = os.path.join(include_dir, "libxml", "module.modulemap")
        if os.path.exists(duplicate):
            os.unlink(modulemap)
    return 0


if __name__ == "__main__":
    sys.exit(main())
