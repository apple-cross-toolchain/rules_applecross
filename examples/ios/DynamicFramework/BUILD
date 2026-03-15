load("@bazel_skylib//rules:build_test.bzl", "build_test")
load("@build_bazel_rules_apple//apple:ios.bzl", "ios_framework")
load("@rules_cc//cc:objc_library.bzl", "objc_library")

licenses(["notice"])

objc_library(
    name = "GreeterLib",
    srcs = ["Sources/Greeter.m"],
    hdrs = ["Sources/Greeter.h"],
    tags = ["manual"],
)

ios_framework(
    name = "GreeterFramework",
    bundle_id = "com.example.greeter-framework",
    bundle_name = "Greeter",
    families = [
        "iphone",
        "ipad",
    ],
    hdrs = ["Sources/Greeter.h"],
    infoplists = ["Info.plist"],
    minimum_os_version = "15.0",
    deps = [":GreeterLib"],
)

build_test(
    name = "ExamplesBuildTest",
    targets = [":GreeterFramework"],
)
