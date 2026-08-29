load("@bazel_skylib//rules:common_settings.bzl", "string_flag")
load("//toolchain:apple_bundling_toolchain.bzl", "apple_bundling_toolchain")

package(default_visibility = ["//visibility:public"])

# Required by every patched rules_apple bundling rule, resource aspect and test
# rule. Those rules can only resolve to an execution platform that has an
# instance of this type registered, which is what keeps their actions -- the
# final link, actool/ibtool, plists, codesign, dSYMs, tests -- off the Linux
# executors when the build wants real Xcode tools for them.
toolchain_type(name = "apple_bundling_toolchain_type")

# The marker itself; the two toolchain() targets below are what get registered.
apple_bundling_toolchain(name = "apple_bundling_marker")

# Where Apple apps get bundled. "default" leaves the Linux instance below
# registered, so bundling follows the ordinary execution platform order.
# "macos" withdraws it, leaving macOS the only platform that can bundle while
# compiles still resolve to Linux first.
#
# This says macOS, not local. The macOS instance below constrains nothing but
# the operating system, so any macOS execution platform satisfies it: the
# machine running the build, as --config=remote_compile uses, or a macOS
# executor pool if one is available.
string_flag(
    name = "apple_bundling",
    build_setting_default = "default",
    values = [
        "default",
        "macos",
    ],
)

config_setting(
    name = "apple_bundling_default",
    flag_values = {":apple_bundling": "default"},
)

toolchain(
    name = "apple_bundling_on_macos",
    exec_compatible_with = ["@platforms//os:macos"],
    toolchain = ":apple_bundling_marker",
    toolchain_type = ":apple_bundling_toolchain_type",
)

toolchain(
    name = "apple_bundling_on_linux",
    exec_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    # Withdrawn by --//toolchain:apple_bundling=macos.
    target_settings = [":apple_bundling_default"],
    toolchain = ":apple_bundling_marker",
    toolchain_type = ":apple_bundling_toolchain_type",
)
