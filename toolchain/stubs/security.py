#!/usr/bin/env python3
"""Minimal security stub for cross-compilation on Linux."""

import subprocess
import sys


def main() -> int:
    args = sys.argv[1:]
    if not args:
        return 0

    if args[0] == "cms" and "-D" in args and "-i" in args:
        idx = args.index("-i")
        if idx + 1 < len(args):
            mobileprovision = args[idx + 1]
            try:
                result = subprocess.run(
                    [
                        "openssl",
                        "smime",
                        "-inform",
                        "DER",
                        "-verify",
                        "-noverify",
                        "-in",
                        mobileprovision,
                    ],
                    capture_output=True,
                )
                if result.returncode == 0 and result.stdout:
                    sys.stdout.buffer.write(result.stdout)
                    return 0
            except FileNotFoundError:
                pass

            print('<?xml version="1.0" encoding="UTF-8"?>')
            print(
                '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" '
                '"http://www.apple.com/DTDs/PropertyList-1.0.dtd">'
            )
            print('<plist version="1.0"><dict>')
            print("<key>DeveloperCertificates</key><array></array>")
            print("</dict></plist>")
            return 0

    if args[0] == "find-identity":
        print(
            '  1) AABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDD '
            '"Apple Development: Cross Compilation"'
        )
        print("     1 valid identities found")
        return 0

    return 0


if __name__ == "__main__":
    sys.exit(main())
