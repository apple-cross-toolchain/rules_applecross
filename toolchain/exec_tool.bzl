def _exec_tool_impl(ctx):
    out = ctx.actions.declare_file(ctx.attr.out or ctx.label.name)
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
        "out": attr.string(
            doc = "The tool's file name, when it is not the target's: wrapped_clang " +
                  "decides its mode by the name it runs under.",
        ),
    },
    executable = True,
)
