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

- Metal shaders (`metal`, `metallib`) as Xcode compiles them: relaxed, they
  compile (below);
- App Shortcuts Flexible Matching (`app_intents_flexible_matching`, on by
  default; set it to `False`). App Intents' intents, parameters, enums,
  entities, queries and App Shortcuts compile as Xcode compiles them
  ([docs/clean-room/appintentsmetadataprocessor.md](docs/clean-room/appintentsmetadataprocessor.md));
- Core Data mapping models (`mapc`). Models compile, each version to the
  `.mom` Xcode writes, and a single version's `VersionInfo.plist` as Xcode
  writes it; a versioned model or one with a derived attribute needs
  `--config=relax_momc`, its `VersionInfo.plist` without the entity version
  hashes, which Core Data computes itself, so stores load, save and migrate
  as with Xcode's; their generated classes are Xcode's but for the header
  line naming the user and the date. Composite
  attributes, fetch indexes and names other than ASCII are refused
  ([docs/clean-room/momc.md](docs/clean-room/momc.md));
- Core ML's older neural networks. ML Programs (`.mlpackage`, states,
  several functions, flexible shapes, optional inputs and compressed weights
  included) and the other `.mlmodel` types compile, and get their generated
  classes, as Xcode makes them: tree ensembles, support vector machines,
  linear models, k-nearest neighbors (updatable too), item similarity
  recommenders, the feature transforms, non-maximum suppression, custom and
  linked models, Vision and audio feature prints, and pipelines of any of
  these; and Create ML's text classifiers, word taggers, gazetteers, word
  embeddings, recommenders, and image, sound and tabular models; any of
  these in a `.mlpackage` too, with the metadata and feature descriptions
  Xcode's model editor saves there
  ([docs/clean-room/coremlc.md](docs/clean-room/coremlc.md));
