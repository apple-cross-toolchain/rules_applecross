#!/usr/bin/env python3
"""Print the Xcode and per-platform SDK versions of a staged Xcode.app tree.

Emits one `key value` pair per line:

    xcode 26.6
    build 17F42
    swift 6.3.1
    sdk iPhoneOS 26.5
    ...

`swift` is the Swift that built the SDKs: the compiler version the macOS
SDK's standard library interface records.

Platforms without an SDK are omitted, so callers must tolerate missing keys.
"""

import glob
import json
import os
import plistlib
import re
import sys

PLATFORMS = [
    "MacOSX",
    "iPhoneOS",
    "iPhoneSimulator",
    "AppleTVOS",
    "AppleTVSimulator",
    "WatchOS",
    "WatchSimulator",
    "XROS",
    "XRSimulator",
]


def _sdk_version(developer_dir: str, platform: str) -> str:
    sdks_dir = os.path.join(
        developer_dir, "Platforms", platform + ".platform", "Developer", "SDKs"
    )
    settings = os.path.join(sdks_dir, platform + ".sdk", "SDKSettings.json")
    if os.path.exists(settings):
        with open(settings, "r") as f:
            version = json.load(f).get("Version", "")
        if version:
            return version

    # Fall back to the versioned directory name (e.g. iPhoneOS26.5.sdk) for
    # trees whose version-neutral symlink is missing.
    if os.path.isdir(sdks_dir):
        for name in sorted(os.listdir(sdks_dir)):
            if name.startswith(platform) and name.endswith(".sdk"):
                version = name[len(platform) : -len(".sdk")]
                if version:
                    return version
    return ""


def _sdk_swift_version(developer_dir: str) -> str:
    interfaces = sorted(glob.glob(os.path.join(
        developer_dir, "Platforms", "MacOSX.platform", "Developer", "SDKs",
        "MacOSX.sdk", "usr", "lib", "swift", "Swift.swiftmodule", "*.swiftinterface",
    )))
    for interface in interfaces:
        with open(interface, "r", errors="replace") as f:
            header = f.read(4096)
        match = re.search(r"^// swift-compiler-version: Apple Swift version ([0-9.]+)", header, re.M)
        if match:
            return match.group(1)
    return ""


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: read_xcode_versions.py <Xcode.app>", file=sys.stderr)
        return 2
    xcode_app = sys.argv[1]

    version_plist = os.path.join(xcode_app, "Contents", "version.plist")
    with open(version_plist, "rb") as f:
        version_info = plistlib.load(f)
    print("xcode", version_info["CFBundleShortVersionString"])
    if version_info.get("ProductBuildVersion"):
        print("build", version_info["ProductBuildVersion"])

    developer_dir = os.path.join(xcode_app, "Contents", "Developer")
    swift = _sdk_swift_version(developer_dir)
    if swift:
        print("swift", swift)
    for platform in PLATFORMS:
        version = _sdk_version(developer_dir, platform)
        if version:
            print("sdk", platform, version)
    return 0


if __name__ == "__main__":
    sys.exit(main())
