"""A contentless toolchain that marks where Apple bundling actions may run.

rules_apple's bundling rules, its resource aspect and its test rules are patched
to require `//toolchain:apple_bundling_toolchain_type`. The toolchain carries no
tools; it exists so toolchain *resolution* picks the execution platform those
rules land on. Registering an instance for a platform declares "linking,
bundling, resource compilation and signing may happen here".
"""

apple_bundling_toolchain = rule(
    implementation = lambda ctx: [platform_common.ToolchainInfo()],
    doc = "Marker toolchain for execution platforms that can bundle Apple apps.",
)
