#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARCH="$(uname -m)"
DEST="$(mkdir -p "${1:-$ROOT/ported-tools}" && cd "${1:-$ROOT/ported-tools}" && pwd)"
WORK="$(mktemp -d)"
OUT="$WORK/out"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$OUT/bin"
cd "$WORK"

curl -fsSL -o pngcrush.tar.gz https://sourceforge.net/projects/pmt/files/pngcrush/1.8.13/pngcrush-1.8.13.tar.gz/download
echo "bac37d4b2be88d7e88aadcde9661beb3a513a90e7d26784f906c1e2da8ba332e  pngcrush.tar.gz" | sha256sum -c --quiet
tar -xf pngcrush.tar.gz
make -C pngcrush-1.8.13 -j"$(nproc)"
install -m 755 pngcrush-1.8.13/pngcrush "$OUT/bin/pngcrush"

cd "$ROOT/tools/apple_tools"
bazel build -c opt //:tools
bazel cquery -c opt --output=files //:tools 2>/dev/null | while read -r tool; do
  install -m 755 "$tool" "$OUT/bin/"
done
swift build -c release --static-swift-stdlib --package-path swiftui_macros --scratch-path "$WORK/swiftui_macros"
install -m 755 "$WORK/swiftui_macros/release/SwiftUIMacros" "$OUT/bin/SwiftUIMacros"

for tool in "$OUT"/bin/*; do
  name="$(basename "$tool")"
  tar -C "$OUT" --sort=name --owner=0 --group=0 --numeric-owner --mtime=@1672560000 -cf - "bin/$name" |
    zstd -q -f -19 -o "$DEST/$name-linux-$ARCH.tar.zst"
done
