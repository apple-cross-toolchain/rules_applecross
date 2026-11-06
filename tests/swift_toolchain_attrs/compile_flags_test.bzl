"""configure()'s Swift toolchain attributes, seen on a compile's command line:
swift_copts' flag, the package configuration's feature
(swift.treat_warnings_as_errors, which adds -warnings-as-errors) and the
generated header rewriter."""

load("@bazel_skylib//lib:unittest.bzl", "analysistest", "asserts")

def _compile_flags_test_impl(ctx):
    env = analysistest.begin(ctx)
    compiles = [a for a in analysistest.target_actions(env) if a.mnemonic == "SwiftCompile"]
    asserts.equals(env, 1, len(compiles), "one SwiftCompile action")
    argv = compiles[0].argv if compiles else []
    for flag in ctx.attr.expected_flags:
        found = [a for a in argv if a.startswith(flag)] if ctx.attr.match_prefix else [a for a in argv if a == flag]
        asserts.true(env, found, "{} on the command line: {}".format(flag, argv))
    for flag in ctx.attr.unexpected_flags:
        asserts.false(env, flag in argv, "{} not on the command line".format(flag))
    return analysistest.end(env)

compile_flags_test = analysistest.make(
    _compile_flags_test_impl,
    attrs = {
        "expected_flags": attr.string_list(),
        # Expected flags match arguments they begin (a flag with a path).
        "match_prefix": attr.bool(),
        "unexpected_flags": attr.string_list(),
    },
    config_settings = {
        "//command_line_option:platforms": str(Label("@apple_support//platforms:ios_sim_arm64")),
    },
)
