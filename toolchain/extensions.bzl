"""Module extension for configuring the Apple cross-compilation toolchain."""

load("//toolchain:apple_cross_toolchain.bzl", _apple_cross_toolchain_rule = "apple_cross_toolchain")

def _apple_cross_toolchain_impl(module_ctx):
    created = {}
    for mod in module_ctx.modules:
        for config in mod.tags.configure:
            if config.name in created:
                continue
            created[config.name] = True
            kwargs = {
                "name": config.name,
                "apple_sdk_path": config.apple_sdk_path,
                "apple_sdk_urls": config.apple_sdk_urls,
                "apple_sdk_sha256": config.apple_sdk_sha256,
                "apple_sdk_strip_prefix": config.apple_sdk_strip_prefix,
                "apple_sdk_archive_type": config.apple_sdk_archive_type,
                "swift_tools": config.swift_tools,
            }

            _apple_cross_toolchain_rule(**kwargs)

_configure_tag = tag_class(
    attrs = {
        "name": attr.string(mandatory = True),
        "apple_sdk_path": attr.string(),
        "apple_sdk_urls": attr.string_list(),
        "apple_sdk_sha256": attr.string(),
        "apple_sdk_strip_prefix": attr.string(),
        "apple_sdk_archive_type": attr.string(),
        "swift_tools": attr.string(mandatory = True),
    },
)

apple_cross_toolchain = module_extension(
    implementation = _apple_cross_toolchain_impl,
    tag_classes = {
        "configure": _configure_tag,
    },
)
