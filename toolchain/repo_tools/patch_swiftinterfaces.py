#!/usr/bin/env python3
"""Create arm64 swiftinterfaces from arm64e-only framework modules."""

import os
import sys


def _patch_tree(framework_root):
    if not os.path.isdir(framework_root):
        return

    for dirpath, dirnames, _ in os.walk(framework_root, followlinks=False):
        if not dirpath.endswith(".swiftmodule"):
            continue
        dirnames[:] = []

        arm64e = os.path.join(dirpath, "arm64e-apple-ios.swiftinterface")
        arm64 = os.path.join(dirpath, "arm64-apple-ios.swiftinterface")
        if not os.path.exists(arm64e) or os.path.exists(arm64):
            continue

        with open(arm64e, "r") as src:
            content = src.read()
        with open(arm64, "w") as dst:
            dst.write(content.replace("arm64e-apple-ios", "arm64-apple-ios"))


def main() -> int:
    for framework_root in sys.argv[1:]:
        _patch_tree(framework_root)
    return 0


if __name__ == "__main__":
    sys.exit(main())
