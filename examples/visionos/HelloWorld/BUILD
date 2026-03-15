load("@bazel_skylib//rules:build_test.bzl", "build_test")
load("@build_bazel_rules_apple//apple:visionos.bzl", "visionos_application")
load("@build_bazel_rules_apple//apple:versioning.bzl", "apple_bundle_version")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_library")

licenses(["notice"])

swift_library(
    name = "Sources",
    srcs = ["Sources/App.swift"],
    tags = ["manual"],
)

apple_bundle_version(
    name = "HelloWorldVersion",
    build_version = "1.0",
)

visionos_application(
    name = "HelloWorld",
    app_icons = ["//examples/resources:VisionAppIcon.xcassets"],
    bundle_id = "com.example.hello-world-visionos",
    infoplists = [":Info.plist"],
    minimum_os_version = "1.0",
    version = ":HelloWorldVersion",
    deps = [":Sources"],
)

build_test(
    name = "ExamplesBuildTest",
    targets = [":HelloWorld"],
)
