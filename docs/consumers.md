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
    xcode_version = "27.2",
    # A packaged SDK archive: under a third of the .xip's size, and nothing to unpack.
    apple_sdk_urls = ["https://storage.example.com/apple-sdks-27.2.tar.gz"],
    # Or Apple's .xip itself, used when no SDK archive is configured.
    xcode_xip_urls = ["file:///mnt/xcode/Xcode_27.2.xip"],
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
`bazel run //tools:package_sdks -- Xcode_27.2.xip apple-sdks-27.2.tar.gz` makes
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
    xcode_version = "27.2",
    # A placeholder: .bazelrc.user says where the archive is, with --distdir
    # or --downloader_config (docs/consumers.md in rules_applecross).
    apple_sdk_urls = ["https://apple-sdks.invalid/apple-sdks-27.2.tar.gz"],
    ...
)
```

Either of Bazel's own options then supplies it:

- `--distdir=<directory>` for a copy on disk. Bazel looks there first for a
  file named as the URL ends, `apple-sdks-27.2.tar.gz`, and checks it against
  the pinned SHA-256.
- `--downloader_config=<file>` for a copy on a server, with a line such as

  ```
  rewrite apple-sdks\.invalid/.* http://sdks.internal:8080/apple-sdks-27.2.tar.gz
  ```

  The pattern is matched against the whole URL without its scheme. Bazel
  keeps the original scheme (`https`) unless the replacement names one, so
  write `http://` out for a plain HTTP server. Bazel's downloader sends the
  rewritten host's credentials itself, from the Bazel client's `.netrc`
  (`~/.netrc`, or the file `NETRC` names in the client's environment) or a
  `--credential_helper`; `--repo_env=NETRC` doesn't reach it. The archive's
  type comes from the URL in `MODULE.bazel`, so a rewrite to a URL without a
  suffix still extracts.

Without either, the fetch fails looking up `apple-sdks.invalid`. Once
fetched, the archive is in the repository cache by its SHA-256 and neither is
consulted again.

### Ported tools newer than their release

The toolchain downloads its ported Apple tools (the clean-room actool, ibtool
and codesign among them) from this repository's release, as
`toolchain/ported_tools.sha256` lists them. Tools built from a later commit,
such as the `ported-tools` workflow's artifacts on `main`, can stand in until
the next release, the same way as the SDK copy:

```starlark
apple_cross_toolchain.configure(
    ...
    # "<sha256>  <tool>-linux-<cpu>.tar.zst" lines, as in ported_tools.sha256.
    ported_tools_manifest = "//third_party:ported_tools.sha256",
    ported_tools_urls = ["https://ported-tools.invalid/sha256/{sha256}"],
)
```

With `{sha256}` in the URL, `--distdir` takes a directory of the archives named
by their SHA-256, and a content-addressed server takes one `rewrite` line.


## Pinning rules_applecross through a registry

rules_applecross's rules_apple patch (patches/rules_apple_execution_platform.patch)
makes rules_apple itself depend on `rules_applecross` 0.0.3, the version in
this repository's `MODULE.bazel`, for its bundling toolchain type. A
`git_override` or `local_path_override` answers that request. A registry
version of any other number doesn't: module discovery then looks for 0.0.3 in
the registries and fails. Pin one version for every request:

```starlark
bazel_dep(name = "rules_applecross", version = "0.0.3.cc07dde")
single_version_override(module_name = "rules_applecross", version = "0.0.3.cc07dde")
```

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
// swift-compiler-version: Apple Swift version 6.4.2 (swiftlang-6.4.2.1.9 clang-2100.4.1.1)
```

Xcode 27.2's SDKs were built by Swift 6.4.2, Xcode 27.0's by Swift 6.4.0
(toolchain/xcode_versions.bzl lists each release's), and `xcode_version`
uses that Swift unless `swift_tools` names another. With your own, set
`swift_version` on `apple_cross_toolchain.swift()` to match; a mismatch
fails as `this SDK is not supported by the compiler`.


## Signing with an identity on Linux

The clean-room codesign signs ad hoc, or with an identity given as two PEM
files: its private key (RSA) and its certificate with the chain above it.
Apple Development, Apple Distribution and Developer ID identities get the
team identifier and designated requirement codesign gives them;
docs/clean-room/codesign.md has the details. Export the identity once, on
the Mac that holds it (Keychain Access › Export, as a `.p12`), then:

```sh
openssl pkcs12 -in identity.p12 -nocerts -nodes -out identity.key
openssl pkcs12 -in identity.p12 -nokeys -out identity.pem   # leaf, and add
# Apple's intermediate (AppleWWDRCAG3.cer or DeveloperIDG2CA.cer) and the
# Apple Root CA, as PEM, to identity.pem
```

and in the builder's untracked `.bazelrc.user`:

```
common --repo_env=RULES_APPLECROSS_CODESIGN_KEY=/abs/path/identity.key
common --repo_env=RULES_APPLECROSS_CODESIGN_CERTIFICATES=/abs/path/identity.pem
```

rules_apple then finds the identity as on a Mac: by name
(`--@build_bazel_rules_apple//apple/build_settings:signing_certificate_name`,
or `--ios_signing_cert_name` for iOS) or by a provisioning profile's
certificate. The files are referenced where they are, never copied into a
repository, so the key is no action input and reaches no cache; signing
therefore runs on the machine holding the key, not on a remote executor.
Point the paths elsewhere to change identity: Bazel reruns signing when
they change, but not when the files at the same paths do.

