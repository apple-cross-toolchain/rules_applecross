load("@rules_swift//swift:swift_library.bzl", "swift_library")
load("@rules_swift//swift:swift_package_configuration.bzl", "swift_package_configuration")
load(":compile_flags_test.bzl", "compile_flags_test")

licenses(["notice"])

# Named in MODULE.bazel's configure() (swift_package_configurations), with
# swift_copts' -DAPPLECROSS_SWIFT_COPTS.
swift_package_configuration(
    name = "warnings_as_errors",
    configured_features = ["swift.treat_warnings_as_errors"],
    packages = ["//tests/swift_toolchain_attrs/configured"],
    # The toolchain repository's targets name it.
    visibility = ["//visibility:public"],
)

compile_flags_test(
    name = "configured_test",
    expected_flags = [
        "-DAPPLECROSS_SWIFT_COPTS",
        "-warnings-as-errors",
    ],
    target_under_test = "//tests/swift_toolchain_attrs/configured:Configured",
)

compile_flags_test(
    name = "unconfigured_test",
    expected_flags = ["-DAPPLECROSS_SWIFT_COPTS"],
    target_under_test = ":Unconfigured",
    unexpected_flags = ["-warnings-as-errors"],
)

swift_library(
    name = "Unconfigured",
    srcs = ["Unconfigured.swift"],
    module_name = "Unconfigured",
    tags = ["manual"],
)
