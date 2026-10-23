# Bazel Apple toolchain for non-Apple platforms

This repository contains toolchain configuration rules for Bazel that can be
used to build apps and frameworks for Apple platforms from non-Apple platforms.
Requires Bazel 8.8+ or 9+.

Builds execute on x86_64 Linux, either locally or through remote execution, or
locally on arm64 Linux. The host driving them can be Linux or macOS; see
[Building on macOS](#building-on-macos) for what a macOS host does differently.

On arm64 Linux, configure the aarch64 Swift too:

```starlark
apple_cross_toolchain.configure(
    name = "apple_cross_toolchain",
    # ...
    swift_tools = {
        "aarch64": "@applecross_swift_ubuntu24.04-aarch64//:tools",
        "x86_64": "@applecross_swift_ubuntu24.04//:tools",
    },
)
```

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
    )
    use_repo(apple_cross_toolchain, "apple_cross_toolchain")

    register_toolchains("@apple_cross_toolchain//:cc-toolchain-ios_x86_64")
    register_toolchains("@apple_cross_toolchain//:cc-toolchain-ios_arm64")
    register_toolchains("@apple_cross_toolchain//:swift-toolchain-ios_x86_64")
    register_toolchains("@apple_cross_toolchain//:swift-toolchain-ios_arm64")
    # ... add more platforms as needed (darwin, tvos, watchos)
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

    The Xcode and SDK versions are read out of the SDK tree when
    `@apple_cross_toolchain` is fetched, so there is nothing to pin by hand.
    The startup flag lets Bazel track the SDK directories the toolchain
    declares as inputs. `incompatible_repo_env_ignores_action_env` keeps the
    relative `DEVELOPER_DIR` out of repository rules on Bazel 8, as Bazel 9
    already does. Inside this repository the paths start with
    `external/+apple_cross_toolchain+apple_cross_toolchain`.

4. Build an example:

    ```
    bazel build @rules_applecross//examples/ios/HelloWorldSwiftUI:HelloWorld
    ```

## Notes

- Apple SDKs archives are not publicly available. If you have access to macOS, you
  can build one yourself by running `tools/package-sdks.sh`.
