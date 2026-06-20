#!/usr/bin/env python3
"""Minimal xcstringtool stub for cross-compilation on Linux."""

import json
import os
import plistlib
import sys


def compile_xcstrings(args):
    output_dir = None
    input_file = None
    i = 0
    while i < len(args):
        if args[i] == "--output-directory" and i + 1 < len(args):
            output_dir = args[i + 1]
            i += 2
        else:
            input_file = args[i]
            i += 1

    if not output_dir or not input_file:
        print("Usage: xcstringtool compile --output-directory <dir> <input>", file=sys.stderr)
        return 1

    with open(input_file, "r") as f:
        data = json.load(f)

    source_lang = data.get("sourceLanguage", "en")
    strings = data.get("strings", {})
    langs = set()
    for info in strings.values():
        langs.update(info.get("localizations", {}).keys())
    if not langs:
        langs.add(source_lang)

    base_name = os.path.splitext(os.path.basename(input_file))[0]
    for lang in langs:
        lproj = os.path.join(output_dir, lang + ".lproj")
        os.makedirs(lproj, exist_ok=True)
        entries = {}
        for key, info in strings.items():
            loc = info.get("localizations", {}).get(lang, {})
            unit = loc.get("stringUnit", {})
            entries[key] = unit.get("value", key)
        out_path = os.path.join(lproj, base_name + ".strings")
        with open(out_path, "wb") as f:
            plistlib.dump(entries, f, fmt=plistlib.FMT_BINARY)

    return 0


def main() -> int:
    if len(sys.argv) < 2:
        return 1
    if sys.argv[1] == "compile":
        return compile_xcstrings(sys.argv[2:])
    print("Unknown subcommand: " + sys.argv[1], file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
