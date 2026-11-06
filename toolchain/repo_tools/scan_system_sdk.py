#!/usr/bin/env python3
"""Run rules_swift's explicit module scanner on a staged Xcode tree.

    scan_system_sdk.py <scan.py> <scan.py's arguments>...

The scanner expects every path the compiler reports to lie inside the
developer directory it was given. A staged tree's parts may be links into the
Xcode it came from (a Mac's own, or one mounted on Linux), which the compiler
reports resolved: such a path is taken as the staged tree's when the tree has
the same file below its Contents/Developer. Clang's own headers are named as
Swift reaches them, through usr/lib/swift/clang: the toolchain's Swift actions
have that link, not clang's resource directory.
"""

import importlib.util
import os
import re
import sys

_DEVELOPER = "/Contents/Developer/"

_CLANG_RESOURCES = re.compile(
    r"^(Toolchains/XcodeDefault\.xctoolchain/usr/lib/)clang/[^/]+/"
)


def main() -> None:
    scan_path = sys.argv[1]
    spec = importlib.util.spec_from_file_location("scan", scan_path)
    scan = importlib.util.module_from_spec(spec)
    # Its dataclasses look their module up while it loads.
    sys.modules["scan"] = scan
    spec.loader.exec_module(scan)

    replace_path_prefix = scan._replace_path_prefix

    def staged_path_prefix(path: str, *, old: str, new: str) -> str:
        if path and not path.startswith(old + "/"):
            index = path.rfind(_DEVELOPER)
            if index >= 0:
                relative = path[index + len(_DEVELOPER) :]
                if os.path.exists(os.path.join(old, relative)):
                    path = old + "/" + relative
        if path.startswith(old + "/"):
            relative = _CLANG_RESOURCES.sub(
                r"\1swift/clang/", path[len(old) + 1 :]
            )
            if os.path.exists(os.path.join(old, relative)):
                path = old + "/" + relative
        return replace_path_prefix(path, old=old, new=new)

    scan._replace_path_prefix = staged_path_prefix
    sys.argv = [scan_path] + sys.argv[2:]
    scan._main()


if __name__ == "__main__":
    main()
