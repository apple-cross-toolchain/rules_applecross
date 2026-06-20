#!/usr/bin/env python3
"""Bridge SDK clang resource libraries into the runnable clang resource dir."""

import os
import re
import subprocess
import sys


def _first_sdk_clang_version(clang_lib_dir):
    if not os.path.isdir(clang_lib_dir):
        return None
    for name in sorted(os.listdir(clang_lib_dir)):
        path = os.path.join(clang_lib_dir, name)
        if os.path.isdir(os.path.join(path, "lib", "darwin")):
            return name
    return None


def _llvm_clang_major(toolchain_bindir):
    clang = os.path.join(toolchain_bindir, "clang")
    try:
        result = subprocess.run([clang, "--version"], check=False, capture_output=True, text=True)
    except OSError:
        return None
    match = re.search(r"clang version\s+([0-9]+)", result.stdout)
    return match.group(1) if match else None


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: ensure_clang_resource_libs.py <clang-lib-dir> <toolchain-bin-dir>", file=sys.stderr)
        return 2

    clang_lib_dir, toolchain_bindir = sys.argv[1:]
    sdk_clang_ver = _first_sdk_clang_version(clang_lib_dir)
    llvm_ver = _llvm_clang_major(toolchain_bindir)
    if not sdk_clang_ver or not llvm_ver or llvm_ver == sdk_clang_ver:
        return 0

    llvm_clang_dir = os.path.join(clang_lib_dir, llvm_ver)
    sdk_clang_dir = os.path.join(clang_lib_dir, sdk_clang_ver)
    if not os.path.isdir(llvm_clang_dir):
        os.symlink(sdk_clang_ver, llvm_clang_dir)
        return 0

    sdk_lib_dir = os.path.join(sdk_clang_dir, "lib")
    llvm_lib_dir = os.path.join(llvm_clang_dir, "lib")
    if os.path.isdir(sdk_lib_dir) and not os.path.exists(llvm_lib_dir):
        os.symlink(os.path.join("..", sdk_clang_ver, "lib"), llvm_lib_dir)

    return 0


if __name__ == "__main__":
    sys.exit(main())