- Reality Composer content (`realitytool`);
- `.xib` files other than an iOS one holding a view, a table view cell or a
  collection view cell
  ([docs/clean-room/ibtool.md](docs/clean-room/ibtool.md#xibs));
- tvOS app icons, and Icon Composer icons, whose layers (all iOS and macOS
  26 and later draw) are Xcode's but whose flattened images aren't
  (`--config=relax_actool_icon_composer` accepts them;
  [docs/clean-room/actool.md](docs/clean-room/actool.md)).
- Vector images (PDF, SVG) in image sets: Xcode keeps the file and draws it
  into a bitmap at each scale with CoreGraphics, which this actool doesn't
  reproduce. `--config=relax_actool_vector` accepts its own drawing
  (resvg, hayro) for one file per appearance on iOS and macOS
  ([docs/clean-room/actool.md](docs/clean-room/actool.md#vector-images)).

A target with Metal shaders builds correctly with its bundling on a macOS
execution platform (`--@rules_applecross//toolchain:apple_bundling=macos`:
compiles stay on Linux, and Xcode's `metal` compiles the shaders);
`--config=relax_metal` instead compiles them with this repository's `metal`
and `metallib` (`tools/apple_tools/crates/metal`): the MSL is compiled to AIR
by LLVM 15, the last LLVM with the typed pointers AIR uses, with headers of our own, learned
from Xcode's output for our own shaders. The shaders work, and many are
Xcode's bytes, but not yet all, so the compile stays relaxed until it matches
Xcode across a measured corpus. Only that config (its `--@rules_applecross//toolchain:relax_metal`)
fetches LLVM 15, from toolchains_llvm. String
Catalogs compile as Xcode compiles them; the few it can't yet reproduce, it
refuses unless `--config=relax_xcstrings`
([docs/clean-room/xcstringstool.md](docs/clean-room/xcstringstool.md)). Each
deviation has its own config, and `--config=relaxed` accepts them all
([Byte-identical output](#byte-identical-output)).

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
        xcode_version = "27.2",
        apple_sdk_urls = ["<url-to-apple-sdk-archive>"],
        apple_sdk_archive_type = "tar.xz",  # if not inferrable from URL
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

    Swift is swift.org's build of the Swift that built `xcode_version`'s
    SDKs ([toolchain/xcode_versions.bzl](toolchain/xcode_versions.bzl)), for
    Linux and, for Swift compiles on a macOS execution platform, for macOS.
    To use another, declare it with `apple_cross_toolchain.swift()` and name
    its targets in `swift_tools` and `swift_tools_macos`, as this
    repository's `MODULE.bazel` does.

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
bazel run //tools:package_sdks -- ~/Downloads/Xcode_27.2.xip apple-sdks-27.2.tar.gz
```

On a Mac, the following packages the selected Xcode instead:

```
tools/package-sdks.sh
```

## Building on macOS

```starlark
apple_cross_toolchain.configure(
    name = "apple_cross_toolchain",
    xcode_version = "27.2",
    apple_sdk_urls = ["<url-to-apple-sdk-archive>"],
    local_xcode = True,  # this skips staging the SDKs
)
```

The Swift toolchain must match the Swift that built those SDKs, which is the
Swift of the Xcode *one release behind* the SDKs' own: Xcode 26.6 ships the 26.5
SDKs, and those were built by Swift 6.3.2. A mismatch shows up as `this SDK is
not supported by the compiler` when compiling any Swift that imports a system
framework. `xcode_version` picks the matching Swift; with your own
`swift_tools`, set `swift_version` (and the archives' `platform_sha256`) on
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
incomplete: the ported `ibtool` compiles only the storyboards and xibs
`docs/clean-room/ibtool.md` describes, and a signing identity's key must be
on the machine that signs, which a remote executor is not (docs/consumers.md,
"Signing with an identity on Linux"). Register it only if you are building
everything remotely and can live with that.

## Byte-identical output

The clean-room tools that bundle on Linux are strict by default: their
output is byte-identical to Xcode's, or the build fails on the input, as
tvOS app icons do for now (`docs/clean-room`). Each deviation from Xcode's
output can be accepted on its own, with an action environment variable that
this repository's `.bazelrc` names as a config, so a project accepts just the
ones it can live with. An app that only runs on iOS 26 and later, say, may
accept Icon Composer icons whose rasters aren't Xcode's, since those systems
draw the icon from its layers:

| Config | Variable (`=1`) | Accepts |
| --- | --- | --- |
| `relax_actool_icon_composer` | `RULES_APPLECROSS_RELAX_ACTOOL_ICON_COMPOSER` | Icon Composer documents whose flattened images are this actool's (their layers are Xcode's) |
| `relax_actool_tvos_radiosity` | `RULES_APPLECROSS_RELAX_ACTOOL_TVOS_RADIOSITY` | tvOS radiosity images that may round apart from Xcode's |
| `relax_actool_vector` | `RULES_APPLECROSS_RELAX_ACTOOL_VECTOR` | vector images (PDF, SVG) whose bitmaps are this actool's drawing, and whose PDFs, where this actool can't write them as CoreGraphics does, are stored as they are |
| `relax_actool_storage` | `RULES_APPLECROSS_RELAX_ACTOOL_STORAGE` | images stored as this actool chooses where Xcode's choice isn't established: atlases split next-fit, dmp2 streams near a margin or tied, bitmap keys for an identifier count whose order isn't pinned (the same images, packed or encoded perhaps otherwise) |
| `relax_actool_xcbuild` | `RULES_APPLECROSS_RELAX_ACTOOL_XCBUILD` | asset catalogs this actool can't reproduce, compiled by xcbuild's actool |
| `relax_docc` | `RULES_APPLECROSS_RELAX_DOCC` | DocC archives with swift.org's render template, where the SDK archive lacks Xcode's |
| `relax_ibtool` | `RULES_APPLECROSS_RELAX_IBTOOL` | Interface Builder files this ibtool can't reproduce, left uncompiled |
| `relax_ibtool_layout` | `RULES_APPLECROSS_RELAX_IBTOOL_LAYOUT` | AppKit controls Auto Layout sizes by their content, which Xcode lays out again as it compiles, compiled with the document's frames |
| `relax_momc` | `RULES_APPLECROSS_RELAX_MOMC` | Core Data models with several versions or a derived attribute, whose `VersionInfo.plist` has no entity version hashes or checksums, without the `.omo` Xcode writes beside a versioned model's current version; generated classes whose header doesn't name the user and the date |
| `relax_pngcrush` | `RULES_APPLECROSS_RELAX_PNGCRUSH` | iOS PNG resources this pngcrush can't make Xcode's iPhone-optimized PNG of (other than 8-bit RGB and RGBA), copied as they are |
| `relax_metal` | `RULES_APPLECROSS_RELAX_METAL`, with `--@rules_applecross//toolchain:relax_metal` | Metal shaders compiled with LLVM 15 and this repository's `metal` (working; Xcode's bytes for many, not yet all; bundling on a Mac compiles them as Xcode does). Without the flag, an empty library, whose shaders fail at run time |
| `relax_xcstrings` | `RULES_APPLECROSS_RELAX_XCSTRINGS` | String Catalog entries not yet reproduced, in their nearest form |
| `relax_app_intents` | `RULES_APPLECROSS_RELAX_APP_INTENTS` | App Intents not yet reproduced, left out |
| `relax_app_intents_flexible_matching` | `RULES_APPLECROSS_RELAX_APP_INTENTS_FLEXIBLE_MATCHING` | App Shortcuts Flexible Matching's assets, left out |
| `relax_xcode_version` | `RULES_APPLECROSS_RELAX_XCODE_VERSION` | an Xcode other than 27.0 and 27.2: actool and coremlc record the library versions of the nearest one (27.0's before 27.2, 27.2's after), momc 27.2's |
| `relax_clang_apple_flags` | `RULES_APPLECROSS_RELAX_APPLE_CLANG_FLAGS` | C, C++ and Objective-C compiled off a Mac with flags only Xcode's clang accepts (`-fobjc-relative-method-lists` and 97 others, listed in `toolchain/wrapped_clang.cc`), dropped with a warning: LLVM's clang compiles there and rejects them, so the code may compile otherwise (Objective-C method lists absolute rather than relative, say) |
| `relax_xcode_build` | `RULES_APPLECROSS_RELAX_XCODE_BUILD` (a `--repo_env`) | another build of `xcode_version`'s Xcode than the pinned one (27B48 for 27.2's 27B52), when the same Swift built its SDKs: the Swift the pin selects compiles, and the bundling tools still produce the pinned build's output |

`--config=relaxed` accepts all of them, and so does
`RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed`, which it also sets for
ported tools from before these configs (whose actool and ibtool read only
that). Copy the lines you want from this repository's `.bazelrc`, for
example:

```
build:relax_actool_icon_composer --action_env=RULES_APPLECROSS_RELAX_ACTOOL_ICON_COMPOSER=1
build:ios26 --config=relax_actool_icon_composer
```

Info.plist's `BuildMachineOSBuild` records the Mac an app was built on, so
it differs between Macs even with Xcode. On Linux it is the macOS SDK's build
(`26A425` for Xcode 27.0); to match a particular Mac's build, pass that Mac's
`sw_vers -buildVersion` as `--repo_env=RULES_APPLECROSS_BUILD_MACHINE_OS_BUILD=...`.
`DTSDKBuild` and `DTPlatformBuild` are the SDK's own build, as Xcode writes
them.

## Building with actiond

[actiond](https://github.com/hermeticbuild/actiond) runs every action in a
Linux VM on the host, in a sandbox holding nothing but glibc and bash. Start it
and build with `--config=actiond`:

```
darwin-actiond_macos_arm64 serve-vm --memory-mib=4096 --cpus=4
bazel build --config=actiond //examples/ios/HelloWorldSwiftUI:HelloWorld
```

The VM has the host's CPU, which the config passes on as
`--repo_env=RULES_APPLECROSS_LINUX_EXEC_CPU=host` (`x86_64` and `aarch64`
also work), so on an Apple silicon Mac `swift_tools` needs its `aarch64` entry.
The toolchain brings the libraries and the static [uutils](https://github.com/uutils)
coreutils and findutils its own actions need, but other actions get neither:
skylib's `build_test` genrules fail on `touch`.

A module depending on this one also needs the `actiond` lines of `.bazelrc`,
naming `@rules_applecross//platforms:actiond` and the `external/` paths as in
[Setup](#setup), and the `rules_python` override from `MODULE.bazel`.

## Developing rules_applecross

### Ported tools

`tools/build-ported-tools.sh` builds, for the host's CPU, the Linux tools the
toolchain stages that LLVM doesn't provide:

- `tools/apple_tools`: `actool`, `ibtool`, `pngcrush`, `codesign`,
  `swift-stdlib-tool`, `PlistBuddy`, `plutil`, `sw_vers`, and `xcrun`
- `tools/apple_tools/swiftui_macros`: `SwiftUIMacros`
- `tools/system_tools`: `zip`, `unzip`, and `python3`

These are prebuilt and released here as `ported-tools-<version>`. When a push
to `main` changes them, the `ported-tools` workflow releases the next patch
version and opens a PR that moves the toolchain to it.

### Tests

On Linux CI, builds are only tested against stub SDKs:

```
tools/stub-sdks.py
bazel test //tests/stub_sdks/...
```

To test against real SDKs on Linux, you need to package an SDK tarball and
provide it to the build yourself. See #packaging-the-sdk.
