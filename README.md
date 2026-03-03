# Bazel Apple toolchain for non-Apple platforms

This repository contains toolchain configuration rules for Bazel that can be
used to build apps and frameworks for Apple platforms from non-Apple platforms.
Current supported host is x86_64 Linux only. Requires Bazel 9+.

## Setup

1. Install system tools (one-time, requires root):

    ```
    sudo tools/install-mandatory-tools.sh
    ```

    This installs `xcrun` and `xcode-select` onto the system PATH. All other
    ported tools are resolved automatically at runtime.

2. Add the dependency to your `MODULE.bazel`:

    ```starlark
    bazel_dep(name = "rules_applecross", version = "0.0.3")
    git_override(
        module_name = "rules_applecross",
        remote = "https://github.com/apple-cross-toolchain/rules_applecross.git",
        commit = "<commit>",
    )
    ```

3. Configure the toolchain in `MODULE.bazel`:

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

4. Add the following to your `.bazelrc`:

    ```
    build --xcode_version_config=@rules_applecross//xcode_config:host_xcodes
    ```

5. Fetch the toolchain and configure the developer directory:

    ```
    bazel fetch //...
    sudo tools/set-developer-dir.sh
    ```

    This points `xcode-select` at the extracted developer directory. No
    `DEVELOPER_DIR` or `PATH` environment variables are needed after this.

6. Build:

    ```
    bazel build //examples/ios/HelloWorldSwiftUI:HelloWorld
    ```

## Notes

- Xcode SDK archives are not publicly available. If you have access to macOS, you
  can build one yourself by running `tools/package-sdks.sh`.

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
            "container-image": "<your-docker-image-with-xcrun-and-xcode-select-installed>",
        },
    )
    ```

2. Add the following to your `.bazelrc`:

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

3. Build with `--config=remote`:

    ```
    bazel build --config=remote //examples/ios/HelloWorldSwiftUI:HelloWorld
    ```
