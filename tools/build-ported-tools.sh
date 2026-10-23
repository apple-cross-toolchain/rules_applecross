#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARCH="$(uname -m)"
DEST="$(mkdir -p "${1:-$ROOT/ported-tools}" && cd "${1:-$ROOT/ported-tools}" && pwd)"
WORK="$(mktemp -d)"
OUT="$WORK/out"
MUSL=(-c opt "--platforms=@llvm//platforms:linux_${ARCH}_musl")
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$OUT/bin" "$OUT/lib"
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

cd "$ROOT/tools/system_tools"
bazel build "${MUSL[@]}" //:tools
bazel cquery "${MUSL[@]}" --output=files //:tools 2>/dev/null | while read -r path; do
  case "$path" in
    */bin/python3.*) install -m 755 -s "$path" "$OUT/bin/python3" ;;
    */lib/python3.*) cp -R "$path" "$OUT/lib/" ;;
    *) install -m 755 -s "$path" "$OUT/bin/" ;;
  esac
done
chmod -R u+w "$OUT/lib"

for tool in zip unzip python3; do
  file "$OUT/bin/$tool" | grep -q -e 'statically linked' -e 'static-pie linked'
done
"$OUT/bin/zip" -qX "$WORK/check.zip" "$OUT/bin/zip"
"$OUT/bin/unzip" -tqq "$WORK/check.zip"
"$OUT/bin/python3" -c 'import bz2, lzma, plistlib, sys, zipfile, zlib; assert zipfile.ZipFile(sys.argv[1]).testzip() is None' "$WORK/check.zip"

for tool in "$OUT"/bin/*; do
  name="$(basename "$tool")"
  paths=("bin/$name")
  if [ "$name" = python3 ]; then
    paths+=(lib)
  fi
  tar -C "$OUT" --sort=name --owner=0 --group=0 --numeric-owner --mtime=@1672560000 -cf - "${paths[@]}" |
    zstd -q -f -19 -o "$DEST/$name-linux-$ARCH.tar.zst"
done
