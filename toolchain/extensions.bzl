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
            }

            # Only pass llvm_*/swift_* when explicitly set, so the
            # repository rule's built-in defaults are used otherwise.
            if config.llvm_path:
                kwargs["llvm_path"] = config.llvm_path
            if config.llvm_urls:
                kwargs["llvm_urls"] = config.llvm_urls
            if config.llvm_sha256:
                kwargs["llvm_sha256"] = config.llvm_sha256
            if config.llvm_strip_prefix:
                kwargs["llvm_strip_prefix"] = config.llvm_strip_prefix
            if config.swift_path:
                kwargs["swift_path"] = config.swift_path
            if config.swift_urls:
                kwargs["swift_urls"] = config.swift_urls
            if config.swift_sha256:
                kwargs["swift_sha256"] = config.swift_sha256
            if config.swift_strip_prefix:
                kwargs["swift_strip_prefix"] = config.swift_strip_prefix

            _apple_cross_toolchain_rule(**kwargs)

_configure_tag = tag_class(
    attrs = {
        "name": attr.string(mandatory = True),
        "apple_sdk_path": attr.string(),
        "apple_sdk_urls": attr.string_list(),
        "apple_sdk_sha256": attr.string(),
        "apple_sdk_strip_prefix": attr.string(),
        "apple_sdk_archive_type": attr.string(),
        "llvm_path": attr.string(),
        "llvm_urls": attr.string_list(),
        "llvm_sha256": attr.string(),
        "llvm_strip_prefix": attr.string(),
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
