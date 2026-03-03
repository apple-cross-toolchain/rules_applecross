#!/bin/bash
#
# Install xcrun and xcode-select to the system PATH so that Bazel actions
# (which run with a restricted PATH) can find them. Other ported tools are
# resolved automatically via xcode-select at runtime.

set -euxo pipefail

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/install_mandatory_tools_work_dir.XXXXXX")"
trap 'rm -rf "${TMP_DIR}"' ERR EXIT

pushd "${TMP_DIR}"
curl -# -LO https://github.com/apple-cross-toolchain/ci/releases/download/0.0.22/ported-tools-linux-x86_64.tar.xz
tar -xf ported-tools-linux-x86_64.tar.xz
install bin/xcrun bin/xcode-select /usr/bin/
popd
