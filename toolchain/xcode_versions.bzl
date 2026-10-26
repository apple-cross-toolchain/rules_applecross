"""The Xcode releases whose SDKs the toolchain can stage by version.

Each entry pins the SHA-256 of Apple's .xip and of the SDK archive
//tools:package_sdks makes from it, so either can be fetched from wherever the
project keeps a copy: a mirror, a bucket, or a file:// URL to a local one.
swift_version is the swift.org Swift that built the SDKs, which
apple_cross_toolchain.swift() must name.
"""

XCODE_VERSIONS = {
    "27.0": struct(
        build = "27A266a",
        xip_sha256 = "6a270c53a5a0c5e0ac78125342d44c3cfff2716ec390373cd5e30834d80a67c3",
        sdk_sha256 = "251c252f7f41bcd3e08655e49e182686682d705945f4a2807049b9051c606ebb",
        swift_version = "6.4.0",
    ),
}
