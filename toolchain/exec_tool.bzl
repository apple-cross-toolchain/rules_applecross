def _exec_tool_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.symlink(
        output = out,
        target_file = ctx.executable.binary,
        is_executable = True,
    )
    return [DefaultInfo(
        executable = out,
        files = depset([out]),
    )]

exec_tool = rule(
    implementation = _exec_tool_impl,
    attrs = {
        "binary": attr.label(
            executable = True,
            allow_single_file = True,
            cfg = "exec",
            mandatory = True,
        ),
    },
    executable = True,
)
