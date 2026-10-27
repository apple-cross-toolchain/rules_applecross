"""Staple an app's notarization ticket to it, as `xcrun stapler staple` does.

    bazel run //tools:staple -- path/to/App.app

Apple's notary service publishes a ticket for each notarized code directory
hash. This finds the app's (its main executable's SHA-256 code directory, of
the slice a Mac would run), fetches the ticket from Apple's public lookup and
writes it to Contents/CodeResources, where stapler puts it and Gatekeeper
reads it offline. Writing it changes no signature: the ticket is outside the
sealed resources.
"""

import base64
import json
import hashlib
import os
import plistlib
import struct
import sys
import urllib.request

LOOKUP = "https://api.apple-cloudkit.com/database/1/com.apple.gk.ticket-delivery/production/public/records/lookup"

CPU_TYPE_ARM64 = 0x0100000C
CPU_SUBTYPE_ARM64E = 2


def _slices(data):
    """(cputype, cpusubtype, bytes) of each Mach-O in a thin or universal file."""
    magic = struct.unpack(">I", data[:4])[0]
    if magic in (0xCAFEBABE, 0xCAFEBABF):
        count = struct.unpack(">I", data[4:8])[0]
        wide = magic == 0xCAFEBABF
        size = 32 if wide else 20
        out = []
        for i in range(count):
            entry = data[8 + size * i:8 + size * (i + 1)]
            if wide:
                cputype, cpusubtype, offset, length = struct.unpack(">IIQQ", entry[:24])
            else:
                cputype, cpusubtype, offset, length = struct.unpack(">IIII", entry[:16])
            out.append((cputype, cpusubtype, data[offset:offset + length]))
        return out
    cputype, cpusubtype = struct.unpack("<II", data[4:12])
    return [(cputype, cpusubtype, data)]


def _code_directory(macho):
    """The SHA-256 code directory of a signed Mach-O image."""
    ncmds = struct.unpack("<I", macho[16:20])[0]
    offset = 32
    for _ in range(ncmds):
        cmd, size = struct.unpack("<II", macho[offset:offset + 8])
        if cmd == 0x1D:  # LC_CODE_SIGNATURE
            dataoff, datasize = struct.unpack("<II", macho[offset + 8:offset + 16])
            sig = macho[dataoff:dataoff + datasize]
            count = struct.unpack(">I", sig[8:12])[0]
            for i in range(count):
                slot, blob = struct.unpack(">II", sig[12 + 8 * i:20 + 8 * i])
                if slot == 0 or 0x1000 <= slot < 0x1005:
                    magic, length = struct.unpack(">II", sig[blob:blob + 8])
                    cd = sig[blob:blob + length]
                    if magic == 0xFADE0C02 and cd[37] == 2:  # SHA-256
                        return cd
            raise SystemExit("the executable has no SHA-256 code directory")
        offset += size
    raise SystemExit("the executable is not signed")


def cdhash(app):
    info = plistlib.load(open(os.path.join(app, "Contents", "Info.plist"), "rb"))
    executable = os.path.join(app, "Contents", "MacOS", info["CFBundleExecutable"])
    slices = _slices(open(executable, "rb").read())
    # The slice an Apple silicon Mac runs: arm64e, else arm64, else the first.
    preferred = next((s for s in slices if s[0] == CPU_TYPE_ARM64 and s[1] & 0xFF == CPU_SUBTYPE_ARM64E), None)
    preferred = preferred or next((s for s in slices if s[0] == CPU_TYPE_ARM64), slices[0])
    return hashlib.sha256(_code_directory(preferred[2])).hexdigest()[:40]


def ticket(code_directory_hash):
    body = json.dumps({"records": [{"recordName": "2/2/" + code_directory_hash}]}).encode()
    request = urllib.request.Request(LOOKUP, data=body, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(request, timeout=60) as response:
        record = json.load(response)["records"][0]
    if "fields" not in record:
        raise SystemExit("no ticket for code directory hash {}: {}".format(
            code_directory_hash, record.get("serverErrorCode", "not notarized")))
    return base64.b64decode(record["fields"]["signedTicket"]["value"])


def main():
    if len(sys.argv) != 2:
        print("usage: bazel run //tools:staple -- <App.app>", file=sys.stderr)
        return 2
    app = os.path.join(os.environ.get("BUILD_WORKING_DIRECTORY", os.getcwd()), os.path.expanduser(sys.argv[1]))
    if not os.path.isfile(os.path.join(app, "Contents", "Info.plist")):
        print("{}: not a macOS app bundle".format(app), file=sys.stderr)
        return 1
    h = cdhash(app)
    data = ticket(h)
    with open(os.path.join(app, "Contents", "CodeResources"), "wb") as f:
        f.write(data)
    print("{}: stapled the ticket for code directory hash {} ({} bytes)".format(app, h, len(data)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
