"""Shared by the asset catalog golden test and its update script."""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "tools"))
from xcode_equivalence import actool_diagnostics as diagnostics, differences  # noqa: E402,F401

# Each platform's arguments, as rules_apple passes them.
PLATFORMS = {
    "mac": ["--platform", "macosx", "--target-device", "mac", "--minimum-deployment-target", "14.0"],
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
    r = subprocess.run(args, capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr
