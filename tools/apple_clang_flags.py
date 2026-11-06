#!/usr/bin/env python3
"""apple_clang_flags.py LLVM_CLANG: the driver flags Xcode's clang accepts that
LLVM_CLANG rejects, as toolchain/wrapped_clang.cc's kAppleOnlyFlags lists them.

Run on a Mac with the Xcode to compare (xcrun clang). Each flag in Xcode's
clang's --help-hidden that LLVM's doesn't list is tried on LLVM_CLANG; those
it rejects as unknown are printed with how they take a value: none, joined
(after "="), or as the next arguments (how many)."""

import re
import subprocess
import sys


def options(clang):
    out = subprocess.run(clang + ["--help-hidden"], capture_output=True, text=True).stdout
    names = {}
    for line in out.splitlines():
        m = re.match(r"^  (-[^ ,<=]+=?)((?: <[^>]+>)*)", line)
        if m and not m.group(1).endswith("="):
            # How many values follow as separate arguments (the most any of
            # its spellings take).
            names[m.group(1)] = max(names.get(m.group(1), 0), m.group(2).count("<"))
        elif m:
            names.setdefault(m.group(1), 0)
    return names


def main():
    llvm = [sys.argv[1]]
    apple, theirs = options(["xcrun", "clang"]), options(llvm)
    for name in sorted(set(apple) - set(theirs)):
        values = apple[name]
        args = [name] + ["x"] * values if not name.endswith("=") else [name + "x"]
        result = subprocess.run(llvm + args + ["-x", "c", "-c", "/dev/null", "-o", "/dev/null"], capture_output=True, text=True)
        if "unknown argument" in result.stderr:
            kind = "kJoined" if name.endswith("=") else "kSeparate" if values else "kFlag"
            print('    {"%s", %s, %d},' % (name, kind, values))


if __name__ == "__main__":
    main()
