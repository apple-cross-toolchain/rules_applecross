#!/usr/bin/env python3
"""Print the Version field from an SDKSettings.json file."""

import json
import sys


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: read_sdk_settings_version.py <SDKSettings.json>", file=sys.stderr)
        return 2
    with open(sys.argv[1], "r") as f:
        print(json.load(f)["Version"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
