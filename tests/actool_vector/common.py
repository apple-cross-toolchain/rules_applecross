"""Shared by the vector image golden test and its update script: the cases,
the actool command for each platform, and the digest of an output."""

import importlib.util
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "tools"))
from xcode_equivalence import vector_output_digest  # noqa: E402

# actool_icon's tool_env: the Xcode identity our actool is given.
_spec = importlib.util.spec_from_file_location("actool_icon_common", os.path.join(HERE, "..", "actool_icon", "common.py"))
_icon = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_icon)
tool_env = _icon.tool_env

# Each platform's arguments, as rules_apple passes them (minimum OS 26).
PLATFORMS = {
    "sim": ["--platform", "iphonesimulator", "--target-device", "iphone", "--target-device", "ipad"],
    "mac": ["--platform", "macosx", "--target-device", "mac"],
}


def cases():
    """(case, its catalog) of each case, for every platform."""
    for case in sorted(os.listdir(os.path.join(HERE, "cases"))):
        yield case, os.path.join(HERE, "cases", case, "Assets.xcassets")


def compile_catalog(tool, catalog, platform, out, env=None):
    """Runs `tool` (a command list) on `catalog`, as rules_apple passes it;
    returns (status, output)."""
    args = tool + ["--compile", out] + PLATFORMS[platform] + [
        "--minimum-deployment-target", "26.0",
        "--output-format", "human-readable-text", "--output-partial-info-plist", os.path.join(out, "partial.plist"), catalog]
    r = subprocess.run(args, capture_output=True, text=True, env=tool_env(tool, env))
    return r.returncode, r.stdout + r.stderr


def digests(out):
    """Every output file's digest, by path within `out`."""
    result = {}
    for directory, _, files in os.walk(out):
        for f in files:
            path = os.path.join(directory, f)
            with open(path, "rb") as fh:
                result[os.path.relpath(path, out)] = vector_output_digest(path, fh.read())
    return result
