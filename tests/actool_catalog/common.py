"""Shared by the asset catalog golden test and its update script."""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "tools"))
from xcode_equivalence import differences  # noqa: E402,F401

# Each platform's arguments, as rules_apple passes them.
PLATFORMS = {
    "mac": ["--platform", "macosx", "--target-device", "mac", "--minimum-deployment-target", "14.0"],
}


def cases():
    """(case, its catalog, the --app-icon it names, platforms) of each case."""
    for case in sorted(os.listdir(os.path.join(HERE, "cases"))):
        catalog = os.path.join(HERE, "cases", case, "Cat.xcassets")
        icons = [f[:-len(".appiconset")] for f in os.listdir(catalog) if f.endswith(".appiconset")]
        yield case, catalog, icons[0] if icons else None, ["mac"]


def compile_catalog(tool, catalog, icon, platform, out):
    """Runs `tool` (a command list) on `catalog`; returns (status, output)."""
    args = tool + ["--compile", out] + PLATFORMS[platform] + (["--app-icon", icon] if icon else []) + [
        "--output-format", "human-readable-text", "--output-partial-info-plist", os.path.join(out, "partial.plist"), catalog]
    r = subprocess.run(args, capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr
