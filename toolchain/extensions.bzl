"""Module extension for configuring the Apple cross-compilation toolchain."""

load("//toolchain:apple_cross_toolchain.bzl", _apple_cross_toolchain_rule = "apple_cross_toolchain")

def _apple_cross_toolchain_impl(module_ctx):
    for mod in module_ctx.modules:
        for config in mod.tags.configure:
            _apple_cross_toolchain_rule(
                name = config.name,
                xcode_path = config.xcode_path,
                xcode_urls = config.xcode_urls,
                xcode_sha256 = config.xcode_sha256,
                xcode_strip_prefix = config.xcode_strip_prefix,
                xcode_archive_type = config.xcode_archive_type,
                clang_path = config.clang_path,
                clang_urls = config.clang_urls,
                clang_sha256 = config.clang_sha256,
                clang_strip_prefix = config.clang_strip_prefix,
                swift_path = config.swift_path,
                swift_urls = config.swift_urls,
                swift_sha256 = config.swift_sha256,
                swift_strip_prefix = config.swift_strip_prefix,
            )

_configure_tag = tag_class(
    attrs = {
        "name": attr.string(mandatory = True),
        "xcode_path": attr.string(),
        "xcode_urls": attr.string_list(),
        "xcode_sha256": attr.string(),
        "xcode_strip_prefix": attr.string(),
        "xcode_archive_type": attr.string(),
        "clang_path": attr.string(),
        "clang_urls": attr.string_list(),
        "clang_sha256": attr.string(),
        "clang_strip_prefix": attr.string(),
        "swift_path": attr.string(),
        "swift_urls": attr.string_list(),
        "swift_sha256": attr.string(),
        "swift_strip_prefix": attr.string(),
    },
)

apple_cross_toolchain = module_extension(
    implementation = _apple_cross_toolchain_impl,
    tag_classes = {
        "configure": _configure_tag,
    },
)
