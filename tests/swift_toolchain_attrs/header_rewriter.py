#!/usr/bin/env python3
"""A generated_header_rewriter for the tests: appends MARKER to the header the
compile after `--` generates (its -emit-objc-header-path), reading response
files as the compiler would."""

import shlex
import sys

MARKER = "// Rewritten by //tests/swift_toolchain_attrs:header_rewriter.\n"


def compile_args(argv):
    args = argv[argv.index("--") + 1:]
    expanded = []
    for arg in args:
        if arg.startswith("@"):
            with open(arg[1:]) as f:
                expanded.extend(shlex.split(f.read()))
        else:
            expanded.append(arg)
    return expanded


def main():
    args = compile_args(sys.argv)
    if "-emit-objc-header-path" not in args:
        return 0
    header = args[args.index("-emit-objc-header-path") + 1]
    with open(header, "a") as f:
        f.write(MARKER)
    return 0


if __name__ == "__main__":
    sys.exit(main())
