"""Shared by the asset catalog golden test and its update script."""

import os
import plistlib
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "tools"))
from xcode_equivalence import actool_diagnostics as diagnostics, differences  # noqa: E402,F401


# The Xcode the expected outputs are from. This repository's actool records
# that Xcode's identity and library versions, which it reads from the
# version.plist beside DEVELOPER_DIR; off a Mac there's none, so its runs
# are given one naming this Xcode (Xcode's own runs, `xcrun actool`, use
# the Mac's).
XCODE = {"CFBundleShortVersionString": "27.2", "ProductBuildVersion": "27B52"}
_XCODE_ROOT = tempfile.mkdtemp(prefix="xcode-identity-")
os.makedirs(os.path.join(_XCODE_ROOT, "Contents", "Developer"))
with open(os.path.join(_XCODE_ROOT, "Contents", "version.plist"), "wb") as _f:
    plistlib.dump(XCODE, _f)


def tool_env(tool, env=None):
    """The environment `tool` runs in: ours is given XCODE's identity."""
    if tool[0] == "xcrun":
        return env
    env = dict(os.environ if env is None else env)
    env["DEVELOPER_DIR"] = os.path.join(_XCODE_ROOT, "Contents", "Developer")
    return env


# Each platform's arguments, as rules_apple passes them.
PLATFORMS = {
    "mac": ["--platform", "macosx", "--target-device", "mac", "--minimum-deployment-target", "14.0"],
    "sim": ["--platform", "iphonesimulator", "--target-device", "iphone", "--target-device", "ipad", "--minimum-deployment-target", "18.0"],
    "watch": ["--platform", "watchsimulator", "--target-device", "watch", "--minimum-deployment-target", "10.0"],
    "tv": ["--platform", "appletvsimulator", "--target-device", "tv", "--minimum-deployment-target", "17.0"],
    "vision": ["--platform", "xrsimulator", "--target-device", "vision", "--minimum-deployment-target", "1.0"],
}


def cases():
    """(case, its catalog, the --app-icon it names, platforms) of each case:
    those its platforms.txt lists, macOS without one."""
    for case in sorted(os.listdir(os.path.join(HERE, "cases"))):
        catalog = os.path.join(HERE, "cases", case, "Cat.xcassets")
        icons = [f[:-len(".appiconset")] for f in os.listdir(catalog) if f.endswith(".appiconset")]
        listed = os.path.join(HERE, "cases", case, "platforms.txt")
        platforms = open(listed).read().split() if os.path.exists(listed) else ["mac"]
        yield case, catalog, icons[0] if icons else None, platforms


def compile_catalog(tool, catalog, icon, platform, out):
    """Runs `tool` (a command list) on `catalog`; returns (status, output)."""
    args = tool + ["--compile", out] + PLATFORMS[platform] + (["--app-icon", icon] if icon else []) + [
        "--output-format", "human-readable-text", "--output-partial-info-plist", os.path.join(out, "partial.plist"), catalog]
    r = subprocess.run(args, capture_output=True, text=True, env=tool_env(tool))
    return r.returncode, r.stdout + r.stderr
