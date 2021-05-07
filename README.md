# Bazel Apple toolchain for non-Apple platforms

This repository contains toolchain configuration rules for Bazel that can be
used to build apps and frameworks for Apple platforms from non-Apple platforms.
Current supported host is x86_64 Linux only.

## Setup

1. Add the following to your `WORKSPACE` file.

```starlark
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "com_github_apple_cross_toolchain_rules_applecross",
    sha256 = "",
    strip_prefix = "com_github_apple_cross_toolchain_rules_applecross-",
    url = "https://github.com/apple-cross-toolchain/rules_applecross/archive/.tar.gz",
)

http_archive(
    name = "build_bazel_rules_apple",
    patch_args = ["-p1"],
    patches = ["@com_github_apple_cross_toolchain_rules_applecross//third_party:rules_apple.patch"],
    sha256 = "5fed4c90b82006176b28d44d7642f520aa2fb9a32d30e24b22071fabcd24cbeb",
    strip_prefix = "rules_apple-0a67f1bd6c4cb9bd1bee5c51ac8d6632da537310",
    url = "https://github.com/bazelbuild/rules_apple/archive/0a67f1bd6c4cb9bd1bee5c51ac8d6632da537310.tar.gz",
)

load("@build_bazel_rules_apple//apple:repositories.bzl", "apple_rules_dependencies")

apple_rules_dependencies()

load(
    "@com_github_apple_cross_toolchain_rules_applecross//toolchain:apple_cross_toolchain.bzl",
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

2. Add the following to your `.bazelrc` file:

    ```
    build --apple_crosstool_top=@apple_cross_toolchain//:toolchain
    build --xcode_version_config=@com_github_apple_cross_toolchain_rules_applecross//xcode_config:host_xcodes # or your own `xcode_config` target
    ```

3. From your terminal, run these two commands:

   ```
   sudo tools/install-mandatory-tools.sh
   tools/set-developer-dir.sh
   ```

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

## Examples

```
bazel build //examples/ios/HelloWorldSwiftUI:HelloWorld
```