Developer ID signatures get a secure timestamp from Apple, as codesign gives
them, unless `--timestamp=none` is among the bundle's codesign options;
others get one with `--timestamp`. Asking for it needs the network from the
signing action, and makes the signature differ from build to build.

A Developer ID-signed Mac app can be notarized off a Mac:
`bazel run @rules_applecross//tools:notarize -- path/to/App.app` (or a
`.zip`, `.dmg` or `.pkg`) submits it as `xcrun notarytool submit --wait`
does, through Apple's notary web API. It authenticates with an App Store
Connect API key, given as `--key AuthKey.p8 --key-id ID --issuer ISSUER` or
by `APP_STORE_CONNECT_KEY_PATH`, `APP_STORE_CONNECT_KEY_ID` and
`APP_STORE_CONNECT_ISSUER_ID`.

A notarized Mac app's ticket can be stapled off a Mac too:
`bazel run @rules_applecross//tools:staple -- path/to/App.app` fetches it
from Apple's public lookup by the app's code directory hash and writes it to
`Contents/CodeResources`, as `xcrun stapler staple` does.

## The Linux host

The toolchain fetches LLVM, swift.org's Swift and the ported tools itself.
The host needs what swift.org's Swift needs, which on Ubuntu 24.04 (the
release its builds target) is: binutils, libc6-dev, libcurl4-openssl-dev,
libedit2, libgcc-13-dev, libncurses-dev, libpython3-dev, libsqlite3-0,
libstdc++-13-dev, libxml2-dev, libz3-dev, pkg-config, tzdata, unzip,
zlib1g-dev. Newer releases may lack libraries it links (Ubuntu 26.04 has no
`libxml2.so.2`).

rules_applecross depends on the `llvm` module, whose `MODULE.bazel` asks for
rules_go 0.62.0 without depending on it. A graph without rules_go is
unaffected. One that has rules_go anyway (flatbuffers, googleapis, envoy and
others bring it) is raised to rules_go 0.62 and, through it, gazelle 0.51, whose
repository tooling needs Go 1.24.12 or later: register a Go SDK at least that
new in the root module (`go_sdk.download(version = "1.24.12")`).

On a Linux host Apple targets build for Apple silicon unless a `--*_cpus`
flag says otherwise: macOS for arm64, and iOS, tvOS and watchOS for their arm64
simulators. rules_apple alone would choose x86_64 there, which a Mac without
Rosetta refuses to open and its simulators refuse to install, so
rules_applecross patches the default (patches/rules_apple_arm64_default.patch).

### Swift for Linux itself

