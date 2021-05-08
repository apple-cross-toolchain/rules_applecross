# Bazel Apple toolchain for non-Apple platforms

This repository contains toolchain configuration rules for Bazel that can be
used to build apps and frameworks for Apple platforms from non-Apple platforms.
Current supported host is x86_64 Linux only.

## Setup

1. First, clone this repository and provision your OS with:

    ```
    git clone https://github.com/apple-cross-toolchain/rules_applecross.git
    cd rules_applecross
    sudo tools/install-mandatory-tools.sh
    ```

This installs tools required by Apple rules (e.g. `xcrun`) onto the system
PATH, as they are not available on non-Apple platforms.

2. Add the following to your `WORKSPACE` file.

```starlark
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_applecross",
    sha256 = "3c6b017792789714323d75f29569239855cfb223ce28768df39a910fe7842863",
    strip_prefix = "rules_applecross-0.0.2",
    url = "https://github.com/apple-cross-toolchain/rules_applecross/archive/refs/tags/0.0.2.tar.gz",
)

http_archive(
    name = "build_bazel_rules_apple",
    patch_args = ["-p1"],
    patches = ["@rules_applecross//third_party:rules_apple.patch"],
    sha256 = "5fed4c90b82006176b28d44d7642f520aa2fb9a32d30e24b22071fabcd24cbeb",
    strip_prefix = "rules_apple-0a67f1bd6c4cb9bd1bee5c51ac8d6632da537310",
    url = "https://github.com/bazelbuild/rules_apple/archive/0a67f1bd6c4cb9bd1bee5c51ac8d6632da537310.tar.gz",
)

load("@build_bazel_rules_apple//apple:repositories.bzl", "apple_rules_dependencies")

apple_rules_dependencies()

load(
    "@rules_applecross//toolchain:apple_cross_toolchain.bzl",
    "apple_cross_toolchain",
)

apple_cross_toolchain(
    name = "apple_cross_toolchain",
    clang_sha256 = "8f50330cfa4c609841e73286a3a056cff95cf55ec04b3f1280d0cd0052e96c2a",
    clang_strip_prefix = "clang+llvm-12.0.0-x86_64-linux-gnu-ubuntu-20.04",
    clang_urls = ["https://github.com/apple-cross-toolchain/ci/releases/download/0.0.6/clang+llvm-12.0.0-x86_64-linux-gnu-ubuntu-20.04-stripped.tar.xz"],
    swift_sha256 = "869edb04a932c9831922541cb354102244ca33be0aa6325d28b0f14ac0a32a4d",
    swift_strip_prefix = "swift-5.3.3-RELEASE-ubuntu20.04",
    swift_urls = ["https://github.com/apple-cross-toolchain/ci/releases/download/0.0.6/swift-5.3.3-RELEASE-ubuntu20.04-stripped.tar.xz"],
    xcode_sha256 = "44221c0f4acd48d7a33ee7e51143433dee94c649cfee44cfff3c7915ac54fdd2",
    xcode_urls = ["https://github.com/apple-cross-toolchain/apple-sdks/releases/download/0.0.4/apple-sdks-xcode-12.4.tar.xz"],
)

load("@apple_cross_toolchain//:repositories.bzl", "apple_cross_toolchain_dependencies")

apple_cross_toolchain_dependencies()

load("@build_bazel_rules_swift//swift:repositories.bzl", "swift_rules_dependencies")

swift_rules_dependencies()

load("@build_bazel_rules_swift//swift:extras.bzl", "swift_rules_extra_dependencies")

swift_rules_extra_dependencies()
```

3. Add the following to your `.bazelrc` file:

    ```
    build --apple_crosstool_top=@apple_cross_toolchain//:toolchain
    build --xcode_version_config=@rules_applecross//xcode_config:host_xcodes # or your own `xcode_config` target
    ```

4. From your workspace, run these commands:

    ```
    bazel fetch @rules_applecross//tests/data:dummy_lib
    DEVELOPER_DIR="$(bazel info output_base)/external/apple_cross_toolchain/Xcode.app/Contents/Developer"
    sudo xcode-select -s "$DEVELOPER_DIR"
    ```

These commands triggers the auto-configuration of the toolchain and selects the
active developer directory. There is currently no other way to avoid this kind
of workaround, because Apple rules don't include tools in their action inputs,
but rely on `xcrun` to invoke tools.

Note: 
- You can use a different `rules_apple` version, but it will need a patch like
  [third_party/rules_apple.patch](third_party/rules_apple.patch) because the
  ported PlistBuddy tool can't handle multiple commands now.
- `apple_cross_toolchain_dependencies()` needs to be called after
  `apple_rules_dependencies()` and before `swift_rules_dependencies()`.
- You can use the official Clang and Swift releases in
  `clang_urls`/`swift_urls`. The example here uses stripped-down archives (that
  only contain what we need) to speed up the decompression during the toolchain
  configuration.

## Remote Build Execution Setup (for BuildBuddy)

1. Define a `platform` target; for example, in `platforms/BUILD`:

```starlark
platform(
    name = "docker_image_platform",
    constraint_values = [
        "@bazel_tools//platforms:x86_64",
        "@bazel_tools//platforms:linux",
        "@bazel_tools//tools/cpp:clang",
    ],
    exec_properties = {
        "OSFamily": "Linux",
        "container-image": "docker://ghcr.io/apple-cross-toolchain/xcode:12.4",
    },
)
```

2. Download BuildBuddy client certificate and key and put them at the top level
   directory of your workspace as `buildbuddy-cert.pem` and
   `buildbuddy-key.pem` (make sure they aren't tracked by your version control
   system).

3. Add the following to your `.bazelrc` file:

```
build:remote --bes_backend=grpcs://cloud.buildbuddy.io
build:remote --bes_results_url=https://app.buildbuddy.io/invocation/
build:remote --host_platform=//platforms:docker_image_platform
build:remote --jobs=100
build:remote --remote_download_toplevel
build:remote --remote_executor=grpcs://cloud.buildbuddy.io
build:remote --remote_timeout=3600
build:remote --strategy=SwiftCompile=remote,sandboxed,worker,local
build:remote --tls_client_certificate=buildbuddy-cert.pem
build:remote --tls_client_key=buildbuddy-key.pem
```

Now you can build your target with `--config=remote`.

## Examples

```
bazel build //examples/ios/HelloWorldSwiftUI:HelloWorld
bazel build --config=remote //examples/ios/HelloWorldSwiftUI:HelloWorld
```

See the [examples
repository](https://github.com/apple-cross-toolchain/examples) for more
real-world examples.
