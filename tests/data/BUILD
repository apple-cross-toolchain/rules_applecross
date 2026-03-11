load("@rules_cc//cc:cc_binary.bzl", "cc_binary")
load("@rules_cc//cc:cc_library.bzl", "cc_library")
load("@rules_cc//cc:objc_library.bzl", "objc_library")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_library")

cc_library(
    name = "hello_c",
    srcs = ["hello.c"],
)

cc_binary(
    name = "hello_bin",
    srcs = ["main.c"],
    deps = [":hello_c"],
)

objc_library(
    name = "dummy_lib",
    srcs = ["dummy.m"],
    target_compatible_with = ["@platforms//os:macos"],
)

swift_library(
    name = "hello_swift",
    srcs = ["hello.swift"],
    module_name = "HelloSwift",
)

swift_library(
    name = "hello_foundation",
    srcs = ["hello_foundation.swift"],
    module_name = "HelloFoundation",
    target_compatible_with = ["@platforms//os:macos"],
)
