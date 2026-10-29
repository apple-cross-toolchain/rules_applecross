"""Shared by the Icon Composer golden test and its update script: the cases,
the actool command for each platform, and the digest of an output."""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "tools"))
from xcode_equivalence import icon_output_digest  # noqa: E402

# Each platform's arguments, as rules_apple passes them (minimum OS 26).
PLATFORMS = {
    "ios": ["--platform", "iphoneos", "--compress-pngs", "--target-device", "iphone", "--target-device", "ipad"],
    "sim": ["--platform", "iphonesimulator", "--target-device", "iphone", "--target-device", "ipad"],
    "mac": ["--platform", "macosx", "--target-device", "mac"],
}
# The real documents also for devices; the rest for the simulator and macOS.
DEVICE_CASES = {"C-Visor", "C-Isomer", "C-app_icon"}


# Cases with an asset catalog beside the document, for one platform.
ONLY = {"combo-catalog": ["sim", "ios"], "combo-catalog-mac": ["mac"], "combo-catalog-mac-hc": ["mac"]}


def cases():
    """(case, its document, its catalogs, platforms) of each case."""
    for case in sorted(os.listdir(os.path.join(HERE, "cases"))):
        files = sorted(os.listdir(os.path.join(HERE, "cases", case)))
        icon = [f for f in files if f.endswith(".icon")][0]
        catalogs = [os.path.join(HERE, "cases", case, f) for f in files if f.endswith(".xcassets")]
        platforms = ONLY.get(case) or ["sim", "mac"] + (["ios"] if case in DEVICE_CASES else [])
        yield case, os.path.join(HERE, "cases", case, icon), catalogs, platforms


def compile_icon(tool, icon, catalogs, platform, out, env=None):
    """Runs `tool` (a command list) on `catalogs` and `icon`, as rules_apple
    passes them; returns (status, output)."""
    name = os.path.basename(icon)[:-len(".icon")]
    args = tool + ["--compile", out] + PLATFORMS[platform] + [
        "--minimum-deployment-target", "26.0", "--app-icon", name,
        "--output-format", "human-readable-text", "--output-partial-info-plist", os.path.join(out, "partial.plist")] + catalogs + [icon]
    r = subprocess.run(args, capture_output=True, text=True, env=env)
    return r.returncode, r.stdout + r.stderr


def digests(out):
    """Every output file's digest, by path within `out`."""
    result = {}
    for directory, _, files in os.walk(out):
        for f in files:
            path = os.path.join(directory, f)
            with open(path, "rb") as fh:
                result[os.path.relpath(path, out)] = icon_output_digest(path, fh.read())
    return result
