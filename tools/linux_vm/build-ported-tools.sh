#!/bin/bash
# apple-cross-toolchain/ci's ported tools for this host's CPU plus the
# clean-room tools, as ported-tools-linux-<arch>.tar.xz in the workspace root.
# --apple-tools-only updates the existing archive.
#
#   tools/linux_vm/run.sh tools/linux_vm/build-ported-tools.sh
set -euo pipefail

CI_RELEASE=0.0.23
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ARCH="$(uname -m)"
WORK="$(mktemp -d)"
OUT="$WORK/out"
trap 'rm -rf "$WORK"' EXIT
archive="$ROOT/ported-tools-linux-$ARCH.tar.xz"
mkdir -p "$OUT"

if [ "${1:-}" != --apple-tools-only ]; then
  url="https://github.com/apple-cross-toolchain/ci/releases/download/$CI_RELEASE/ported-tools-linux-$ARCH.tar.xz"
  curl -fsSL -o "$WORK/ported.tar.xz" "$url"
  curl -fsSL "$url.sha256" | awk '{print $1}' > "$WORK/expected"
  echo "$(cat "$WORK/expected")  $WORK/ported.tar.xz" | sha256sum -c --quiet
  cp "$WORK/ported.tar.xz" "$archive"
fi
tar -C "$OUT" -xf "$archive"

(cd "$ROOT/tools/apple_tools" && CARGO_TARGET_DIR="$WORK/apple_tools" cargo build -q --release --locked)
for tool in actool codesign ibtool; do
  install -m 755 "$WORK/apple_tools/release/$tool" "$OUT/bin/$tool"
done
# The marker tells the toolchain repository to keep this codesign.
git -C "$ROOT" rev-parse HEAD > "$OUT/bin/.apple-tools-revision" 2>/dev/null || echo unknown > "$OUT/bin/.apple-tools-revision"

tar -C "$OUT" -Jcf "$archive" bin lib
sha256sum "$archive"
