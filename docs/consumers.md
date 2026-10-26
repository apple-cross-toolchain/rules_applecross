# Building a project with rules_applecross

What a project needs beyond the README's setup to build its Apple apps on
Linux, as found building Visor (an iOS and macOS app with Swift packages).

## The Apple SDKs by Xcode version

`xcode_version` names a release in
[toolchain/xcode_versions.bzl](../toolchain/xcode_versions.bzl), which pins
the SHA-256 of Apple's `.xip` and of the SDK archive made from it. Either can
then come from wherever the project keeps a copy:

```starlark
apple_cross_toolchain.configure(
    name = "apple_cross_toolchain",
    xcode_version = "27.0",
    # A packaged SDK archive: under a third of the .xip's size, and nothing to unpack.
    apple_sdk_urls = ["https://storage.example.com/apple-sdks-27.0.tar.gz"],
    # Or Apple's .xip itself, used when no SDK archive is configured.
    xcode_xip_urls = ["file:///mnt/xcode/Xcode_27.xip"],
    ...
)
```

The URLs are tried in order, so a mirror can come before a local copy, and
`file://` URLs name a copy on disk. Bazel checks every download against the
pinned SHA-256 and keeps it in its repository cache, so a second clone or
`bazel clean --expunge` reuses it. Hosts that need credentials take them the
way any Bazel download does: `.netrc`, or `--credential_helper`.

From a `.xip`, the rule extracts the SDKs, stubs their Mach-O binaries with
this repository's LLVM and discards the rest, which takes a minute or two.
`bazel run //tools:package_sdks -- Xcode_27.xip apple-sdks-27.0.tar.gz` makes
the SDK archive from the same `.xip`. Its output is reproducible, so it
matches the pinned SHA-256 whoever runs it. Both give the same tree.

Whatever the source, a pinned `xcode_version` fails the build when the staged
SDKs are from another Xcode release, a local Xcode included:
`--repo_env=RULES_APPLECROSS_LOCAL_XCODE=0` makes a Mac with another Xcode
use the pinned one.

### Where the copy is, kept out of the repository

The SDKs are Apple's to distribute, so the copy belongs on a host you
control, and a public repository shouldn't name anyone's own machines. Name a placeholder in
`MODULE.bazel` and say where the copy really is in each builder's untracked
`.bazelrc` (one `try-import`ed, say `.bazelrc.user`):

```starlark
apple_cross_toolchain.configure(
    name = "apple_cross_toolchain",
    xcode_version = "27.0",
    # A placeholder: .bazelrc.user says where the archive is, with --distdir
    # or --downloader_config (docs/consumers.md in rules_applecross).
    apple_sdk_urls = ["https://apple-sdks.invalid/apple-sdks-27.0.tar.gz"],
    ...
)
```

Either of Bazel's own options then supplies it:

- `--distdir=<directory>` for a copy on disk. Bazel looks there first for a
  file named as the URL ends, `apple-sdks-27.0.tar.gz`, and checks it against
  the pinned SHA-256.
- `--downloader_config=<file>` for a copy on a server, with a line such as

  ```
  rewrite apple-sdks\.invalid/.* http://sdks.internal:8080/apple-sdks-27.0.tar.gz
  ```

  The pattern is matched against the whole URL without its scheme. Bazel
  keeps the original scheme (`https`) unless the replacement names one, so
  write `http://` out for a plain HTTP server. Credentials come from
  `.netrc` or `--credential_helper` for the rewritten host.

Without either, the fetch fails looking up `apple-sdks.invalid`. Once
fetched, the archive is in the repository cache by its SHA-256 and neither is
consulted again.

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
