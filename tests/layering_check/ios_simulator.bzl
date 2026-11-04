"""srcs built for the iOS simulator."""

def _ios_simulator_impl(_settings, _attr):
    return {"//command_line_option:platforms": str(Label("@apple_support//platforms:ios_sim_arm64"))}

_ios_simulator = transition(
    implementation = _ios_simulator_impl,
    inputs = [],
    outputs = ["//command_line_option:platforms"],
)

def _ios_simulator_files_impl(ctx):
    return [DefaultInfo(files = depset(transitive = [s[DefaultInfo].files for s in ctx.attr.srcs]))]

ios_simulator_files = rule(
    implementation = _ios_simulator_files_impl,
    attrs = {"srcs": attr.label_list(cfg = _ios_simulator)},
)
