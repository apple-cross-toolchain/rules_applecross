#!/bin/bash
# Builds an app on this Mac (Xcode's actool, ibtool, codesign) and in the Linux
# VM (the clean-room ones), and compares the .ipa files byte for byte.
#
#   tools/compare_ipa.sh [//examples/ios/CleanRoom:CleanRoom]
set -euo pipefail

TARGET="${1:-//examples/ios/CleanRoom:CleanRoom}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/.compare"
FLAGS=(
  --ios_multi_cpus=arm64
  --ios_signing_cert_name=-
  -c opt
)
package="${TARGET#//}"; package="${package%%:*}"; name="${TARGET##*:}"
rel="bazel-bin/$package/$name.ipa"
mkdir -p "$OUT"
cd "$ROOT"

echo "== macOS (Xcode's Apple tools)" >&2
bazel build --symlink_prefix=/ --repo_env=BAZEL_NO_APPLE_CPP_TOOLCHAIN=0 "${FLAGS[@]}" "$TARGET"
mac_bin="$(bazel cquery --symlink_prefix=/ --repo_env=BAZEL_NO_APPLE_CPP_TOOLCHAIN=0 "${FLAGS[@]}" --output=files "$TARGET" 2>/dev/null | grep '\.ipa$')"
cp -f "$(bazel info --symlink_prefix=/ execution_root)/$mac_bin" "$OUT/mac.ipa"

echo "== Linux (clean-room Apple tools)" >&2
tools/linux_vm/run.sh bash -c "bazel build ${FLAGS[*]} $TARGET && cp -f $rel .compare/linux.ipa"

echo >&2
shasum -a 256 "$OUT/mac.ipa" "$OUT/linux.ipa"
if cmp -s "$OUT/mac.ipa" "$OUT/linux.ipa"; then
  echo "identical"
else
  echo "DIFFERENT" >&2
  rm -rf "$OUT/mac" "$OUT/linux"
  mkdir -p "$OUT/mac" "$OUT/linux"
  (cd "$OUT/mac" && unzip -q ../mac.ipa)
  (cd "$OUT/linux" && unzip -q ../linux.ipa)
  diff -r "$OUT/mac" "$OUT/linux" >&2 || true
  exit 1
fi
