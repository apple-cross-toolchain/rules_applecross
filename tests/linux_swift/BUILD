load("@bazel_skylib//rules:build_test.bzl", "build_test")
load("@rules_python//python:py_test.bzl", "py_test")
load("@rules_swift//swift:swift_binary.bzl", "swift_binary")
load("@rules_swift//swift:swift_library.bzl", "swift_library")
load(":swift_cc_binary.bzl", "swift_cc_binary")

licenses(["notice"])

# Swift for Linux itself, as a macro plugin or other host tool is built:
# @apple_cross_toolchain//:all registers the Swift toolchain for it.
swift_library(
    name = "LinuxGreeting",
    srcs = ["Greeting.swift"],
    module_name = "LinuxGreeting",
    target_compatible_with = ["@platforms//os:linux"],
)

build_test(
    name = "linux_library_test",
    targets = [":LinuxGreeting"],
)

swift_binary(
    name = "linux_tool",
    srcs = ["main.swift"],
    tags = ["manual"],
    target_compatible_with = ["@platforms//os:linux"],
    deps = [":LinuxGreeting"],
)

# Linked with @apple_cross_toolchain//linux:cc_toolchain, run. Manual: that
# toolchain's sysroot is rules_swift's, whose Ubuntu packages and package
# index come from snapshot.ubuntu.com alone, so this fails whenever that
# service is down (as on 2026-11-06), whatever the change.
swift_cc_binary(
    name = "linux_tool_linked",
    binary = ":linux_tool",
    tags = ["manual"],
    target_compatible_with = ["@platforms//os:linux"],
)

py_test(
    name = "linux_tool_test",
    srcs = ["linux_tool_test.py"],
    args = ["$(rootpath :linux_tool_linked)"],
    data = [":linux_tool_linked"],
    tags = ["manual"],
    target_compatible_with = ["@platforms//os:linux"],
)
