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

cd "$ROOT/tools/apple_tools"
bazel build -c opt //:tools
bazel cquery -c opt --output=files //:tools 2>/dev/null | while read -r tool; do
  install -m 755 "$tool" "$OUT/bin/"
done

cd "$ROOT/tools/apple_tools/swiftui_macros"
for plugin in SwiftUIMacros AppIntentsMacros PreviewsMacros; do
  bazel build -c opt "//:$plugin"
  install -m 755 "$(bazel cquery -c opt --output=files "//:$plugin" 2>/dev/null)" "$OUT/bin/"
done

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
