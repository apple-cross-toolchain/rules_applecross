#!/bin/bash
#
# Apple rules invoke some tools with the absolute paths, so we need to
# provision our machine with those tools first.

set -euxo pipefail

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/install_mandatory_tools_work_dir.XXXXXX")"
trap 'rm -rf "${TMP_DIR}"' ERR EXIT

pushd "${TMP_DIR}"
curl -# -LO https://github.com/apple-cross-toolchain/ci/releases/download/0.0.4/ported-tools-linux-x86_64.tar.xz
tar -xf ported-tools-linux-x86_64.tar.xz
install bin/codesign bin/lipo bin/plutil bin/sw_vers bin/xcrun bin/xcodebuild bin/xcode-select /usr/bin/
mkdir -p /usr/libexec
install bin/PlistBuddy /usr/libexec/
popd
