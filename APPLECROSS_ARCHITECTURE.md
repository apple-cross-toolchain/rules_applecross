# rules_applecross architecture overview

This document explains the current `rules_applecross` strategy for building
Apple targets from Linux execution platforms with Bazel, `rules_apple`,
`rules_swift`, and `apple_support`.

The short version: `rules_applecross` materializes an Xcode-shaped filesystem
inside the generated `@apple_cross_toolchain` repository, then replaces the
host-executable parts of Xcode with Linux-native Swift, LLVM, and ported Apple
tools. Bazel and the Apple rules still see an Xcode-like developer directory,
but actions execute on Linux.

## Design goal

Apple Bazel rules were written around Xcode assumptions:

- Apple SDKs live below `Xcode.app/Contents/Developer/Platforms`.
- compiler tools live below
  `Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr`.
- `DEVELOPER_DIR`, `SDKROOT`, `APPLE_SDK_PLATFORM`, and
  `APPLE_SDK_VERSION_OVERRIDE` describe the selected Xcode and SDK.
- `xcrun` resolves tools out of the selected developer directory.
- Swift finds resources, stdlibs, overlays, and target libraries relative to
  the toolchain root.

Rather than rewriting `rules_apple`, `rules_swift`, and `apple_support` around a
new Linux-specific layout, `rules_applecross` keeps the Xcode layout contract and
changes what is inside that layout.

## Generated repository layout

The module extension creates a repository named `@apple_cross_toolchain`. The
repository rule implementation is in `toolchain/apple_cross_toolchain.bzl`.

The generated repository contains a fake Xcode tree:

```text
@apple_cross_toolchain//
  BUILD
  cc_wrapper.sh
  xcrunwrapper.sh
  libtool.cc
  wrapped_clang.cc
  swift_version
  Xcode.app/Contents/Developer/
    version.plist
    Platforms/
      iPhoneOS.platform/
        Info.plist
        Developer/
          SDKs/iPhoneOS*.sdk/
          Library/Frameworks/
          usr/lib/
      iPhoneSimulator.platform/
      MacOSX.platform/
      ...
    Toolchains/
      XcodeDefault.xctoolchain/
        ToolchainInfo.plist
        usr/
          bin/
          include/
          lib/
            clang/
            swift/
```

For current local and remote builds the relevant developer directory is:

```text
external/+apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer
```

That path is exported through `.bazelrc` as `DEVELOPER_DIR`, and the fake
toolchain `usr/bin` is placed on `PATH`.

## Materialization inputs

The fake Xcode tree is assembled from several input sources.

### Apple SDK archive

Configured by the module extension:

```starlark
apple_cross_toolchain.configure(
    name = "apple_cross_toolchain",
    apple_sdk_path = "apple-sdks.tar.zst",
    apple_sdk_archive_type = "tar.zst",
)
```

The SDK archive is private Xcode-derived content. It provides the real Apple
target SDKs:

- platform directories, for example `iPhoneOS.platform`.
- SDK roots, for example `iPhoneOS.sdk`.
- framework headers and modulemaps.
- `.tbd` libraries.
- platform developer libraries and frameworks.
- Apple Swift target stdlibs and `.swiftinterface` files.
- `SDKSettings.json`, `SDKSettings.plist`, `Info.plist`, and version metadata.

The repository rule supports:

- `apple_sdk_path` pointing to a local tarball.
- `apple_sdk_path` pointing to a pre-extracted local directory.
- `apple_sdk_urls` plus optional sha, strip prefix, archive type, and netrc auth.

If the local path is a directory, the rule uses a hardlink copy with `cp -al`.
That keeps repository creation fast while preserving real paths well enough for
tools such as `xcrun`.

If the local path is an archive, the rule extracts it into the generated repo.

After extraction it:

- removes AppleDouble files and `__MACOSX` directories.
- normalizes conflicting SDK modulemaps, currently around duplicated libxml2
  modulemap locations.

### Swift Linux toolchain

Configured by `swift_path` or `swift_urls`.

In the local development wiring this currently points at Swift 6.2.1 for
Ubuntu 24.04. The repository rule also has defaults for a stripped Swift archive
from the `apple-cross-toolchain/ci` releases.

Swift binaries are copied from:

```text
tmp_swift/usr/bin/swift*
```

into:

```text
Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/
```

The Swift runtime and standard library directory is copied from:

```text
tmp_swift/usr/lib/swift
```

into:

```text
Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/
```

Then Linux-specific Swift overlay modules are removed:

```text
dispatch
CoreFoundation
Block
os
```

Those modules are provided by the Apple SDK when compiling for Apple targets.
Keeping the Linux overlays would make the compiler see the wrong module
definitions for an Apple target.

The rule also creates `lib/swift/linux` and `lib/swift/host` symlinks when
needed so the Linux Swift binaries can find their own host runtime libraries.

