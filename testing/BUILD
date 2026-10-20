load(
    "@build_bazel_rules_apple//apple/testing/default_runner:ios_xctestrun_runner.bzl",
    "ios_xctestrun_runner",
)
load("@bazel_features//:features.bzl", "bazel_features")
load("//testing:empty_toolchain.bzl", "empty_toolchain")

package(default_visibility = ["//visibility:public"])

# Test toolchain that allows iOS tests to execute on the Linux host.
# The default test toolchain requires the execution platform to match
# the target platform constraints, which fails for cross-compiled iOS tests.
empty_toolchain(name = "empty_test_toolchain")

toolchain(
    name = "ios_test_toolchain",
    target_compatible_with = ["@platforms//os:ios"],
    toolchain = ":empty_test_toolchain",
    # Bazel 8 has no test toolchain type; :test_toolchain_type stands in for it.
    toolchain_type = "@bazel_tools//tools/test:default_test_toolchain_type" if bazel_features.toolchains.has_default_test_toolchain_type else ":test_toolchain_type",
)

toolchain_type(name = "test_toolchain_type")

toolchain(
    name = "default_test_toolchain",
    toolchain = ":empty_test_toolchain",
    toolchain_type = ":test_toolchain_type",
    use_target_platform_constraints = True,
)

ios_xctestrun_runner(
    name = "ios_xctestrun_ordered_runner",
    create_simulator_action = "@xctestrunner//:simulator_creator",
    random = False,
)

ios_xctestrun_runner(
    name = "ios_xctestrun_random_runner",
    create_simulator_action = "@xctestrunner//:simulator_creator",
    random = True,
)
