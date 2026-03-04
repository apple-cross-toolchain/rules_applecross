"""A rule that returns an empty ToolchainInfo provider."""

empty_toolchain = rule(
    implementation = lambda ctx: [platform_common.ToolchainInfo()],
)
