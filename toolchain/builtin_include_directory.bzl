"""Helpers for C/C++ builtin include directory allowlists."""

load("@bazel_skylib//rules/directory:providers.bzl", "DirectoryInfo", "create_directory_info")

def _builtin_include_directory_impl(ctx):
    path = ctx.attr.path
    if path.startswith("/"):
        fail("builtin include directory paths must be execroot-relative: {}".format(path))
    if ctx.label.workspace_root:
        path = ctx.label.workspace_root + "/" + path

    directory = create_directory_info(
        entries = {},
        human_readable = str(ctx.label),
        path = path,
        transitive_files = depset(),
    )

    return [
        directory,
        DefaultInfo(files = depset()),
    ]

builtin_include_directory = rule(
    implementation = _builtin_include_directory_impl,
    attrs = {
        "path": attr.string(mandatory = True),
    },
    provides = [DirectoryInfo],
    doc = """Declares an execroot-relative include root implied by a compiler flag.

The provider is only for rules_cc include checking. It intentionally carries no
files; action inputs must be declared separately through the cc_args data or
cc_tool data that make this include path usable.
""",
)
