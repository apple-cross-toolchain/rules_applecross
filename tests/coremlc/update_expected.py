#!/usr/bin/env python3
"""Writes expected/<case>/ from Xcode's own coremlc (on a Mac with the Xcode
the tests are against): compiled/ (`coremlc compile`, without its partial
Info.plist), swift/ (`coremlc generate --public-access --language Swift`)
and objc/ (`coremlc generate`), each where the case had it before, or
refused.txt (Xcode's error) for a model Xcode refuses. Run from anywhere:

    python3 tests/coremlc/update_expected.py
"""

import glob
import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))


def restore_empty_items(package):
    """A package's Core ML weights directory, when its manifest lists one
    that is empty (an immediate-value model's), which git doesn't keep:
    Xcode's coremlc refuses a package whose manifest names an item that
    isn't there."""
    manifest = os.path.join(package, "Manifest.json")
    if not os.path.isfile(manifest):
        return
    with open(manifest) as f:
        items = json.load(f).get("itemInfoEntries", {})
    for item in items.values():
        # Only Core ML's weights directory: another missing item stays
        # missing, as the case means it to be (pkg_missing_notes).
        if item.get("author") != "com.apple.CoreML" or os.path.basename(item["path"]) != "weights":
            continue
        path = os.path.join(package, "Data", item["path"])
        if not os.path.exists(path):
            os.makedirs(path)


def coremlc(*args):
    return subprocess.run(["xcrun", "coremlc"] + list(args), capture_output=True, text=True)


def main():
    for case in sorted(os.listdir(os.path.join(HERE, "cases"))):
        package = (glob.glob(os.path.join(HERE, "cases", case, "*.mlpackage"))
                   + glob.glob(os.path.join(HERE, "cases", case, "*.mlmodel")))[0]
        restore_empty_items(package)
        expected = os.path.join(HERE, "expected", case)
        had = set(os.listdir(expected)) if os.path.isdir(expected) else {"compiled", "swift", "objc"}
        shutil.rmtree(expected, ignore_errors=True)
        os.makedirs(expected)
        with tempfile.TemporaryDirectory() as tmp:
            out = os.path.join(tmp, "compiled")
            os.makedirs(out)
            result = coremlc("compile", package, out, "--output-partial-info-plist", os.path.join(tmp, "partial.plist"))
            if result.returncode != 0:
                error = next((line for line in (result.stderr + result.stdout).splitlines() if "error" in line.lower()), result.stderr.strip())
                with open(os.path.join(expected, "refused.txt"), "w") as f:
                    f.write(error.replace(os.path.dirname(package) + "/", "") + "\n")
            elif "compiled" in had or "refused.txt" in had:
                shutil.copytree(out, os.path.join(expected, "compiled"))
        for language, args in (("swift", ["--public-access", "--language", "Swift"]), ("objc", [])):
            if language not in had:
                continue
            with tempfile.TemporaryDirectory() as tmp:
                result = coremlc("generate", package, tmp, *args)
                if result.returncode != 0:
                    sys.exit("%s %s: Xcode's coremlc generate failed:\n%s" % (case, language, result.stderr))
                shutil.copytree(tmp, os.path.join(expected, language))
        print(case, ", ".join(sorted(os.listdir(expected))))


if __name__ == "__main__":
    main()
