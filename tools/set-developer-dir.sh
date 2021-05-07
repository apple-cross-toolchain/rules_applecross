#!/bin/bash
#
# This is a hack -- many Apple rules don't include tools in their action
# inputs, but rely on xcrun to invoke tools.

set -euo pipefail

bazel fetch //tests/data:dummy_lib
DEVELOPER_DIR="$(bazel info output_base)/external/apple_cross_toolchain/Xcode.app/Contents/Developer"
sudo xcode-select -s "$DEVELOPER_DIR"
