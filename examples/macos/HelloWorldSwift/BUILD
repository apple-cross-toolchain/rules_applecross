load("@bazel_skylib//rules:build_test.bzl", "build_test")
load("@build_bazel_rules_apple//apple:macos.bzl", "macos_application")
load("@build_bazel_rules_apple//apple:versioning.bzl", "apple_bundle_version")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_library")

licenses(["notice"])

swift_library(
    name = "Sources",
    srcs = ["Sources/AppDelegate.swift"],
    tags = ["manual"],
)

apple_bundle_version(
    name = "HelloWorldSwiftVersion",
    build_version = "1.0",
)

macos_application(
    name = "HelloWorldSwift",
    app_icons = ["//examples/resources:MacAppIcon.xcassets"],
    bundle_id = "com.example.hello-world-macos-swift",
    infoplists = [":Info.plist"],
    minimum_os_version = "14.0",
    version = ":HelloWorldSwiftVersion",
    deps = [":Sources"],
)

build_test(
    name = "ExamplesBuildTest",
    targets = [":HelloWorldSwift"],
)
