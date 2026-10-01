load("@build_bazel_rules_apple//apple:ios.bzl", "ios_unit_test")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_library")
load("@rules_applecross//testing:ios_vm_test_runner.bzl", "ios_vm_test_runner")

package(default_visibility = ["//visibility:private"])

swift_library(
    name = "TestsLib",
    testonly = True,
    srcs = ["PassingTests.swift"],
    module_name = "IOSVMEndToEndTests",
    tags = ["manual"],
)

ios_vm_test_runner(
    name = "runner",
    runtime = "@ios_vm_runtime//:iphone12_ios26_5",
    tags = ["manual"],
)

ios_unit_test(
    name = "UnitTests",
    timeout = "long",
    minimum_os_version = "16.0",
    runner = ":runner",
    tags = ["manual"],
    deps = [":TestsLib"],
)
