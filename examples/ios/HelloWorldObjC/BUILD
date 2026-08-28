load("@bazel_skylib//rules:build_test.bzl", "build_test")
load("@build_bazel_rules_apple//apple:ios.bzl", "ios_application")
load("@build_bazel_rules_apple//apple:resources.bzl", "apple_precompiled_resource_bundle")
load("@build_bazel_rules_apple//apple:versioning.bzl", "apple_bundle_version")
load("@build_bazel_rules_apple//apple:xcarchive.bzl", "xcarchive")
load("@rules_cc//cc:objc_library.bzl", "objc_library")

licenses(["notice"])

objc_library(
    name = "Sources",
    srcs = [
        "Sources/AppDelegate.h",
        "Sources/AppDelegate.m",
        "Sources/main.m",
    ],
    data = [
        "Resources/Main.storyboard",
    ],
    tags = ["manual"],
)

apple_bundle_version(
    name = "HelloWorldVersion",
    build_version = "1.0",
)

ios_application(
    name = "HelloWorld",
    app_icons = ["//examples/resources:PhoneAppIcon.xcassets"],
    bundle_id = "com.example.hello-world",
    families = [
        "iphone",
        "ipad",
    ],
    infoplists = [":Info.plist"],
    launch_storyboard = "//examples/resources:Launch.storyboard",
    minimum_os_version = "15.0",
    version = ":HelloWorldVersion",
    deps = [
        ":PrecompiledResources",
        ":Sources",
    ],
)

build_test(
    name = "ExamplesBuildTest",
    targets = [":HelloWorld"],
)

# Manual because it has no Apple rule transition of its own: built straight
# from a wildcard it takes the host platform, which on Linux carries no Apple
# constraint for it to read. It is built through :HelloWorld either way.
apple_precompiled_resource_bundle(
    name = "PrecompiledResources",
    bundle_name = "PrecompiledResources",
    infoplists = ["PrecompiledResources-Info.plist"],
    resources = ["Resources/Main.storyboard"],
    tags = ["manual"],
)

xcarchive(
    name = "HelloWorldArchive",
    bundle = ":HelloWorld",
    tags = ["manual"],
)
