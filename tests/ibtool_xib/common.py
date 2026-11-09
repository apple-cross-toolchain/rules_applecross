"""Shared by the xib golden test and its update script: the cases, the
ibtool command rules_apple runs for each, and the digest of its nib."""

import hashlib
import os
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))


def cases():
    """(case, its xib or storyboard) of each case."""
    for name in sorted(os.listdir(os.path.join(HERE, "cases"))):
        for ext in (".xib", ".storyboard"):
            if name.endswith(ext):
                yield name[:-len(ext)], os.path.join(HERE, "cases", name)


def target(case):
    """The deployment target: iOS 18, or the one a case's name ends with
    (`-ios16`)."""
    base, _, version = case.rpartition("-ios")
    return version + ".0" if base and version.isdigit() else "18.0"


def compile_xib(tool, xib, out, env=None):
    """Runs `tool` (a command list) on `xib` as rules_apple's compile_xib
    does (on a storyboard, as its compile_storyboard does, into `out`'s
    directory), for an iPhone and iPad app of the case's target, or an
    AppKit one for a Mac app of macOS 14; returns (status, output)."""
    case = os.path.splitext(os.path.basename(xib))[0]
    output = ["--compilation-directory", os.path.dirname(out)] if xib.endswith(".storyboard") else ["--compile", out]
    # An AppKit document compiles for a Mac app of macOS 14.
    with open(xib) as f:
        appkit = 'targetRuntime="MacOSX.Cocoa"' in f.read()
    devices = ["--minimum-deployment-target", "14.0", "--target-device", "mac"] if appkit else [
        "--minimum-deployment-target", target(case), "--target-device", "iphone", "--target-device", "ipad"]
    args = tool + output + devices + ["--module", "App", "--output-format", "human-readable-text", xib]
    r = subprocess.run(args, capture_output=True, text=True, env=env)
    return r.returncode, r.stdout + r.stderr


def sha256(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def digest(path):
    """The nib's SHA-256; for a xib deploying before iOS 8, compiled into a
    nib directory for each idiom beside it (`X~iphone.nib/runtime.nib`,
    ...), or a storyboard (`X.storyboardc/...`), a line for each of their
    files, its path and SHA-256, sorted.
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
