def _swift_autoconfiguration_impl(repository_ctx):
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
    swift_executable = "@%{repo_name}//:swift_executable",
)
""",
    )

swift_autoconfiguration = repository_rule(
    environ = ["PATH"],
    implementation = _swift_autoconfiguration_impl,
)
