load("@bazel_skylib//rules:build_test.bzl", "build_test")
load("@build_bazel_rules_apple//apple:macos.bzl", "macos_application", "macos_unit_test")
load("@build_bazel_rules_apple//apple:versioning.bzl", "apple_bundle_version")
load("@rules_swift//swift:swift.bzl", "swift_library")

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

swift_library(
    name = "LogicTestsLib",
    testonly = True,
    srcs = ["Tests/LogicTests.swift"],
    module_name = "HelloWorldSwiftLogicTests",
    tags = ["manual"],
)

# A logic test (no host): its bundle runs under `xctest`.
macos_unit_test(
    name = "LogicTests",
    minimum_os_version = "14.0",
    deps = [":LogicTestsLib"],
)

swift_library(
    name = "HostedTestsLib",
    testonly = True,
    srcs = ["Tests/HostedTests.swift"],
    module_name = "HelloWorldSwiftHostedTests",
    tags = ["manual"],
)

# A hosted test: injected into the running app.
macos_unit_test(
    name = "HostedTests",
    minimum_os_version = "14.0",
    test_host = ":HelloWorldSwift",
    deps = [":HostedTestsLib"],
)

build_test(
    name = "ExamplesBuildTest",
    targets = [
        ":HelloWorldSwift",
        ":HostedTests",
        ":LogicTests",
    ],
)
