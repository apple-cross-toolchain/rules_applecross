load("@apple_support//xcode:xcode_config.bzl", "xcode_config")
load("@apple_support//xcode:xcode_version.bzl", "xcode_version")

package(default_visibility = ["//visibility:public"])

xcode_version(
    name = "version26_1_1",
    aliases = [
        "26.1",
        "26.1.1",
    ],
    default_ios_sdk_version = "26.1",
    default_macos_sdk_version = "26.1",
    default_tvos_sdk_version = "26.1",
    default_visionos_sdk_version = "26.1",
    default_watchos_sdk_version = "26.1",
    version = "26.1.1",
)

xcode_config(
    name = "host_xcodes",
    default = ":version26_1_1",
    versions = [
        ":version26_1_1",
    ],
)
