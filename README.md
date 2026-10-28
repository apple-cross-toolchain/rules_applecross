# Bazel Apple toolchains for Linux

This repository contains toolchains for Bazel that can be used to build apps
and frameworks for Apple platforms from Linux.

Supports:

- Bazel 8.8+
- Linux x86_64 and arm64
- macOS x86_64 and arm64

## What it does, and what it doesn't

`bazel build` of rules_apple's targets on Linux gives what it gives on a Mac
with Xcode: apps, extensions, frameworks, and the bundles of unit and UI
tests, signed ad hoc or with a signing identity (Apple Development, Apple
Distribution or Developer ID; see [docs/consumers.md](docs/consumers.md)).
Mac apps can also be notarized and stapled from Linux (`//tools:notarize`,
`//tools:staple`).

It builds; it doesn't run. Apps and test bundles built on Linux run on a
Mac, a simulator or a device, as those built with Xcode do; `bazel test` of
an Apple test target off a Mac builds the bundles but has nothing to run
them on.

Resources are compiled by clean-room tools whose output is byte-identical to
Xcode's or fails the build ([Byte-identical output](#byte-identical-output)).
Code is compiled by swift.org's Swift and linked by LLVM's linker, not
Apple's forks of them: it runs as Xcode's does, but its bytes differ.

Off a Mac, these have no Linux version yet, and building a target that needs
one fails:

- Metal shaders (`metal`, `metallib`);
- App Shortcuts Flexible Matching (`app_intents_flexible_matching`, on by
  default; set it to `False`). App Intents' intents, parameters, enums,
  entities, queries and App Shortcuts compile as Xcode compiles them
  ([docs/clean-room/appintentsmetadataprocessor.md](docs/clean-room/appintentsmetadataprocessor.md));
- Core Data models (`momc`, `mapc`);
- Core ML models other than ML Programs: Create ML's text classifiers, word
  taggers, image classifiers and tabular models, and older neural networks.
  ML Programs (`.mlpackage`) compile, and get their generated classes, as
  Xcode makes them ([docs/clean-room/coremlc.md](docs/clean-room/coremlc.md));
- Reality Composer content (`realitytool`), DocC, and `.xib` files;
- tvOS app icons and Icon Composer icons
  ([docs/clean-room](docs/clean-room/README.md)).

`--config=relaxed` lets Metal through as an empty Metal library. String
Catalogs compile as Xcode compiles them; the few it can't yet reproduce, it
refuses unless relaxed ([docs/clean-room/xcstringstool.md](docs/clean-room/xcstringstool.md)).

## Setup

1. Add the dependency to your `MODULE.bazel`:

    ```starlark
    bazel_dep(name = "rules_applecross", version = "0.0.3")
    git_override(
        module_name = "rules_applecross",
        remote = "https://github.com/apple-cross-toolchain/rules_applecross.git",
        commit = "<commit>",
    )
    ```

    Bazel applies `single_version_override` only in the root module, so also
    copy this repository's `patches/` and its `rules_apple` and `apple_support`
    overrides from `MODULE.bazel`.

2. Configure the toolchain in `MODULE.bazel`:

    ```starlark
    apple_cross_toolchain = use_extension(
        "@rules_applecross//toolchain:extensions.bzl",
        "apple_cross_toolchain",
    )
    apple_cross_toolchain.configure(
        name = "apple_cross_toolchain",
        apple_sdk_urls = ["<url-to-apple-sdk-archive>"],
        apple_sdk_archive_type = "tar.xz",  # if not inferrable from URL
        swift_tools = {
            "aarch64": "@applecross_swift_ubuntu24.04-aarch64//:tools",
            "x86_64": "@applecross_swift_ubuntu24.04//:tools",
        },
    )
    use_repo(apple_cross_toolchain, "apple_cross_toolchain")

    register_toolchains("@apple_cross_toolchain//:cc-toolchain-ios_x86_64")
    register_toolchains("@apple_cross_toolchain//:cc-toolchain-ios_arm64")
    register_toolchains("@apple_cross_toolchain//:swift-toolchain-ios_x86_64")
    register_toolchains("@apple_cross_toolchain//:swift-toolchain-ios_arm64")
    # ... add more platforms as needed (darwin, tvos, watchos)

    # Where Apple products may be bundled: on Linux with the clean-room
    # tools, and on a Mac with its Xcode. Without the macOS one, a build on
    # a Mac fails with "No matching toolchains found for
    # ...apple_bundling_toolchain_type".
    register_toolchains(
        "@rules_applecross//toolchain:apple_bundling_on_linux",
        "@rules_applecross//toolchain:apple_bundling_on_macos",
    )
    ```

3. Add the following to your `.bazelrc`:

    ```
    startup --host_jvm_args=-DBAZEL_TRACK_SOURCE_DIRECTORIES=1
    common --incompatible_repo_env_ignores_action_env
    build --xcode_version_config=@apple_cross_toolchain//:host_xcodes
    build --action_env=DEVELOPER_DIR=external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer
    build --action_env=PATH=external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin:/bin:/usr/bin:/usr/local/bin
    build --@build_bazel_rules_apple//apple:sdk_tool_files=@apple_cross_toolchain//:sdk_tool_files
    ```

    [docs/consumers.md](docs/consumers.md) covers the rest of a project's
    setup: SDKs from a mounted Xcode, the Linux host, Swift packages.

4. Build an example:

    ```
    bazel build @rules_applecross//examples/ios/HelloWorldSwiftUI:HelloWorld
    ```

## Packaging the SDK

