#!/bin/bash
# Wrapper around xcrunwrapper.sh to invoke libtool.
# This replaces the old @bazel_tools//tools/objc:libtool.sh which was
# removed in Bazel 9.

set -eu

MY_LOCATION="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Use xcrunwrapper to invoke llvm-libtool-darwin with proper Xcode env setup
exec "${MY_LOCATION}/xcrunwrapper.sh" llvm-libtool-darwin "$@"
