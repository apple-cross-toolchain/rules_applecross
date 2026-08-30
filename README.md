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
    build --action_env=DEVELOPER_DIR=external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer
    build --action_env=PATH=external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin:/bin:/usr/bin:/usr/local/bin
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

## Running iOS unit tests on Linux

`//testing:ios_vm_test_runner.bzl` provides an `AppleTestRunnerInfo` runner for
unhosted iOS unit tests. It boots a prepared
[`darwin-vm`](https://github.com/apple-cross-toolchain/darwin-vm) image under
QEMU, patches the device-arm64 test executable into a reserved APFS slot, adds
its ad-hoc signature to a per-test trust cache, and invokes XCTest in the guest.
The test action is local-only by default, so firmware and Xcode runtime files
are not uploaded to a remote executor or cache.

Use a **non-SPTM** restore image. The tested combination is iPhone 12
(`iPhone13,2`), iOS 26.5, Xcode 26.6, arm64, and an arm64 Linux host. Newer
SPTM/TXM images can hit `nested panic count exceeds limit` and currently kill
Linux-built XCTest bundles even when a retry boots successfully.

Prepare the XCTest-enabled ramdisk on a Mac using darwin-vm's
`xctest/prepare_ramdisk.sh`. The script reserves a 4 MiB executable slot,
installs the matching device XCTest runtime from Xcode's Restore DDI, fixes the
restore image ownership, and emits a manifest that binds all firmware hashes:

```sh
DEVNAME=iPhone13,2 URL='<Apple iPhone 12 restore IPSW URL>' ./get_files.sh

./xctest/prepare_ramdisk.sh \
  --ramdisk firmware/ramdisk.dmg \
  --hashes firmware/all_hashes \
  --output-dir firmware/xctest
```

The runtime consists of five matching firmware files (`bootkc`, `dtree`,
`ramdisk.dmg`, `ramdisk.tc`, and `slot.json`) plus a native Linux
`qemu-system-aarch64` built from darwin-vm's `qemu-sptm` fork. Stock distro
QEMU does not provide the `darwin` machine. The firmware and prepared ramdisk
contain Apple IPSW and Xcode software; do not commit, redistribute, or upload
them to a third-party remote execution service unless the applicable licenses
permit it. This repository's CI deletes its one-run transfer artifact after
the Linux consumer finishes.

Consumers running the test on an arm64 Linux host while building products on
x86_64 Linux must register the test toolchain and both Linux bundling markers
in `MODULE.bazel`:

```starlark
register_toolchains("@rules_applecross//testing:ios_test_toolchain")
register_toolchains("@rules_applecross//toolchain:apple_bundling_on_linux")
register_toolchains("@rules_applecross//toolchain:apple_bundling_on_linux_arm64")
```

Use the `ios_vm` module extension to declare prepared firmware runtimes and a
QEMU binary. For downloaded inputs, provide immutable URLs and SHA-256
digests:

```starlark
ios_vm = use_extension(
    "@rules_applecross//testing:ios_vm_extension.bzl",
    "ios_vm",
)
ios_vm.firmware(
    name = "iphone12_ios26_5",
    device_type = "iPhone 12",
    os_version = "26.5",
    urls = ["https://example.invalid/ios-vm-iphone12-26.5.tar.zst"],
    sha256 = "<firmware archive SHA-256>",
    archive_type = "tar.zst",
)
ios_vm.qemu(
    urls = ["https://example.invalid/qemu-sptm-linux-arm64.tar.xz"],
    sha256 = "<QEMU archive SHA-256>",
    archive_type = "tar.xz",
)
use_repo(ios_vm, "ios_vm_runtime")
```

Repeat `ios_vm.firmware` for every available runtime, giving each declaration
a unique, stable Bazel target `name`. Names must start with a lowercase ASCII
letter or digit; subsequent characters may also be `_`, `.`, or `-`, and
`all` is reserved. The extension materializes these typed targets in
`@ios_vm_runtime`. Firmware archives are fetched lazily, so selecting one
runtime target does not download the others. The extension is the sole source
of each runtime's `device_type` and `os_version`; include the OS build
identifier in `os_version` when more than one build shares a marketing
version.

After extraction and any `strip_prefix`, the firmware archive must contain
`bootkc`, `dtree`, `ramdisk.dmg`, `ramdisk.tc`, and `slot.json` at its root.
It may additionally contain both `sptm` and `txm` for the experimental SPTM
configuration. The QEMU archive must contain `qemu-system-aarch64` at its root
by default; set `binary_path` when the executable has another relative path.
It must match the Linux OS and architecture that will execute the local test;
the current CI test host is Linux arm64.
Set `archive_type` when it cannot be inferred from the URL. Both tags also
accept `strip_prefix` for archives with a common top-level directory. Preserve
the QEMU executable bit, and do not include Bazel repository files such as
`BUILD.bazel` or `MODULE.bazel`; the extension generates those files.

For assets already staged beside the consumer's `MODULE.bazel`, use paths
relative to that module instead. The firmware `path` names the flat directory,
while the QEMU `path` names the executable:

```starlark
ios_vm = use_extension(
    "@rules_applecross//testing:ios_vm_extension.bzl",
    "ios_vm",
)
ios_vm.firmware(
    name = "iphone12_ios26_5",
    device_type = "iPhone 12",
    os_version = "26.5",
    path = "assets",
)
ios_vm.qemu(path = "assets/qemu-system-aarch64")
use_repo(ios_vm, "ios_vm_runtime")
```

There is deliberately no default firmware or QEMU download. In particular,
`qemu-sptm` is GPLv2 software: anyone distributing a prebuilt QEMU archive
must preserve its notices and provide the complete corresponding source under
the GPL. Supply a release that meets those obligations or point at a local
build. See the
[`qemu-sptm` license](https://github.com/jprx/qemu-sptm/blob/006cc6b174e6177e64d06a6457e4125fd627649f/LICENSE).

Define the test runner by selecting the materialized runtime target. The
extension keeps the shared QEMU implementation private inside each runtime, so
the runner needs only this one label:

```starlark
load("@rules_applecross//testing:ios_vm_test_runner.bzl", "ios_vm_test_runner")

ios_vm_test_runner(
    name = "ios_vm_runner",
    runtime = "@ios_vm_runtime//:iphone12_ios26_5",
)

ios_unit_test(
    name = "UnitTests",
    runner = ":ios_vm_runner",
    minimum_os_version = "16.0",
    timeout = "long",
    deps = [":TestsLib"],
)
```

The runner gets its device type and OS version from the selected typed target,
whose metadata comes from the extension declaration. Those runtime values are
separate from the test target's
`minimum_os_version`, which is the deployment target used when compiling the
test bundle. A test built with `minimum_os_version = "16.0"` can therefore run
in the selected iOS 26.5 VM.

The VM emulates an iOS device, so select device arm64 rather than a simulator:

```sh
bazel test --config=remote \
  --remote_header=x-buildbuddy-api-key="$BUILDBUDDY_API_KEY" \
  --apple_platform_type=ios \
  --ios_multi_cpus=arm64 \
  //:UnitTests
```

On arm64 Linux, `--config=remote` supplies the x86_64 Linux execution platform
needed by the current Apple build tools while the `no-remote` test action runs
QEMU on the local host. An equivalent self-hosted x86_64 execution platform can
be used instead.

The initial runner supports one unhosted XCTest bundle executable and positive
test filters. Hosted tests, UI tests, nested frameworks, bundle resources,
coverage, arm64e payloads, and negative filters fail explicitly rather than
silently producing incorrect results. It requires `python3` on the Linux test
host (Python 3.10 or newer). Its JUnit XML currently represents the whole
XCTest invocation as one synthetic testcase rather than one element per XCTest
method. SPTM/TXM inputs and `boot_attempts > 1` remain available for
experiments; only recognized SPTM/TXM panic signatures are retried.

The license-controlled CI smoke test is a standalone consumer workspace under
`examples/ios/VMUnitTest`. Once its ignored assets and `apple-sdks.tar.zst`
have been staged as described in its README, run:

```sh
cd examples/ios/VMUnitTest
bazel test --config=remote \
  --remote_header=x-buildbuddy-api-key="$BUILDBUDDY_API_KEY" \
  --apple_platform_type=ios \
  --ios_multi_cpus=arm64 \
  //:UnitTests
```

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