### Hermetic LLVM minimal prebuilt

The root module imports the LLVM minimal prebuilt repo from the `llvm` module
extension and aliases it as `@llvm_prebuilt`.

The repository rule resolves:

```starlark
Label("@llvm_prebuilt//:bin/clang")
```

and copies:

```text
@llvm_prebuilt/bin/* -> XcodeDefault.xctoolchain/usr/bin/
@llvm_prebuilt/lib/* -> XcodeDefault.xctoolchain/usr/lib/
```

LLVM is copied after the ported-tools archive so these LLVM binaries take
precedence.

The rule also creates Apple-compatible tool-name symlinks:

```text
install_name_tool -> llvm-install-name-tool
lipo              -> llvm-lipo
ar                -> llvm-ar
ranlib            -> llvm-ranlib
otool             -> llvm-otool
strip             -> llvm-strip
nm                -> llvm-nm
objdump           -> llvm-objdump
```

`libtool` is intentionally not symlinked directly to an LLVM binary. Apple static
archive creation goes through the custom `libtool` wrapper, which delegates to
`llvm-libtool-darwin` with Bazel-specific argument handling.

The rule also bridges clang resource-directory version mismatches. Xcode SDK
content may contain clang support files under one version directory, while the
Linux LLVM binary expects another. The rule symlinks the missing version or
missing resource subdirectories so clang finds SDK-provided compiler runtime
pieces.

### Ported Apple tools

The repository rule downloads:

```text
https://github.com/apple-cross-toolchain/ci/releases/download/0.0.22/ported-tools-linux-x86_64.tar.xz
```

and extracts it into:

```text
Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/
```

These tools fill gaps where Apple rules expect Xcode/macOS command-line tools.
The most important shape is that `xcrun` and related tools exist under the fake
Xcode toolchain and are Linux-executable.

Current host support is x86_64 Linux. The downloaded ported tools, selected LLVM
prebuilt, Swift archive, generated toolchains, and remote execution platform all
assume Linux x86_64.

### Generated helper tools

`wrapped_clang` and `libtool` are not opaque prebuilt binaries anymore. The
generated BUILD file declares them as Bazel `cc_binary` targets:

```starlark
cc_binary(name = "_wrapped_clang", srcs = ["wrapped_clang.cc"])
exec_tool(name = "wrapped_clang", binary = ":_wrapped_clang")
exec_tool(name = "wrapped_clang_pp", binary = ":_wrapped_clang")

cc_binary(name = "_libtool", srcs = ["libtool.cc"])
exec_tool(name = "libtool", binary = ":_libtool")
```

The repository rule symlinks the source files into `@apple_cross_toolchain`; the
helper binaries are then built in the Bazel graph for the execution platform.

`exec_tool` is used so the generated executable has the requested tool name.
That matters for `wrapped_clang`, which dispatches behavior based on `argv[0]`
between clang and clang++ modes.

## Compatibility stubs and fixes

The fake Xcode is not a complete Linux port of Xcode. Some tools are stubbed
because the corresponding Apple feature is either not available on Linux or not
yet implemented.

Current stubs include:

- `metal`
- `metallib`
- `intentbuilderc`
- `xcstringtool`
- `codesign`
- `codesign_allocate`
- `security`

The `security` stub handles the subset used by rules_apple codesigning paths,
including `security cms -D -i <mobileprovision>` and `security find-identity`.

The repository rule also synthesizes Swift compatibility archives when the SDK
or Swift distribution does not provide them:

```text
libswiftCompatibility51.a
libswiftCompatibility56.a
libswiftCompatibilityConcurrency.a
libswiftCompatibilityDynamicReplacements.a
libswiftCompatibilityPacks.a
```

These archives contain force-load symbols expected by Apple Swift link lines.
They are generated for selected Apple platforms using fake-Xcode clang and ar.

The rule also copies some `arm64e-apple-ios.swiftinterface` files to
`arm64-apple-ios.swiftinterface` with the target triple rewritten. This works
around SDK frameworks that only ship arm64e interfaces even though the build is
targeting arm64.

## Generated BUILD file

After materialization, the repository rule writes `BUILD` from
`toolchain/BUILD.template.bzl`.

The generated BUILD declares:

- filegroups for SDK/tool inputs.
- rule-based C/C++ Apple toolchains.
- Swift Apple toolchains.
- a Linux Swift toolchain rooted in the fake Xcode toolchain.
- helper binaries and wrapper tools.
- native Bazel `toolchain(...)` registrations for Apple target constraints.

Important filegroups:

```starlark
filegroup(name = "sdk_tool_files", ...)
filegroup(name = "toolchain_files", ...)
filegroup(name = "ported_tools", ...)
```

`sdk_tool_files` is intended for `rules_apple` actions that need Xcode tool and
SDK metadata inputs.

