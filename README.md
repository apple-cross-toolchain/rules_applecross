# Bazel Apple toolchain for non-Apple platforms

This repository contains toolchain configuration rules for Bazel that can be
used to build apps and frameworks for Apple platforms from non-Apple platforms.
Requires Bazel 9+.

Builds execute on x86_64 Linux, either locally or through remote execution. The
host driving them can be x86_64 Linux or macOS; see [Building on
macOS](#building-on-macos) for what a macOS host does differently.

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
    build --xcode_version_config=@apple_cross_toolchain//:host_xcodes
    build --action_env=DEVELOPER_DIR=external/+apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer
    build --@build_bazel_rules_apple//apple:sdk_tool_files=@apple_cross_toolchain//:sdk_tool_files
    ```

    The Xcode and SDK versions are read out of the SDK tree when
    `@apple_cross_toolchain` is fetched, so there is nothing to pin by hand.

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
    swift_tools = "...",
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
framework. Set `swift_version` on `applecross_swift.toolchain()` accordingly.

## Remote Build Execution Setup (for BuildBuddy)

1. Define a `platform` target; for example, in `platforms/BUILD`:

    ```starlark
    platform(
        name = "docker_image_platform",
        constraint_values = [
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

### Keeping bundling on Linux

`@rules_applecross//toolchain:apple_bundling_on_linux` registers the same type
for the Linux platform, which is what an all-remote build uses. It is
incomplete: the ported `ibtool` writes an empty `.storyboardc` and `codesign`
is a no-op stub, so bundles come out unsigned and missing their storyboards,
and `-c opt` needs a `zip` the executor image does not have. Register it only
if you are building everything remotely and can live with that.

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
GitHub API, which is the only endpoint that honors token auth there. CI runs
the same script, so neither path requires editing a checked-in file to build.
