"""Shared by the xib golden test and its update script: the cases, the
ibtool command rules_apple runs for each, and the digest of its nib."""

import hashlib
import os
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))


def cases():
    """(case, its xib) of each case."""
    for name in sorted(os.listdir(os.path.join(HERE, "cases"))):
        if name.endswith(".xib"):
            yield name[:-len(".xib")], os.path.join(HERE, "cases", name)


def compile_xib(tool, xib, out, env=None):
    """Runs `tool` (a command list) on `xib` as rules_apple's compile_xib
    does, for an iPhone and iPad app of iOS 18; returns (status, output)."""
    args = tool + ["--compile", out, "--minimum-deployment-target", "18.0", "--target-device", "iphone",
                   "--target-device", "ipad", "--module", "App", "--output-format", "human-readable-text", xib]
    r = subprocess.run(args, capture_output=True, text=True, env=env)
    return r.returncode, r.stdout + r.stderr


def sha256(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def digest(path):
    """The nib's SHA-256; for a xib deploying before iOS 8, compiled into a
    nib directory for each idiom beside it (`X~iphone.nib/runtime.nib`,
    ...), a line for each of their files, its path and SHA-256, sorted.
    None where there's neither."""
    if os.path.isfile(path):
        return sha256(path)
    parent = os.path.dirname(path)
    lines = []
    for root, _, files in os.walk(parent):
        for name in files:
            full = os.path.join(root, name)
            lines.append("%s %s" % (os.path.relpath(full, parent), sha256(full)))
    return "\n".join(sorted(lines)) or None