`toolchain_files` is much broader. It currently includes large portions of:

- SDK `usr`.
- SDK `System`.
- platform developer libraries/frameworks.
- fake Xcode toolchain `usr/bin`.
- fake Xcode toolchain `usr/include`.
- clang resources.
- Swift libraries and stdlibs.

This broad filegroup is functional but expensive for remote execution because it
can make individual actions carry very large input sets.

## C, C++, Objective-C, and linking

The generated C++ toolchain uses upstream `apple_support` rule-based toolchain
features and replaces only the tool map/environment needed for Applecross.

The generated BUILD loads upstream pieces from:

```starlark
@apple_support//configs:platforms.bzl
@apple_support//toolchain:...
@apple_support_toolchain_env//:...
@rules_cc//cc/toolchains:...
```

The main Applecross-specific override is the `cc_tool_map`:

```starlark
cc_tool_map(
    name = "applecross_tools",
    tools = {
        "ar actions": ":libtool_tool",
        "C compile actions": ":clang_tool",
        "C++ compile actions": ":clangpp_tool",
        "ObjC compile actions": ":clang_tool",
        "ObjC++ compile actions": ":clangpp_tool",
        "link actions": ":clang_tool",
        "strip": ":strip_tool",
        "coverage": ":llvm_cov_tool / :llvm_profdata_tool",
    },
)
```

The generated `rule_based_cc_toolchain` keeps the upstream Apple support feature
set, artifact patterns, module map, make variables, and dynamic toolchain info.
That is the compatibility goal: reuse upstream Apple toolchain semantics and
only swap the Linux-executable tool implementations.

The generated toolchain also injects:

```starlark
cc_args(
    name = "applecross_env",
    env = {
        "DEVELOPER_DIR": ".../Xcode.app/Contents/Developer",
        "LD_LIBRARY_PATH": ".../XcodeDefault.xctoolchain/usr/lib",
    },
)
```

### wrapped_clang

`wrapped_clang.cc` is the compile/link bridge.

For Linux execution it:

- finds `DEVELOPER_DIR`.
- computes `SDKROOT` from Apple platform environment.
- calls fake-Xcode `xcrun`.
- asks `xcrun` for `clang` or `clang++`.
- rewrites Bazel placeholders such as `__BAZEL_XCODE_DEVELOPER_DIR__`.
- handles SDK-root substitutions.
- keeps Apple support behavior such as dsym/strip/linkmap related processing.

So a Bazel ObjC compile action still looks like an Apple compile action, but the
actual process is a Linux binary invoking Linux clang against Apple SDK inputs.

### libtool

`libtool.cc` handles static archive creation.

It resolves:

```text
DEVELOPER_DIR/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-libtool-darwin
```

and delegates to it. The wrapper exists because Apple `libtool` behavior and
Bazel action arguments do not map perfectly to a direct LLVM symlink. It also
handles duplicate basenames through temporary symlinks and response files.

## Swift toolchain

The Apple Swift toolchain rule is implemented in `toolchain/swift_toolchain.bzl`.

For each Apple CPU/platform, the generated BUILD creates:

```starlark
swift_toolchain(
    name = "swift-compiler-" + arch,
    cpu = arch,
    toolchain_files = ":toolchain_files",
    toolchain_path_prefix = "...",
)
```

The Swift toolchain computes:

- platform type: iOS, macOS, tvOS, watchOS, visionOS.
- device vs simulator SDK platform.
- target triple, for example `arm64-apple-ios...`.
- SDK root below fake Xcode.
- Swift resource directory:

  ```text
  XcodeDefault.xctoolchain/usr/lib/swift
  ```

- Swift platform stdlib directory, for example:

  ```text
  XcodeDefault.xctoolchain/usr/lib/swift/iphoneos
  ```

It passes rules_swift action configs such as:

- `-target`.
- `-sdk`.
- `-resource-dir`.
- platform developer framework search paths.
- debug prefix map for Xcode path remapping.

It also adds Swift link options through `CcInfo`, including:

```text
-L.../usr/lib/swift/<swift-platform>
-L.../Platforms/<platform>.platform/Developer/usr/lib
-F.../Platforms/<platform>.platform/Developer/Library/Frameworks
```

Currently `toolchain_files` is passed as `additional_tools` for Swift actions.
That makes SwiftCompile actions functional in sandboxes and remote execution,
but it is a major performance cost because the action input set becomes very
large.

## rules_apple integration

`rules_apple` is still used for Apple application and bundle graph expansion.
The strategy is to make `rules_apple` find Linux-compatible tools through the
fake Xcode tree rather than host macOS.

The root `.bazelrc` wires:

```text
--xcode_version_config=@rules_applecross//xcode_config:host_xcodes
--action_env=DEVELOPER_DIR=external/+apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer
--action_env=PATH=external/+apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin:/bin:/usr/bin:/usr/local/bin
--@build_bazel_rules_apple//apple:sdk_tool_files=@apple_cross_toolchain//:sdk_tool_files
```

The `rules_apple` patches in the module remove or reduce assumptions that are
invalid on Linux, such as:

- Darwin-only execution constraints for some Apple actions.
- hardcoded absolute host tool paths.
- direct `/usr/bin/lipo`.
- bundle tool path assumptions.

When `rules_apple` asks for Xcode tools, those tools now come from declared
inputs under `@apple_cross_toolchain`.

## xcode_config

The project provides an Xcode version config target under `xcode_config`.

This lets Bazel's Apple configuration believe there is a selected Xcode version
and SDK version even though the host machine is Linux. The repository rule reads
Xcode and SDK version metadata from the materialized fake Xcode tree and injects
that data into generated BUILD substitutions.

The Swift toolchain also reads `apple_common.XcodeVersionConfig` to derive:

- selected Xcode version.
- SDK version.
- minimum OS for the selected Apple platform type.
- execution info.

## Remote execution flow

For an iOS arm64 Swift application build on BuildBuddy RBE:

1. Bazel selects the iOS arm64 target platform.
2. Bazel selects Linux x86_64 execution toolchains from `@apple_cross_toolchain`.
3. `rules_apple` expands bundle/application actions.
4. `rules_swift` emits Swift actions using the Applecross Swift toolchain.
5. C, ObjC, C++, ObjC++, archive, and link actions use the Applecross
   rule-based C++ toolchain.
6. Remote Linux workers receive declared inputs from `@apple_cross_toolchain`,
   the source tree, and transitive dependencies.
7. `swiftc`, `clang`, `llvm-libtool-darwin`, `llvm-lipo`, and ported `xcrun`
   run on Linux.
8. Those tools target Apple triples and Apple SDK roots.
9. Outputs are normal Apple build artifacts: objects, Swift modules, archives,
   linked binaries, bundle files, and application bundles where the corresponding
   `rules_apple` action is supported or stubbed.

## What is real and what is fake

Real:

- Apple target SDK contents from the private Xcode-derived archive.
- Apple Swift target stdlibs and interfaces from the SDK archive.
- Linux Swift compiler and driver.
- Linux LLVM and Mach-O tools.
- upstream `apple_support` rule-based feature semantics.
- upstream `rules_swift` action configuration machinery.
- upstream `rules_apple` graph/rule expansion, with patches.

Fake or synthetic:

- `Xcode.app` as a Linux repository layout.
- selected Xcode metadata.
- some Apple tool implementations.
- codesigning behavior.
- Metal compilation behavior.
- selected Swift compatibility archives.
- selected arm64 Swift interfaces derived from arm64e interfaces.

## Why this enables full Apple graphs on Linux

The important part is that the Apple graph remains an Apple graph.

`rules_apple` still owns application and bundle semantics. `rules_swift` still
owns Swift compile semantics. `apple_support` still owns Apple C++ feature
semantics. `rules_applecross` supplies a Linux-executable Xcode-shaped toolchain
that satisfies their assumptions.

That gives a path to building whole Apple applications remotely:

- keep target semantics identical to Apple Bazel builds.
- make host tools Linux-executable.
- make every Xcode path a declared Bazel input.
- patch only places where upstream rules assume host macOS instead of selected
  Xcode/toolchain inputs.

## Current limitations

The current implementation is functional but not yet production-performance
quality.

Known architectural limits:

- execution platform is currently Linux x86_64.
- ported tools are downloaded as `ported-tools-linux-x86_64`.
- LLVM minimal prebuilt selection currently uses a Linux amd64 prebuilt.
- Swift archive selection currently uses a Linux x86_64 Swift distribution.
- generated Apple C++ and Swift toolchains declare Linux x86_64 exec
  compatibility.
- `toolchain_files` is too broad and is attached to too many actions.
- Swift actions receive the broad `toolchain_files` set as `additional_tools`.
- some `rules_apple` actions are stubbed rather than truly implemented,
  especially around asset catalogs, Metal, codesigning, and other Xcode-only
  tools.
- private SDK archives cannot be committed or uploaded to public cache storage.

The main performance problem is not the fake-Xcode idea itself. The problem is
the current granularity of declared inputs. Too many actions depend on too much
of the generated Xcode tree.

Production-level work should split the generated filegroups by action and
platform:

- minimal clang tool inputs.
- minimal Swift compiler host inputs.
- selected target SDK headers/modules/frameworks.
- selected Swift target stdlibs.
- archive/linker tool inputs.
- bundle/plist tool inputs.
- signing/provisioning tool inputs.
- platform-specific SDK metadata.

That keeps the same compatibility model while reducing analysis time, action
input discovery, CAS uploads, and remote scheduling overhead.
