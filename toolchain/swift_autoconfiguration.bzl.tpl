load(
    "@build_bazel_rules_swift//swift/internal:feature_names.bzl",
    "SWIFT_FEATURE_MODULE_MAP_NO_PRIVATE_HEADERS",
)

def _swift_autoconfiguration_impl(repository_ctx):
    """Creates BUILD targets for the Swift toolchain on macOS using Xcode.

    Args:
      repository_ctx: The repository rule context.
    """
    feature_values = [
        # TODO: This should be removed so that private headers can be used with
        # explicit modules, but the build targets for CgRPC need to be cleaned
        # up first because they contain C++ code.
        SWIFT_FEATURE_MODULE_MAP_NO_PRIVATE_HEADERS,
    ]

    repository_ctx.file(
        "BUILD",
        """\
load(
    "@%{repo_name}//:swift_toolchain.bzl",
    "swift_toolchain",
)

package(default_visibility = ["//visibility:public"])

swift_toolchain(
    name = "toolchain",
    features = [{feature_list}],
    swift_executable = "@%{repo_name}//:swift_executable",
)
""".format(
            feature_list = ", ".join([
                '"{}"'.format(feature)
                for feature in feature_values
            ]),
        ),
    )

swift_autoconfiguration = repository_rule(
    environ = ["PATH"],
    implementation = _swift_autoconfiguration_impl,
)
