_SETTINGS = {
    "compilation_mode": "//command_line_option:compilation_mode",
    "ios_multi_cpus": "//command_line_option:ios_multi_cpus",
    "macos_cpus": "//command_line_option:macos_cpus",
    "tvos_cpus": "//command_line_option:tvos_cpus",
    "visionos_cpus": "//command_line_option:visionos_cpus",
    "watchos_cpus": "//command_line_option:watchos_cpus",
}

def _apple_settings_impl(settings, attr):
    return {
        setting: getattr(attr, name) or settings[setting]
        for name, setting in _SETTINGS.items()
    }

_apple_settings = transition(
    implementation = _apple_settings_impl,
    inputs = _SETTINGS.values(),
    outputs = _SETTINGS.values(),
)

def _apple_transition_filegroup_impl(ctx):
    files = depset(transitive = [src[DefaultInfo].files for src in ctx.attr.srcs])
    return [DefaultInfo(files = files, runfiles = ctx.runfiles(transitive_files = files))]

apple_transition_filegroup = rule(
    implementation = _apple_transition_filegroup_impl,
    attrs = {
        "compilation_mode": attr.string(),
        "ios_multi_cpus": attr.string_list(),
        "macos_cpus": attr.string_list(),
        "srcs": attr.label_list(cfg = _apple_settings, mandatory = True),
        "tvos_cpus": attr.string_list(),
        "visionos_cpus": attr.string_list(),
        "watchos_cpus": attr.string_list(),
    },
    doc = "Builds srcs with the given Apple CPUs and compilation mode, whatever the command line says.",
)
