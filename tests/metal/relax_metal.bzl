"""Builds its srcs for the iOS simulator with --//toolchain:relax_metal on."""

_SETTINGS = ["//command_line_option:ios_multi_cpus", "//toolchain:relax_metal"]

def _relax_metal_impl(_settings, _attr):
    return {
        "//command_line_option:ios_multi_cpus": ["sim_arm64"],
        "//toolchain:relax_metal": True,
    }

_relax_metal = transition(
    implementation = _relax_metal_impl,
    inputs = [],
    outputs = _SETTINGS,
)

def _relax_metal_filegroup_impl(ctx):
    files = depset(transitive = [src[DefaultInfo].files for src in ctx.attr.srcs])
    return [DefaultInfo(files = files, runfiles = ctx.runfiles(transitive_files = files))]

relax_metal_filegroup = rule(
    implementation = _relax_metal_filegroup_impl,
    attrs = {"srcs": attr.label_list(cfg = _relax_metal, mandatory = True)},
)
