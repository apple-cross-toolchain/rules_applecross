"""The Objective-C header a Swift library generates, built for the iOS
simulator, as a file other targets can read."""

load("@rules_cc//cc/common:cc_info.bzl", "CcInfo")

def _ios_simulator_impl(_settings, _attr):
    return {"//command_line_option:platforms": str(Label("@apple_support//platforms:ios_sim_arm64"))}

_ios_simulator = transition(
    implementation = _ios_simulator_impl,
    inputs = [],
    outputs = ["//command_line_option:platforms"],
)

def _generated_header_impl(ctx):
    library = ctx.attr.library[0]
    headers = [h for h in library[CcInfo].compilation_context.direct_public_headers if h.basename.endswith("-Swift.h")]
    return [DefaultInfo(files = depset(headers))]

generated_header = rule(
    implementation = _generated_header_impl,
    attrs = {
        "library": attr.label(cfg = _ios_simulator, providers = [CcInfo]),
        "_allowlist_function_transition": attr.label(default = "@bazel_tools//tools/allowlists/function_transition_allowlist"),
    },
)
