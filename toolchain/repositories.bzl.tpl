load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")
load(
    "@%{repo_name}//:swift_autoconfiguration.bzl",
    "swift_autoconfiguration",
)

def apple_cross_toolchain_dependencies():
    maybe(
        swift_autoconfiguration,
        name = "build_bazel_rules_swift_local_config",
    )
