#!/usr/bin/env python3
"""Drop architectures LLVM's linker cannot read from the SDKs' text stubs.

Xcode 27's SDK stubs list "arm64e.x1" targets beside "arm64e", which ld64.lld
(LLVM 22) rejects as an unknown architecture, refusing the whole stub. The
rest of each stub is unchanged; arm64 links resolve against the arm64e
entries as before.

Rewritten stubs replace their path rather than being edited in place: the
tree may hard-link a packaged archive's files or symlink an installed Xcode.

Usage: normalize_sdk_tbds.py <Xcode.app>
"""

import os
import re
import sys

_UNKNOWN = ("arm64e.x1",)
_TARGET = re.compile(r",\s*(?:%s)-[A-Za-z0-9_-]+" % "|".join(re.escape(a) for a in _UNKNOWN))
_TARGET_FIRST = re.compile(r"(\[\s*)(?:%s)-[A-Za-z0-9_-]+\s*,\s*" % "|".join(re.escape(a) for a in _UNKNOWN))


def rewrite(text):
    text = _TARGET.sub("", text)
    return _TARGET_FIRST.sub(r"\1", text)


def main():
    root = os.path.join(sys.argv[1], "Contents", "Developer", "Platforms")
    changed = 0
    for platform in sorted(os.listdir(root)):
        sdks = os.path.join(root, platform, "Developer", "SDKs")
        if not os.path.isdir(sdks):
            continue
        for dirpath, dirnames, filenames in os.walk(sdks, followlinks=False):
            dirnames[:] = [d for d in dirnames if not os.path.islink(os.path.join(dirpath, d))]
            for name in filenames:
                if not name.endswith(".tbd"):
                    continue
                path = os.path.join(dirpath, name)
                if os.path.islink(path) and not os.path.exists(path):
                    continue
                with open(path, encoding="utf-8") as f:
                    text = f.read()
                if not any(a in text for a in _UNKNOWN):
                    continue
                new = rewrite(text)
                tmp = path + ".tmp"
                with open(tmp, "w", encoding="utf-8") as f:
                    f.write(new)
                os.replace(tmp, path)
                changed += 1
    print("rewrote {} stubs".format(changed), file=sys.stderr)


if __name__ == "__main__":
    main()
