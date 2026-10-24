# Building a project with rules_applecross

What a project needs beyond the README's setup to build its Apple apps on
Linux, as found building Visor (an iOS and macOS app with Swift packages).

## The Apple SDKs from a mounted Xcode

Instead of an SDK archive, the toolchain can read a Mac's Xcode mounted
read-only into the Linux host or container:

```starlark
apple_cross_toolchain.configure(
    name = "apple_cross_toolchain",
    local_xcode = True,
    ...
)
```

On Linux it reads `/Applications/Xcode.app/Contents/Developer` (or
`DEVELOPER_DIR`) when its `Platforms` directory exists, and otherwise
`apple_sdk_path` or `apple_sdk_urls`. `--repo_env=RULES_APPLECROSS_LOCAL_XCODE=1`
makes a missing mount an error rather than a fall back to the archive. The
tree is linked into the repository, not written to.

swift.org's Swift must be the one that built that Xcode's SDKs. The second
line of any of their `.swiftinterface` files says which, for example
`Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/Foundation.framework/Modules/Foundation.swiftmodule/arm64e-apple-ios.swiftinterface`:

```
// swift-compiler-version: Apple Swift version 6.4 effective-5.10 (swiftlang-6.4.0.31.4 clang-2100.3.31.1)
```

Xcode 27.0's SDKs were built by Swift 6.4.0, which this repository's
`MODULE.bazel` sets. For another Xcode, set `swift_version` on
`apple_cross_toolchain.swift()` to match; a mismatch fails as `this SDK is
not supported by the compiler`.

## The Linux host

The toolchain fetches LLVM, swift.org's Swift and the ported tools itself.
The host needs what swift.org's Swift needs, which on Ubuntu 24.04 (the
release its builds target) is: binutils, libc6-dev, libcurl4-openssl-dev,
libedit2, libgcc-13-dev, libncurses-dev, libpython3-dev, libsqlite3-0,
libstdc++-13-dev, libxml2-dev, libz3-dev, pkg-config, tzdata, unzip,
zlib1g-dev. Newer releases may lack libraries it links (Ubuntu 26.04 has no
`libxml2.so.2`).

On a Linux host rules_apple builds macOS targets for x86_64 unless told
otherwise; pass `--macos_cpus=arm64` for Apple silicon.

## Swift packages

rules_swift_package_manager reads package manifests by running
`swift package dump-package` on the host, so the host needs swift.org's
Swift on `PATH`, ideally the same version as the toolchain's.

Manifests are evaluated on the host, as SwiftPM always does, so a
manifest's `#if canImport(Darwin)` is false on Linux. swift-nio and
swift-crypto use it to add their privacy manifests, which are then missing
from the app. Take the Darwin branch, as a Mac would:

```starlark
swift_deps.configure_package(
    name = "swift-nio",
    patch_cmds = ["sed -i.orig 's/^#if canImport(Darwin)$/#if true/' Package.swift"],
)
```

## Output

The clean-room tools are strict by default (README, "Byte-identical
output"): an input whose output can't be guaranteed to be Xcode's fails the
build, an Icon Composer app icon among them. `--config=relaxed` accepts a
near match.
