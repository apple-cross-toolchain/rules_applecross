"""Shared by the pngcrush golden test and its update script: the cases, the
command rules_apple's copypng runs for each, and the digest of its PNG."""

import hashlib
import os
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
ARGS = ["-q", "-iphone", "-f", "0", "-rem", "text"]


def cases(kind="cases"):
    """(case, its PNG) of each case in `kind` (cases, or refused)."""
    for name in sorted(os.listdir(os.path.join(HERE, kind))):
        if name.endswith(".png"):
            yield name[:-len(".png")], os.path.join(HERE, kind, name)


def crush(tool, png, out, env=None):
    """Runs `tool` (a command list) on `png` as rules_apple's copy_png does;
    returns (status, output)."""
    r = subprocess.run(tool + ARGS + [png, out], capture_output=True, text=True, env=env)
    return r.returncode, r.stdout + r.stderr


def digest(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()