**[Please ensure you have read and understood the Xcode license terms before continuing.](https://www.apple.com/legal/sla/docs/xcode.pdf)**

If you build on macOS, Xcode will be automatically detected and used. On
Linux, `xcode_version` with `xcode_xip_urls` or `apple_sdk_urls` stages a
pinned release from a `.xip` or an SDK archive you host
([docs/consumers.md](docs/consumers.md#the-apple-sdks-by-xcode-version)).
This makes that archive from a `.xip`, on any host:

```
bazel run //tools:package_sdks -- ~/Downloads/Xcode_27.xip apple-sdks-27.0.tar.gz
```

On a Mac, the following packages the selected Xcode instead:

```
tools/package-sdks.sh
```

## Building on macOS

```starlark
apple_cross_toolchain.configure(
    name = "apple_cross_toolchain",
    apple_sdk_urls = ["<url-to-apple-sdk-archive>"],
    local_xcode = True,  # this skips staging the SDKs
    swift_tools = {"x86_64": "..."},
)
```

The Swift toolchain must match the Swift that built those SDKs, which is the
Swift of the Xcode *one release behind* the SDKs' own: Xcode 26.6 ships the 26.5
SDKs, and those were built by Swift 6.3.2. A mismatch shows up as `this SDK is
not supported by the compiler` when compiling any Swift that imports a system
framework. Set `swift_version` (and the archives' `platform_sha256`) on
`apple_cross_toolchain.swift()` accordingly.

## Remote Build Execution Setup (for BuildBuddy)

1. Define a `platform` target; for example, in `platforms/BUILD`:

    ```starlark
    platform(
        name = "docker_image_platform",
        constraint_values = [
            "@llvm//constraints/libc:gnu.2.28",
            "@platforms//cpu:x86_64",
            "@platforms//os:linux",
        ],
        exec_properties = {
            "OSFamily": "Linux",
            "container-image": "<your-docker-image>",
        },
    )
    ```

2. Add the following to your `.bazelrc`:

    ```
    build:remote --bes_backend=grpcs://app.buildbuddy.io
    build:remote --bes_results_url=https://app.buildbuddy.io/invocation/
    build:remote --jobs=1000
    build:remote --remote_executor=grpcs://app.buildbuddy.io
    build:remote --remote_timeout=3600
    build:remote --strategy=SwiftCompile=remote,sandboxed,worker,local
    ```

3. Build with `--config=remote`:

    ```
    bazel build --config=remote @rules_applecross//examples/ios/HelloWorldSwiftUI:HelloWorld
    ```

## Multi-platform RBE

rules_applecross supports multi-platform RBE, which means you can send source
code compilation to the Linux executors while having everything else (e.g.
resource handling, signing etc) runs on a Mac. That allows you to build a
complete app without relying on ported tools that may not have functional
parity.

Register the macOS instance to keep bundling actions on a Mac:

```starlark
register_toolchains("@rules_applecross//toolchain:apple_bundling_on_macos")
```

Then list the Linux platform first, so compiling lands there and everything
else falls through to the Mac:

```
build --extra_execution_platforms=@rules_applecross//platforms:linux_x86_64,@rules_applecross//platforms:macos_arm64_local
build --remote_executor=grpcs://<your-executor>
```

A Swift compile forced onto the Mac (say, a `swift_library` with
`exec_compatible_with = ["@platforms//os:macos"]`) still uses swift.org's
Swift, the macOS build of the same release, rather than Xcode's `swiftc`:
Xcode's compiler refuses modules built by a different compiler, so it could
not import the libraries compiled on Linux. Configure it with
`swift_tools_macos` (see this repository's `MODULE.bazel`).

### Keeping bundling on Linux

`@rules_applecross//toolchain:apple_bundling_on_linux` registers the same type
for the Linux platform, which is what an all-remote build uses. It is
incomplete: the ported `ibtool` compiles only the storyboards
`docs/clean-room/ibtool.md` describes, and a signing identity's key must be
on the machine that signs, which a remote executor is not (docs/consumers.md,
"Signing with an identity on Linux"). Register it only if you are building
everything remotely and can live with that.

## Byte-identical output

The clean-room tools that bundle on Linux are strict by default: their
output is byte-identical to Xcode's, or the build fails on the input, as
tvOS app icons do for now (`docs/clean-room`). To accept a near match
instead, relax them with `RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed` for
actions, as this repository's `--config=relaxed` does:

```
build:relaxed --action_env=RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed
```

Info.plist's `BuildMachineOSBuild` records the Mac an app was built on, so
it differs between Macs even with Xcode. On Linux it is the macOS SDK's build
(`26A425` for Xcode 27.0); to match a particular Mac's build, pass that Mac's
`sw_vers -buildVersion` as `--repo_env=RULES_APPLECROSS_BUILD_MACHINE_OS_BUILD=...`.
`DTSDKBuild` and `DTPlatformBuild` are the SDK's own build, as Xcode writes
them.

## Developing rules_applecross

### Ported tools

`tools/build-ported-tools.sh` builds, for the host's CPU, the Linux tools the
toolchain stages that LLVM doesn't provide:

- `tools/apple_tools`: `actool`, `ibtool`, `codesign`, `swift-stdlib-tool`,
  `PlistBuddy`, `plutil`, `sw_vers`, and `xcrun`
- `tools/apple_tools/swiftui_macros`: `SwiftUIMacros`
- `tools/system_tools`: `zip`, `unzip`, and `python3`
- `pngcrush`

These are prebuilt and uploaded here in this repository. See
`ported-tools-<VERSION>` releases.

### Tests

On Linux CI, builds are only tested against stub SDKs:

```
tools/stub-sdks.py
bazel test //tests/stub_sdks/...
```

To test against real SDKs on Linux, you need to package an SDK tarball and
provide it to the build yourself. See #packaging-the-sdk.
