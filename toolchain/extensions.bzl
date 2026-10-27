"""Module extension for configuring the Apple cross-compilation toolchain."""

load("@rules_swift//swift/internal/extensions:standalone_toolchain.bzl", "standalone_toolchain")
load("//toolchain:apple_cross_toolchain.bzl", _apple_cross_toolchain_rule = "apple_cross_toolchain")

def _apple_cross_toolchain_impl(module_ctx):
    # swift.org's toolchains, one repository per platform (<name>_<platform>),
    # as rules_swift's `swift` extension makes them. That extension serves
    # only the root module, which may need it for its own toolchains.
    swift_repos = {}
    for mod in module_ctx.modules:
        for swift in mod.tags.swift:
            for platform, sha256 in swift.platform_sha256.items():
                name = "{}_{}".format(swift.name, platform)
                if name in swift_repos:
                    continue
                swift_repos[name] = True
                standalone_toolchain(
                    name = name,
                    platform = platform,
                    sha256 = sha256,
                    swift_version = swift.swift_version,
                )

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
                "xcode_version": config.xcode_version,
                "xcode_xip_urls": config.xcode_xip_urls,
                "local_xcode": config.local_xcode,
                "ported_tools_archives": config.ported_tools_archives,
                "ported_tools_manifest": config.ported_tools_manifest,
                "ported_tools_urls": config.ported_tools_urls,
                "swift_tools": config.swift_tools,
                "swift_tools_macos": config.swift_tools_macos,
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
        "xcode_version": attr.string(
            doc = "An Xcode release from toolchain/xcode_versions.bzl, such " +
                  "as \"27.0\", pinning the SHA-256 of xcode_xip_urls and " +
                  "apple_sdk_urls. The staged SDKs must be from that release.",
        ),
        "xcode_xip_urls": attr.string_list(
            doc = "URLs of xcode_version's .xip (a mirror, a bucket, or a " +
                  "file:// URL to a local copy), from which the SDKs are " +
                  "extracted when no SDK archive is configured.",
        ),
        "local_xcode": attr.bool(
            default = False,
            doc = "Read the Apple SDKs from this host's Xcode: installed on " +
                  "macOS, mounted on Linux. A host without one uses the " +
                  "archive source, so one MODULE.bazel can name both.",
        ),
        "ported_tools_archives": attr.string_dict(
            doc = "Workspace-relative ported Apple tools keyed by Linux " +
                  "CPU (\"x86_64\", \"aarch64\"): a directory of " +
                  "<tool>-linux-<cpu>.tar.zst archives or a single archive. " +
                  "CPUs without one download the ones this repository releases.",
        ),
        "ported_tools_manifest": attr.label(
            doc = "The ported tools to download instead of this repository's " +
                  "release: a file of \"<sha256>  <tool>-linux-<cpu>.tar.zst\" " +
                  "lines like toolchain/ported_tools.sha256.",
        ),
        "ported_tools_urls": attr.string_list(
            doc = "URL templates for the ported tools, tried in order, with " +
                  "{name} and {sha256}. Defaults to this repository's release.",
        ),
        "swift_tools": attr.string_dict(
            mandatory = True,
            doc = "Labels of rules_swift swift_tools targets keyed by Linux " +
                  "CPU (\"x86_64\", \"aarch64\").",
        ),
        "swift_tools_macos": attr.string(
            doc = "Label of a rules_swift swift_tools target from swift.org's " +
                  "macOS Swift, matching swift_tools' version. When set, Swift " +
                  "compiles that land on a macOS execution platform use it " +
                  "rather than Xcode's swiftc, which rejects modules built by " +
                  "swift_tools.",
        ),
    },
)

_swift_tag = tag_class(
    attrs = {
        "name": attr.string(mandatory = True),
        "platform_sha256": attr.string_dict(
            mandatory = True,
            doc = "SHA-256 of swift.org's archive per rules_swift platform " +
                  "name (\"ubuntu24.04\", \"ubuntu24.04-aarch64\", \"xcode\").",
        ),
        "swift_version": attr.string(mandatory = True),
    },
    doc = "swift.org's Swift for the toolchain's compilers, as repositories " +
          "<name>_<platform> holding rules_swift's swift_tools targets.",
)

apple_cross_toolchain = module_extension(
    implementation = _apple_cross_toolchain_impl,
    tag_classes = {
        "configure": _configure_tag,
        "swift": _swift_tag,
    },
)