Swift that runs on the Linux host (macro plugins, code generators and other
tools the build runs) compiles with the Swift toolchain
`@apple_cross_toolchain//:all` registers for Linux: rules_swift's own, from the
`swift_tools` repository. The project's C++ toolchain for Linux links it.
Swift's runtime needs libstdc++, libcurl and libxml2, which a hermetic toolchain
without a sysroot (the `llvm` module's) can't find. Such a project can register
rules_swift's C++ toolchain for Linux, which has Ubuntu's libraries as its
sysroot, ahead of its own:

```starlark
register_toolchains("@apple_cross_toolchain//linux:cc_toolchain")
```

It then compiles and links all C and C++ for Linux too. Its sysroot is
rules_swift's, which fetches Ubuntu's packages, and their index, from
snapshot.ubuntu.com alone.

## Swift toolchain settings

rules_swift's own toolchains take flags, features and package
configurations as attributes of the toolchain target. Here the toolchains
are generated, so `apple_cross_toolchain.configure()` takes them and gives
them to every Swift toolchain it registers:

```starlark
apple_cross_toolchain.configure(
    name = "apple_cross_toolchain",
    # ...
    swift_copts = ["-DMY_FLAG"],
    swift_default_enabled_features = ["swift.emit_symbol_graph"],
    swift_default_unsupported_features = [],
    swift_feature_allowlists = ["//build:swift_allowlist"],
    swift_generated_header_rewriter = "//build:rewrite_generated_header",
    swift_package_configurations = ["//build:strict_packages"],
)
```

`swift_copts` come before `--@rules_swift//swift:copt`'s. The
`swift_feature_allowlist` and `swift_package_configuration` targets must be
visible to the toolchain's repository (`//visibility:public`).

`swift_generated_header_rewriter` is rules_swift's Xcode toolchain's
`generated_header_rewriter`: an executable that rewrites the Objective-C header
a Swift compile generates (`<module>-Swift.h`), run after that compile with the
compile's command line after a `--` argument. Setting it enables
`swift.rewrite_generated_header`, the feature under which rules_swift runs it,
for every Swift target (a target's `features` can still turn it off).
rules_swift gives the rewriter no runfiles, so it must run as it is: a script
or a self-contained binary. Like the targets above, it must be visible to the
toolchain's repository.

### Explicit modules

`swift.use_c_modules` with `swift.emit_c_module` turns implicit modules off.
The compiler then finds a module only when it is given one, the SDK's included,
and SwiftShims too. rules_swift's Xcode toolchain gets the SDK's modules from
its `system_sdk` extension, which reads the Mac's Xcode. Here the toolchain
describes them from the staged SDKs itself when asked:

```starlark
apple_cross_toolchain.configure(
    name = "apple_cross_toolchain",
    # ...
    swift_system_modules = True,
    # Optional, by SDK, besides the modules rules_swift leaves out:
    swift_system_modules_exclude = {"iPhoneOS": ["SomeModule"]},
)
```

The first build that needs them scans the SDKs into a repository
`<name>_system_sdk`, with rules_swift's own scanner run by swift.org's swiftc.
It has the same targets as rules_swift's `@system_sdk` (`all_modules`,
`implicit_modules`, `all_cross_import_overlays`, one per module), and the
toolchains take them as rules_swift's Xcode toolchain takes those. Every Swift
module is compiled from its interface, never from Xcode's prebuilt modules,
and every Clang module is precompiled for the SDK's version, as Xcode does.
Targets that name a module themselves (`deps = ["@apple_cross_toolchain_system_sdk//:UIKit"]`)
need `use_repo(apple_cross_toolchain, "apple_cross_toolchain_system_sdk")`.
Turn both features on where every C or Objective-C dependency of the Swift
code builds too (`--features` or a package's `features`): a dependency's module
is only precompiled where `swift.emit_c_module` is on. examples/ios/ExplicitModules
builds that way.

rules_swift asks its worker for `-hermetic-pcm` on every C module precompile.
That needs a Swift whose resource directory is inside `DEVELOPER_DIR`, which
swift.org's never is beside a staged Xcode, so these toolchains leave it out.
The precompiles need no rewriting: every path they take is already the staged
tree's, relative to the execution root.

## Layering check

`--features=layering_check` (or a package's `features`) works as on a Mac:
a C, C++ or Objective-C file may include only its target's direct
dependencies' headers and the SDK's. apple_support generates the SDK's
`crosstool` module on a Mac; here the toolchain writes it when it is
fetched (`layering_check.modulemap`, every header of the staged developer
directory), and the clang wrapper reads it in place.

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
build, an Icon Composer app icon among them. Each near match can be accepted
on its own (`--config=relax_actool_icon_composer` and the others in the
README's table), or all of them with `--config=relaxed`.
