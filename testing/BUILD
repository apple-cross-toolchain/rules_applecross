load(
    "@build_bazel_rules_apple//apple/testing/default_runner:ios_xctestrun_runner.bzl",
    "ios_xctestrun_runner",
)
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
    toolchain_type = "@bazel_tools//tools/test:default_test_toolchain_type",
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
