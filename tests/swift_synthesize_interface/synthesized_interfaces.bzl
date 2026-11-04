"""The Swift interfaces rules_swift's synthesize aspect writes for deps,
built for the iOS simulator."""

load("@rules_swift//swift:swift_synthesize_interface_aspect.bzl", "swift_synthesize_interface_aspect")

def _ios_simulator_impl(_settings, _attr):
    return {"//command_line_option:platforms": str(Label("@apple_support//platforms:ios_sim_arm64"))}

_ios_simulator = transition(
    implementation = _ios_simulator_impl,
    inputs = [],
    outputs = ["//command_line_option:platforms"],
)

def _synthesized_interfaces_impl(ctx):
    files = depset(transitive = [dep[OutputGroupInfo].swift_synthesized_interface for dep in ctx.attr.deps])
    return [DefaultInfo(files = files)]

synthesized_interfaces = rule(
    implementation = _synthesized_interfaces_impl,
    attrs = {
        "deps": attr.label_list(aspects = [swift_synthesize_interface_aspect], cfg = _ios_simulator),
    },
)
