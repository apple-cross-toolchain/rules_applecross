load("@build_bazel_rules_apple//apple:ios.bzl", "ios_application")
load("@build_bazel_rules_apple//apple:versioning.bzl", "apple_bundle_version")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_library")
load("@rules_cc//cc:cc_binary.bzl", "cc_binary")
load("@rules_cc//cc:cc_library.bzl", "cc_library")
load("@rules_cc//cc:objc_library.bzl", "objc_library")

cc_library(
    name = "hello_c",
    srcs = ["hello.c"],
    hdrs = ["hello.h"],
)

cc_library(
    name = "hello_cc",
    srcs = ["hello.cc"],
    hdrs = ["hello_cc.h"],
    deps = [":hello_c"],
)

cc_binary(
    name = "hello_bin",
    srcs = ["main.c"],
    deps = [":hello_c"],
)

cc_binary(
    name = "hello_cc_bin",
    srcs = ["main.cc"],
    deps = [":hello_cc"],
)

cc_binary(
    name = "bcr_cc_deps_bin",
    srcs = ["bcr_deps.cc"],
    deps = [
        "@fmt",
        "@zlib",
    ],
)

cc_binary(
    name = "bcr_complex_cc_deps_bin",
    srcs = ["bcr_complex_deps.cc"],
    deps = [
        "@fmt",
        "@libpng",
        "@sqlite3",
        "@zlib",
    ],
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

swift_library(
    name = "hello_uikit_swift",
    srcs = ["HelloUIKit.swift"],
    module_name = "HelloUIKit",
)

swift_library(
    name = "hello_ios_app_lib",
    srcs = ["main.swift"],
    data = ["Greeting.txt"],
    linkopts = ["-Wl,-framework,UIKit"],
    module_name = "HelloIOSApp",
)

apple_bundle_version(
    name = "hello_ios_app_version",
    build_version = "1",
)

ios_application(
    name = "hello_ios_app",
    bundle_id = "com.example.HelloIOSApp",
    families = ["iphone"],
    features = [
        "apple.skip_codesign_simulator_bundles",
        "disable_legacy_signing",
    ],
    infoplists = ["HelloIOSAppInfo.plist"],
    minimum_os_version = "16.0",
    version = ":hello_ios_app_version",
    deps = [":hello_ios_app_lib"],
)
