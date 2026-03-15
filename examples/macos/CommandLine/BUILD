load("@bazel_skylib//rules:build_test.bzl", "build_test")
load("@build_bazel_rules_apple//apple:macos.bzl", "macos_command_line_application")
load("@build_bazel_rules_apple//apple:versioning.bzl", "apple_bundle_version")
load("@rules_cc//cc:objc_library.bzl", "objc_library")

licenses(["notice"])

objc_library(
    name = "Sources",
    srcs = ["Sources/main.m"],
    tags = ["manual"],
)

apple_bundle_version(
    name = "CommandLineVersion",
    build_version = "1.0",
)

macos_command_line_application(
    name = "CommandLine",
    bundle_id = "com.example.command-line",
    infoplists = [":Info.plist"],
    minimum_os_version = "14.0",
    version = ":CommandLineVersion",
    deps = [":Sources"],
)

build_test(
    name = "ExamplesBuildTest",
    targets = [":CommandLine"],
)
