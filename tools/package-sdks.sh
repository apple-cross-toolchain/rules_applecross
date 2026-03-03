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
  # Scripting frameworks never used by the toolchain
  --exclude "Ruby.framework"
  --exclude "Perl.framework"
  --exclude "Python3.framework"
  --exclude "Python.framework"
)

## Copy SDKs
for sdk in MacOSX iPhoneOS iPhoneSimulator WatchOS WatchSimulator AppleTVOS AppleTVSimulator XROS XRSimulator; do
  # xcrun relies on this Info.plist to find and invoke tools
  rsync -a --relative "$DEVELOPER_DIR/./Platforms/$sdk.platform/Info.plist" "$NEW_DEVELOPER_DIR"

  rsync -a --relative "${SDK_EXCLUDES[@]}" "$DEVELOPER_DIR/./Platforms/$sdk.platform/Developer/SDKs/" "$NEW_DEVELOPER_DIR"

  if [[ -d "$DEVELOPER_DIR/Platforms/$sdk.platform/usr/lib" ]]; then
    rsync -a --relative "$DEVELOPER_DIR/./Platforms/$sdk.platform/usr/lib/" "$NEW_DEVELOPER_DIR"
  fi

  if [[ -d "$DEVELOPER_DIR/Platforms/$sdk.platform/Developer/usr/lib" ]]; then
    rsync -a --relative "$DEVELOPER_DIR/./Platforms/$sdk.platform/Developer/usr/lib/" "$NEW_DEVELOPER_DIR"
  fi

  if [[ -d "$DEVELOPER_DIR/Platforms/$sdk.platform/Developer/Library/Frameworks" ]]; then
    rsync -a --relative "${SDK_EXCLUDES[@]}" "$DEVELOPER_DIR/./Platforms/$sdk.platform/Developer/Library/Frameworks/" "$NEW_DEVELOPER_DIR"
  fi

  # PrivateFrameworks contains XCTestCore.framework, XCTAutomationSupport.framework,
  # etc. that XCTest.framework re-exports. Without these the linker fails with
  # "unable to locate re-export".
  if [[ -d "$DEVELOPER_DIR/Platforms/$sdk.platform/Developer/Library/PrivateFrameworks" ]]; then
    rsync -a --relative "${SDK_EXCLUDES[@]}" "$DEVELOPER_DIR/./Platforms/$sdk.platform/Developer/Library/PrivateFrameworks/" "$NEW_DEVELOPER_DIR"
  fi
done

# Copy toolchain libraries
rsync -a --relative "$DEVELOPER_DIR/./Toolchains/XcodeDefault.xctoolchain/ToolchainInfo.plist" "$NEW_DEVELOPER_DIR"
rsync -a --relative "$DEVELOPER_DIR/./Toolchains/XcodeDefault.xctoolchain/usr/include/" "$NEW_DEVELOPER_DIR"
if [[ -d "$DEVELOPER_DIR/Toolchains/XcodeDefault.xctoolchain/usr/lib/arc" ]]; then
  rsync -a --relative "$DEVELOPER_DIR/./Toolchains/XcodeDefault.xctoolchain/usr/lib/arc/" "$NEW_DEVELOPER_DIR"
fi
rsync -a --relative "$DEVELOPER_DIR/./Toolchains/XcodeDefault.xctoolchain/usr/lib/clang/" "$NEW_DEVELOPER_DIR"
rsync -a --relative "$DEVELOPER_DIR/./Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/" "$NEW_DEVELOPER_DIR"
if [[ -d "$DEVELOPER_DIR/./Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.0" ]]; then
  rsync -a --relative "$DEVELOPER_DIR/./Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.0/" "$NEW_DEVELOPER_DIR"
fi

# Create a placeholder bin directory
mkdir -p "$NEW_DEVELOPER_DIR/Toolchains/XcodeDefault.xctoolchain/usr/bin"

# Remove self-referencing symlinks (e.g. Ruby.framework/Headers/ruby/ruby -> .)
# that cause infinite loops when Bazel globs the SDK tree.
find "$PROJECT_ROOT/Xcode.app" -type l -exec sh -c 'test "$(readlink "$1")" = "." && rm "$1"' _ {} \;

XCODE_VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$PROJECT_ROOT/Xcode.app/Contents/version.plist")"

tar -Jcf "apple-sdks-xcode-$XCODE_VERSION.tar.xz" Xcode.app
