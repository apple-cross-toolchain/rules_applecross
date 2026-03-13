#!/bin/bash

set -euxo pipefail

if [[ -z "${DEVELOPER_DIR:-}" ]]; then
  DEVELOPER_DIR="$(xcode-select -p)"
fi

SCRIPT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_ROOT/.." && pwd)"
NEW_DEVELOPER_DIR="$PROJECT_ROOT/Xcode.app/Contents/Developer"
mkdir -p "$NEW_DEVELOPER_DIR"

cp -a "$DEVELOPER_DIR/../Info.plist" "$NEW_DEVELOPER_DIR/.."
cp -a "$DEVELOPER_DIR/../version.plist" "$NEW_DEVELOPER_DIR/.."

## Excludes applied to all SDK rsync operations.
## These remove content that is not needed for cross-compilation linking.
SDK_EXCLUDES=(
  # Documentation and resource data
  --exclude "usr/share"
  --exclude "*.lproj"
  # Code signatures (not verified during cross-compilation)
  --exclude "_CodeSignature"
  # Swift documentation blobs (compiler needs .swiftmodule/.swiftinterface only)
  --exclude "*.swiftdoc"
  # Static archives — the SDK provides .tbd text stubs for linking
  --exclude "*.a"
  # Mach-O dynamic libraries — only .tbd text stubs are needed for linking
  # on Linux; the real binaries are large and architecture-specific
  --exclude "*.dylib"
  # Scripting frameworks never used by the toolchain
  --exclude "Ruby.framework"
  --exclude "Perl.framework"
  --exclude "Python3.framework"
  --exclude "Python.framework"
)

# Change into DEVELOPER_DIR so that rsync --relative uses short relative
# paths ("./Platforms/...") instead of absolute ones.  macOS openrsync
# ignores the /path/./ marker inside absolute paths, which caused the
# entire source prefix to be recreated at the destination.
cd "$DEVELOPER_DIR"

## Copy SDKs
for sdk in MacOSX iPhoneOS iPhoneSimulator WatchOS WatchSimulator AppleTVOS AppleTVSimulator XROS XRSimulator; do
  # xcrun relies on this Info.plist to find and invoke tools
  rsync -a --relative "./Platforms/$sdk.platform/Info.plist" "$NEW_DEVELOPER_DIR"

  rsync -a --relative "${SDK_EXCLUDES[@]}" "./Platforms/$sdk.platform/Developer/SDKs/" "$NEW_DEVELOPER_DIR"

  if [[ -d "Platforms/$sdk.platform/usr/lib" ]]; then
    rsync -a --relative --exclude "*.a" --exclude "*.dylib" "./Platforms/$sdk.platform/usr/lib/" "$NEW_DEVELOPER_DIR"
  fi

  if [[ -d "Platforms/$sdk.platform/Developer/usr/lib" ]]; then
    rsync -a --relative --exclude "*.a" --exclude "*.dylib" "./Platforms/$sdk.platform/Developer/usr/lib/" "$NEW_DEVELOPER_DIR"
  fi

  if [[ -d "Platforms/$sdk.platform/Developer/Library/Frameworks" ]]; then
    rsync -a --relative "${SDK_EXCLUDES[@]}" "./Platforms/$sdk.platform/Developer/Library/Frameworks/" "$NEW_DEVELOPER_DIR"
  fi

  # PrivateFrameworks contains XCTestCore.framework, XCTAutomationSupport.framework,
  # etc. that XCTest.framework re-exports. Without these the linker fails with
  # "unable to locate re-export".
  if [[ -d "Platforms/$sdk.platform/Developer/Library/PrivateFrameworks" ]]; then
    rsync -a --relative "${SDK_EXCLUDES[@]}" "./Platforms/$sdk.platform/Developer/Library/PrivateFrameworks/" "$NEW_DEVELOPER_DIR"
  fi
done

# Copy toolchain libraries
rsync -a --relative "./Toolchains/XcodeDefault.xctoolchain/ToolchainInfo.plist" "$NEW_DEVELOPER_DIR"
rsync -a --relative "./Toolchains/XcodeDefault.xctoolchain/usr/include/" "$NEW_DEVELOPER_DIR"
if [[ -d "Toolchains/XcodeDefault.xctoolchain/usr/lib/arc" ]]; then
  rsync -a --relative "./Toolchains/XcodeDefault.xctoolchain/usr/lib/arc/" "$NEW_DEVELOPER_DIR"
