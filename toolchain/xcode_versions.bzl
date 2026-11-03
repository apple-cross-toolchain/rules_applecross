"""The Xcode releases whose SDKs the toolchain can stage by version.

Each entry pins the SHA-256 of Apple's .xip and of the SDK archive
//tools:package_sdks makes from it, so either can be fetched from wherever the
project keeps a copy: a mirror, a bucket, or a file:// URL to a local one.
swift_version is the swift.org Swift that built the SDKs. The toolchain uses
that Swift (SWIFT_ARCHIVES) unless apple_cross_toolchain.configure() names
swift_tools of its own.
"""

XCODE_VERSIONS = {
    "27.0": struct(
        build = "27A266a",
        xip_sha256 = "6a270c53a5a0c5e0ac78125342d44c3cfff2716ec390373cd5e30834d80a67c3",
        sdk_sha256 = "251c252f7f41bcd3e08655e49e182686682d705945f4a2807049b9051c606ebb",
        swift_version = "6.4.0",
    ),
    "27.2": struct(
        build = "27B52",
        xip_sha256 = "c28b05526225394fa4017d7f9919c1c898822713edca68872cb4701599f01978",
        sdk_sha256 = "b1583e60f9fd35881b5a948ce71627079877f1ced7ea231e14d3c342dd389668",
        swift_version = "6.4.2",
    ),
}

# swift.org's archives of each Swift above, by rules_swift platform name: the
# SHA-256 that rules_swift's standalone_toolchain checks.
SWIFT_ARCHIVES = {
    "6.4.0": {
        "ubuntu24.04": "69f7b2b4dbce6090b6c92194b71149a057e16269093b5a9dded38afc4a05094e",
        "ubuntu24.04-aarch64": "21eacfe571194717d269cc1e0030533bbee6131dfde82cd4f9cfbc2512e354a9",
        "xcode": "8fd03185b98fe27f54a54631c2449decf75d5b466ce8e34abbd414141063c6aa",
    },
    "6.4.2": {
        "ubuntu24.04": "2e9ebec040ad459dd89a6d093a02d8314939135104850fae4d5c0f174fb3ba8c",
        "ubuntu24.04-aarch64": "307772afcc0b9501faa4797eb4197076facf5768593a0d8d2b83615c95b0040e",
        "xcode": "0a7d8489900d21dbf5f0187cb570e646a4c0226d0ddef2cbde8c7f54dfe79736",
    },
}