- On a macOS host you can skip the archive entirely: set `local_xcode = True` on
  `apple_cross_toolchain.configure()` and the toolchain repository builds the SDK
  tree straight out of the installed Xcode. See [Building on
  macOS](#building-on-macos).

## Building on macOS

A macOS host has an Xcode already, so there is no reason to repackage one:

```starlark
apple_cross_toolchain.configure(
    name = "apple_cross_toolchain",
    apple_sdk_urls = ["<url-to-apple-sdk-archive>"],
    local_xcode = True,
    swift_tools = {"x86_64": "..."},
)
```

`local_xcode` reads the SDKs from the Xcode that `DEVELOPER_DIR` names, or that
`xcode-select -p` selects. Hosts other than macOS ignore it and read the archive
source instead, so the same `MODULE.bazel` serves a Linux CI host and a macOS
developer machine unchanged. Set `--repo_env=RULES_APPLECROSS_LOCAL_XCODE=0` to
force the archive on macOS, or `=1` to turn a missing Xcode into an error rather
than a silent fallback.

The tree is assembled almost entirely out of symlinks into the install: a real
directory appears only where the toolchain has to write, which is the framework
stubs no Xcode ships and the toolchain directory the Linux-hosted tools land in.
It presents exactly the file set the archive contains — same names, same contents
— so both sources produce identical action keys and share the remote cache.

Upgrading Xcode in place refetches the repository, because the rule records the
selected Xcode's `Contents/version.plist` as an input, and so does changing
`DEVELOPER_DIR`. Pointing `xcode-select -s` at a *different* Xcode does not:
nothing the rule records changes, so refetch it with `bazel fetch --configure
--force`. Deleting or moving the selected Xcode leaves the symlinks dangling,
which the same command repairs.

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

    Exec tools link against glibc, so the image must provide glibc 2.28 or
    newer.

2. Add the following to your `.bazelrc`:

    ```
    build:remote --bes_backend=grpcs://app.buildbuddy.io
    build:remote --bes_results_url=https://app.buildbuddy.io/invocation/
    build:remote --jobs=100
    build:remote --remote_download_toplevel
    build:remote --remote_executor=grpcs://app.buildbuddy.io
    build:remote --remote_timeout=3600
    build:remote --strategy=SwiftCompile=remote,sandboxed,worker,local
    build:remote --tls_client_certificate=buildbuddy-cert.pem
    build:remote --tls_client_key=buildbuddy-key.pem
    ```

3. Build with `--config=remote`:

    ```
    bazel build --config=remote @rules_applecross//examples/ios/HelloWorldSwiftUI:HelloWorld
    ```

## Splitting a build between Linux and macOS

Compiling can run on the Linux executors while everything `rules_apple`
produces — linking, `actool` and `ibtool`, plists, bundling, signing — runs on
a Mac with a real Xcode. That allows you to build a complete app without
relying on ported tools that may not have functional parity.

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
`docs/clean-room/ibtool.md` describes, and `codesign` signs ad hoc only.
Register it only if you are building everything remotely and can live with
that.

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

This repository's own `MODULE.bazel` sets `local_xcode = True`, so on macOS the
examples build against the installed Xcode with nothing to fetch or unpack.

Every other host reads the SDK archive from the workspace root via
`apple_sdk_path`, so building the examples there needs that archive in place
first. Download a packaged one:

```
SDKS_URL=<url> tools/fetch-sdks.sh
```

Add `SDKS_TOKEN=<token>` when the host needs credentials. For a GitHub
release on a private repository, `SDKS_URL` can be the ordinary release
download URL (`.../releases/download/<tag>/<name>` or
`.../releases/latest/download/<name>`); the script resolves it through the
GitHub API, which is the only endpoint that honors token auth there. Neither
path requires editing a checked-in file to build.

### Ported tools

`tools/build-ported-tools.sh` builds, for the host's CPU, the Linux tools the
toolchain stages that LLVM doesn't provide:

- `tools/apple_tools`, built with Bazel and rules_rs: the clean-room tools in
  `crates` (see `docs/clean-room`) and, in `xcbuild`, the Rust port of the
  xcbuild tools rules_apple runs (`xcrun`, `plutil`, `PlistBuddy`, `sw_vers`);
- `tools/apple_tools/swiftui_macros`, built with SwiftPM;
- `tools/system_tools`, built with Bazel and statically linked against musl,
  as executor images may lack them: Info-ZIP `zip` and `unzip` with Debian's
  patches, and CPython `python3` through rules_foreign_cc, whose archive also
  carries the standard library;
- pngcrush, built from source.

It runs in the Linux container `tools/linux_vm/Containerfile` describes and
writes each tool as `ported-tools/<tool>-linux-<arch>.tar.zst`. The
`ported-tools` workflow builds them for x86_64 and aarch64 and, on `main`,
publishes them with a `SHA256.txt` as the release `ported-tools-<VERSION>`
unless it exists, so bump `VERSION` in `.github/workflows/ported-tools.yml` to
release changes.

The toolchain downloads the archives `toolchain/ported_tools.sha256` lists, a
copy of a release's `SHA256.txt`, from the release `_PORTED_TOOLS_URL` in
`toolchain/apple_cross_toolchain.bzl` names; update the two together. To try
local builds, name the `ported-tools` directory in `ported_tools_archives`.

### Tests

Stub SDK tests run on GitHub's Linux runners, which never see an Apple SDK.
They build against stub SDKs: generated trees holding swift.org's Darwin Swift
runtime and placeholders for everything Apple ships. `tools/stub-sdks.py` writes
them as the `apple-sdks.tar.zst` that a Linux host without Xcode reads:

```
tools/stub-sdks.py
bazel test //tests/stub_sdks/...
```

Stub SDKs compile, link and bundle code that imports no Apple framework, and
every example analyzes against them.

Real SDK tests build the examples against a real Xcode, so they run only on
Apple hardware. CI runs them natively on GitHub's macOS runners:

```
tools/real-sdk-tests.sh macos
```

The Mac leg skips platforms without a simulator runtime installed, because
Xcode's `actool` needs one.

CI doesn't run them on Linux, so run the Linux leg on a Mac before sending a
change. It builds the examples in a Linux VM on that Mac, reading its Xcode
through a read-only mount:

```
tools/real-sdk-tests.sh linux-vm
```

It needs podman, or Docker whose VM can see the Xcode. For Docker Desktop,
share `/Applications`, or point `RULES_APPLECROSS_VM_XCODE` at a copy under a
shared path.
