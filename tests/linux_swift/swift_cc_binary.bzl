"""A binary built with @apple_cross_toolchain//linux:cc_toolchain registered
first, as a project whose own C++ toolchain can't link Swift registers it.
This repository's (@llvm's, hermetic, without a sysroot) can't."""

def _swift_cc_impl(settings, _attr):
    return {"//command_line_option:extra_toolchains": [
        str(Label("@apple_cross_toolchain//linux:cc_toolchain")),
    ] + settings["//command_line_option:extra_toolchains"]}

_swift_cc = transition(
    implementation = _swift_cc_impl,
    inputs = ["//command_line_option:extra_toolchains"],
    outputs = ["//command_line_option:extra_toolchains"],
)

def _swift_cc_binary_impl(ctx):
    binary = ctx.attr.binary[0][DefaultInfo]
    return [DefaultInfo(files = binary.files, runfiles = binary.default_runfiles)]

swift_cc_binary = rule(
    implementation = _swift_cc_binary_impl,
    attrs = {
        "binary": attr.label(cfg = _swift_cc, executable = True),
        "_allowlist_function_transition": attr.label(default = "@bazel_tools//tools/allowlists/function_transition_allowlist"),
    },
)
