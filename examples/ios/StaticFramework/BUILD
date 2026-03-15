load("@bazel_skylib//rules:build_test.bzl", "build_test")
load("@build_bazel_rules_apple//apple:ios.bzl", "ios_static_framework")
load("@rules_cc//cc:objc_library.bzl", "objc_library")

licenses(["notice"])

objc_library(
    name = "ObjCLib",
    srcs = ["Sources/MyFramework.m"],
    hdrs = ["Sources/MyFramework.h"],
    tags = ["manual"],
)

ios_static_framework(
    name = "ObjCStaticFramework",
    bundle_name = "MyFramework",
    hdrs = ["Sources/MyFramework.h"],
    minimum_os_version = "15.0",
    deps = [":ObjCLib"],
)

build_test(
    name = "ExamplesBuildTest",
    targets = [":ObjCStaticFramework"],
)
