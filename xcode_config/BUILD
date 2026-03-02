load("@build_bazel_apple_support//xcode:xcode_config.bzl", "xcode_config")
load("@build_bazel_apple_support//xcode:xcode_version.bzl", "xcode_version")

package(default_visibility = ["//visibility:public"])

xcode_version(
    name = "version26_2_17C52",
    aliases = [
        "26.2",
        "26.2.0",
    ],
    default_ios_sdk_version = "26.2",
    default_macos_sdk_version = "26.2",
    default_tvos_sdk_version = "26.2",
    default_visionos_sdk_version = "26.2",
    default_watchos_sdk_version = "26.2",
    version = "26.2.0.17C52",
)

xcode_config(
    name = "host_xcodes",
    default = ":version26_2_17C52",
    versions = [
        ":version26_2_17C52",
    ],
)
