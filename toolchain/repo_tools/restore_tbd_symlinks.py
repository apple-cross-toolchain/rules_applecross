#!/usr/bin/env python3
"""Restore framework binary symlinks that point to .tbd stubs."""

import os
import sys


def _is_framework_path(path):
    return any(part.endswith(".framework") for part in path.split(os.sep))


def _remove_if_broken_symlink(path):
    if not os.path.islink(path):
        return False
    if os.path.exists(path):
        return True
    try:
        os.unlink(path)
    except FileNotFoundError:
        pass
    return True


def main() -> int:
    root = sys.argv[1] if len(sys.argv) > 1 else "Xcode.app"

    for dirpath, dirnames, filenames in os.walk(root, topdown=True, followlinks=False):
        kept_dirs = []
        for name in dirnames:
            path = os.path.join(dirpath, name)
            if not _remove_if_broken_symlink(path):
                kept_dirs.append(name)
        dirnames[:] = kept_dirs

        in_framework = _is_framework_path(dirpath)
        for name in filenames:
            path = os.path.join(dirpath, name)
            if _remove_if_broken_symlink(path):
                continue

            if in_framework and name.endswith(".tbd"):
                link = os.path.join(dirpath, name[:-4])
                if not os.path.exists(link):
                    try:
                        os.unlink(link)
                    except FileNotFoundError:
                        pass
                    os.symlink(name, link)

    return 0


if __name__ == "__main__":
    sys.exit(main())
