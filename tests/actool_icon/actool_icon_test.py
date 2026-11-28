#!/usr/bin/env python3
"""Icon Composer documents through this repository's actool, against Xcode
27.2's output (expected/, from update_expected.py): every file and every
rendition the same, the flattened images' pixels aside, with
--config=relax_actool_icon_composer; refused without it. And Visor's
document strict with a Flattened folder (tools/icon_flattened.py's, here of
made-up pixels): the same files and renditions, its pixels in them, and
refused once the document changes.

    python3 tests/actool_icon/actool_icon_test.py PATH/TO/actool
"""

import json
import os
import shutil
import sys
import tempfile

from common import HERE, cases, compile_icon, digests

sys.path.insert(0, os.path.join(HERE, "..", "..", "tools"))
import icon_flattened  # noqa: E402

# Each image a Flattened folder holds: its size and the bytes it holds.
SIDECAR_IMAGES = dict(
    [("ios/" + a, (1024, "rgba")) for a in ("any", "dark", "tintable")]
    + [("ios/60x60@2x", (120, "rgba")), ("ios/76x76@2x~ipad", (152, "rgba"))]
    + [("mac/%d@%dx" % (s, k), (s * k, "bgra-premultiplied")) for s, k in ((16, 2), (32, 2), (64, 2), (128, 2), (256, 2), (512, 2), (1024, 1))]
    + [("mac/icns/" + m, (n, "rgba")) for m, n in (("ic13", 256), ("ic11", 32), ("ic04", 16), ("ic07", 128))])


def made_up(side, kind, seed, opaque):
    """Pixels no rendering has, `kind`'s bytes: opaque (iOS's), or translucent
    toward a corner (macOS's, as Xcode's are)."""
    out = bytearray()
    for y in range(side):
        for x in range(side):
            r, g, b = (x * 7 + y * 13 + seed) & 255, (x * 3 + seed) & 255, (y * 5 + seed) & 255
            a = 255 if opaque or x + y >= side // 4 else (x + y) * 255 // (side // 4)
            if kind == "rgba":
                out += bytes((r, g, b, a))
            else:
                out += bytes(((b * a + 127) // 255, (g * a + 127) // 255, (r * a + 127) // 255, a))
    return bytes(out)


def sidecar_checks(tool, strict, failures):
    case = "C-Visor"
    expected = {}
    with open(os.path.join(HERE, "expected", case + ".txt")) as f:
        for line in f:
            key, digest = line.split(": ", 1)
            platform, path = json.loads(key)
            expected.setdefault(platform, {})[path] = json.loads(digest)
    with tempfile.TemporaryDirectory() as tmp:
        doc = os.path.join(tmp, "Visor.icon")
        shutil.copytree(os.path.join(HERE, "cases", case, "Visor.icon"), doc)
        folder = os.path.join(doc, icon_flattened.SIDECAR)
        os.makedirs(folder)
        index = {"format": icon_flattened.FORMAT, "document": icon_flattened.document_key(doc), "xcode": "made up", "images": {}}
        pixels = {}
        for seed, (label, (side, kind)) in enumerate(sorted(SIDECAR_IMAGES.items())):
            pixels[label] = made_up(side, kind, seed, label.startswith("ios/"))
            file = label.replace("/", "_") + ".png"
            icon_flattened.write_png(os.path.join(folder, file), side, side, pixels[label])
            index["images"][label] = {"file": file, "width": side, "height": side, "bytes": kind}
        with open(os.path.join(folder, "index.json"), "w") as f:
            json.dump(index, f)
        for platform in ("sim", "mac", "ios"):
            with tempfile.TemporaryDirectory() as out:
                status, log = compile_icon(tool, doc, [], platform, out, strict)
                if status:
                    failures.append("%s %s with Flattened/: refused:\n%s" % (case, platform, log))
                    continue
                ours = json.loads(json.dumps(digests(out)))
                if platform == "sim":
                    w, h, png = icon_flattened.png_rgba(os.path.join(out, "Visor60x60@2x.png"))
                    if png != pixels["ios/60x60@2x"]:
                        failures.append("%s sim with Flattened/: the bundle icon isn't its pixels" % case)
            want = expected[platform]
            different = sorted(p for p in set(want) | set(ours) if want.get(p) != ours.get(p))
            if different:
                failures.append("%s %s with Flattened/: different from Xcode's: %s" % (case, platform, ", ".join(different)))
        with open(os.path.join(doc, "icon.json"), "a") as f:
            f.write(" ")
        with tempfile.TemporaryDirectory() as out:
            status, _ = compile_icon(tool, doc, [], "sim", out, strict)
            if status == 0:
                failures.append("%s: compiled strict with a Flattened folder of another version of the document" % case)


def main(argv):
    if len(argv) != 1:
        sys.exit(__doc__)
    tool = [os.path.abspath(argv[0])]
    relaxed = dict(os.environ, RULES_APPLECROSS_RELAX_ACTOOL_ICON_COMPOSER="1")
    strict = {k: v for k, v in os.environ.items() if not k.startswith("RULES_APPLECROSS_RELAX") and k != "RULES_APPLECROSS_RESOURCE_OUTPUT_MODE"}
    failures, count = [], 0
    for case, icon, catalogs, platforms in cases():
        expected = {}
        with open(os.path.join(HERE, "expected", case + ".txt")) as f:
            for line in f:
                key, digest = line.split(": ", 1)
                platform, path = json.loads(key)
                expected.setdefault(platform, {})[path] = json.loads(digest)
        for platform in platforms:
            count += 1
            with tempfile.TemporaryDirectory() as out:
                status, log = compile_icon(tool, icon, catalogs, platform, out, relaxed)
                if status:
                    failures.append("%s %s: refused:\n%s" % (case, platform, log))
                    continue
                ours = json.loads(json.dumps(digests(out)))
            want = expected[platform]
            different = sorted(p for p in set(want) | set(ours) if want.get(p) != ours.get(p))
            if different:
                failures.append("%s %s: different from Xcode's: %s" % (case, platform, ", ".join(different)))
        with tempfile.TemporaryDirectory() as out:
            status, _ = compile_icon(tool, icon, catalogs, "sim", out, strict)
            if status == 0:
                failures.append("%s: compiled strict, where its rasters can't be Xcode's" % case)
    sidecar_checks(tool, strict, failures)
    for f in failures:
        print(f)
    print("%d of %d compiled as Xcode does" % (count - len([f for f in failures if "strict" not in f]), count))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
