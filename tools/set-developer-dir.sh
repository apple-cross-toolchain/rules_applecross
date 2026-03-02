#!/bin/bash
#
# Point xcode-select at the toolchain's developer directory so that ported
# tools (xcrun, xcode-select, etc.) can resolve it automatically.

set -euo pipefail

bazel fetch @apple_cross_toolchain//:all
DEVELOPER_DIR="$(bazel info output_base)/external/+apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer"

if [[ ! -d "$DEVELOPER_DIR" ]]; then
  echo "ERROR: DEVELOPER_DIR not found at $DEVELOPER_DIR" >&2
  echo "Run 'bazel fetch //tests/data:dummy_lib' first." >&2
  exit 1
fi

sudo xcode-select -s "$DEVELOPER_DIR"
echo "Developer directory set to: $DEVELOPER_DIR"