fi
rsync -a --relative "./Toolchains/XcodeDefault.xctoolchain/usr/lib/clang/" "$NEW_DEVELOPER_DIR"
# Exclude .a and .dylib from Swift runtime dirs — the cross-compilation
# toolchain uses its own Swift runtime; only .tbd stubs, .swiftmodule/
# and .swiftinterface files are needed for compilation and linking.
rsync -a --relative --exclude "*.a" --exclude "*.dylib" "./Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/" "$NEW_DEVELOPER_DIR"
if [[ -d "./Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.0" ]]; then
  rsync -a --relative --exclude "*.a" --exclude "*.dylib" "./Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.0/" "$NEW_DEVELOPER_DIR"
fi

# Create a placeholder bin directory
mkdir -p "$NEW_DEVELOPER_DIR/Toolchains/XcodeDefault.xctoolchain/usr/bin"

# Replace Mach-O binaries with TBD text stubs.
# Framework bundles contain fat Mach-O binaries (e.g. XCTest.framework/XCTest)
# that are only needed at runtime.  For cross-compilation linking we only need
# .tbd stubs.  Use `tapi stubify` to generate them, then remove the originals.
find "$PROJECT_ROOT/Xcode.app" -path "*.framework/*" -type f ! -name "*.*" \
  -exec sh -c '
    if file "$1" | grep -q "Mach-O"; then
      tbd="${1}.tbd"
      if [ ! -f "$tbd" ]; then
        xcrun tapi stubify "$1" -o "$tbd" 2>/dev/null || true
      fi
      rm "$1"
    fi
  ' _ {} \;

# Generate TBD stubs for .dylib files that were excluded by rsync.
# Re-scan the source and stubify any dylib whose .tbd doesn't already exist.
for sdk in MacOSX iPhoneOS iPhoneSimulator WatchOS WatchSimulator AppleTVOS AppleTVSimulator XROS XRSimulator; do
  for dir in \
    "Platforms/$sdk.platform/usr/lib" \
    "Platforms/$sdk.platform/Developer/usr/lib" \
    "Toolchains/XcodeDefault.xctoolchain/usr/lib/swift" \
    "Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.0"
  do
    src_dir="$DEVELOPER_DIR/$dir"
    dst_dir="$NEW_DEVELOPER_DIR/$dir"
    [[ -d "$src_dir" ]] || continue
    [[ -d "$dst_dir" ]] || continue
    find "$src_dir" -name "*.dylib" -type f | while read -r dylib; do
      rel="${dylib#"$src_dir/"}"
      tbd_path="$dst_dir/${rel%.dylib}.tbd"
      [[ -f "$tbd_path" ]] && continue
      mkdir -p "$(dirname "$tbd_path")"
      xcrun tapi stubify "$dylib" -o "$tbd_path" 2>/dev/null || true
    done
  done
done

# Strip reexported_libraries from Developer framework TBDs.
# tapi stubify preserves re-export chains (e.g. XCTest → XCTestCore) but ld64.lld
# cannot resolve @rpath references to PrivateFrameworks.  For cross-compilation
# we only need the stub to exist; the re-export metadata is not required.
find "$PROJECT_ROOT/Xcode.app" -path "*/Developer/Library/*/Frameworks/*.framework/*.tbd" -type f \
  -exec python3 -c '
import json, sys
for path in sys.argv[1:]:
    try:
        with open(path) as f:
            tbd = json.load(f)
    except (json.JSONDecodeError, ValueError):
        continue
    changed = False
    for key in ("main_library", "libraries"):
        obj = tbd.get(key)
        items = [obj] if isinstance(obj, dict) else (obj if isinstance(obj, list) else [])
        for item in items:
            if "reexported_libraries" in item:
                del item["reexported_libraries"]
                changed = True
    if changed:
        with open(path, "w") as f:
            json.dump(tbd, f)
' {} +

# Remove self-referencing symlinks (e.g. Ruby.framework/Headers/ruby/ruby -> .)
# that cause infinite loops when Bazel globs the SDK tree.
find "$PROJECT_ROOT/Xcode.app" -type l -exec sh -c 'test "$(readlink "$1")" = "." && rm "$1"' _ {} \;

XCODE_VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$PROJECT_ROOT/Xcode.app/Contents/version.plist")"

tar -C "$PROJECT_ROOT" --zstd -cf "$PROJECT_ROOT/apple-sdks-xcode-$XCODE_VERSION.tar.zst" Xcode.app
