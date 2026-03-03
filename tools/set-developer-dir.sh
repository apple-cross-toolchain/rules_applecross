#!/bin/bash
#
# Point xcode-select at the toolchain's developer directory so that ported
# tools (xcrun, xcode-select, etc.) can resolve it automatically.

set -euo pipefail

bazel fetch @apple_cross_toolchain//:all
EXTERNAL="$(bazel info output_base)/external"
DEVELOPER_DIR="$(find "$EXTERNAL" -maxdepth 2 -type d -name Xcode.app -path "*apple_cross_toolchain*" 2>/dev/null | head -1)/Contents/Developer"

if [[ ! -d "$DEVELOPER_DIR" ]]; then
  echo "ERROR: DEVELOPER_DIR not found under $EXTERNAL" >&2
  echo "Make sure 'bazel fetch @apple_cross_toolchain//:all' succeeded." >&2
  exit 1
fi

sudo xcode-select -s "$DEVELOPER_DIR"
echo "Developer directory set to: $DEVELOPER_DIR"
