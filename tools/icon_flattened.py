#!/usr/bin/env python3
"""Pre-render an Icon Composer document's flattened images with Xcode.

    tools/icon_flattened.py DOC.icon [DOC.icon ...]

On a Mac with Xcode, compiles each document with Xcode's own actool for iOS
(iPhone and iPad) and macOS and writes `DOC.icon/Flattened/`: Xcode's pixels
for everything an Icon Composer document's output holds that this actool
doesn't draw itself (docs/clean-room/actool.md, Icon Composer documents):

- iOS: the 1024 px Any, Dark and Tintable images and the bundle's
  `60x60@2x` and `76x76@2x~ipad` icons;
- macOS: the System appearance's images at 16 to 512 pt @2x and 1024 pt @1x
  (premultiplied, as the catalog stores them) and the icon family's members.

`index.json` names the document's contents' SHA-256 (every file in the
document but `Flattened/`, see `document_key`); this actool uses the images,
strict, only when the document still has that digest, and refuses it
otherwise. Run it again whenever the document changes.

Each image is a lossless 8-bit RGBA PNG holding exactly the bytes named in
the index (`rgba`, or `bgra-premultiplied` for what the catalog stores).
The index also records the Xcode, machine model and macOS build that drew
them: the pixels come from the host's GPU (Metal) and vImage, so another
macOS build or machine can differ by one in a few channels; Xcode on one
host is deterministic. Render on the host whose output you mean to match.
Nothing here is checked by this script beyond Xcode's own output: compare a
build with `tools/xcode_equivalence.py`.
"""

import ctypes
import hashlib
import json
import os
import struct
import subprocess
import sys
import tempfile
import zlib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import xcode_equivalence as eq  # noqa: E402

SIDECAR = "Flattened"
FORMAT = "rules_applecross flattened Icon Composer images 1"


def document_key(doc):
    """SHA-256 over the document's files but `Flattened/`, in byte order of
    their paths: each as its path relative to the document (UTF-8, `/`), a
    NUL, its length (8 bytes little-endian) and its bytes."""
    files = []
    for root, dirs, names in os.walk(doc):
        dirs.sort()
        for n in names:
            p = os.path.join(root, n)
            rel = os.path.relpath(p, doc).replace(os.sep, "/")
            if rel.split("/")[0] == SIDECAR:
                continue
            files.append((rel.encode(), p))
    h = hashlib.sha256()
    for rel, p in sorted(files):
        data = open(p, "rb").read()
        h.update(rel + b"\0" + struct.pack("<Q", len(data)) + data)
    return h.hexdigest()


def actool(doc, out, platform, devices, deployment):
    args = ["xcrun", "actool", "--compile", out, "--platform", platform, "--minimum-deployment-target", deployment,
            "--app-icon", os.path.splitext(os.path.basename(doc))[0], "--output-partial-info-plist", os.path.join(out, "partial.plist")]
    for d in devices:
        args += ["--target-device", d]
    subprocess.run(args + [doc], check=True, stdout=subprocess.DEVNULL)


_lib = None


def _lzfse(src, cap):
    """LZFSE decoding with macOS's libcompression (this script runs on a Mac)."""
    global _lib
    if _lib is None:
        _lib = ctypes.CDLL("/usr/lib/libcompression.dylib")
    dst = ctypes.create_string_buffer(cap)
    n = _lib.compression_decode_buffer(dst, ctypes.c_size_t(cap), src, ctypes.c_size_t(len(src)), None, 0x801)
    return dst.raw[:n]


def _mlec(payload, width, height, stride=None):
    """A rendition's MLEC payload: KCBC chunks of LZFSE rows of BGRA, `stride`
    bytes apart (an atlas's rows are padded); the rows unpadded."""
    stride = stride or width * 4
    if payload[:4] != b"MLEC":
        raise ValueError("not an MLEC payload")
    at, rows = 16, []
    while at < len(payload) and payload[at:at + 4] == b"KCBC":
        _, _, n, size = struct.unpack_from("<IIII", payload, at + 4)
        rows.append(_lzfse(payload[at + 20:at + 20 + size], n * stride + 64))
        at += 20 + size
    data = b"".join(rows)
    if len(data) != stride * height:
        raise ValueError("an MLEC payload of %d bytes for %dx%d" % (len(data), width, height))
    return b"".join(data[r * stride:r * stride + width * 4] for r in range(height))


def _tlvs(block):
    count = struct.unpack_from("<I", block, 168)[0]
    at, end, out = 184, 184 + count, {}
    while at < end:
        tag, n = struct.unpack_from("<II", block, at)
        out[tag] = block[at + 8:at + 8 + n]
        at += 8 + n
    return out, block[end:]


def car_rasters(path):
    """The catalog's flattened icon images (part 220): name prefix, scale and
    premultiplied BGRA, an atlas member cut out of its atlas."""
    blocks, variables, _ = eq._store(open(path, "rb").read())
    names = dict(variables)
    entries = [(blocks[v], eq._key_attributes(blocks, variables, blocks[k])) for v, k in eq._tree_entries(blocks, names[b"RENDITIONS"])]
    atlases = {}
    for block, attrs in entries:
        if attrs.get(1) == 9 and block[:4] == b"ISTC":
            w, h = struct.unpack_from("<II", block, 12)
            tlvs, payload = _tlvs(block)
            stride = struct.unpack_from("<I", tlvs[0x3ef])[0] if 0x3ef in tlvs else None
            atlases[tuple(sorted((a, v) for a, v in attrs.items() if v))] = (w, h, _mlec(payload, w, h, stride))
    out = []
    for block, attrs in entries:
        if attrs.get(2) != 220 or block[:4] != b"ISTC":
            continue
        layout = struct.unpack_from("<H", block, 36)[0]
        name = block[40:168].split(b"\0")[0].decode()
        w, h = struct.unpack_from("<II", block, 12)
        tlvs, payload = _tlvs(block)
        if layout == 12:
            data = _mlec(payload, w, h, struct.unpack_from("<I", tlvs[0x3ef])[0] if 0x3ef in tlvs else None)
        elif layout == 1003:
            link = tlvs[0x3f2]
            _, x, y, lw, lh = struct.unpack_from("<5I", link, 4)
            n = (struct.unpack_from("<I", link, 26)[0] - 4) // 4
            key = tuple(sorted(struct.unpack_from("<2H", link, 30 + 4 * i) for i in range(n)))
            atlas = next((a for k, a in atlases.items() if all(p in k for p in key)), None)
            if atlas is None:
                raise ValueError("%s: no atlas for %s" % (path, name))
            aw, ah, pixels = atlas
            data = b"".join(pixels[((ah - (y + lh)) + r) * aw * 4 + x * 4:((ah - (y + lh)) + r) * aw * 4 + (x + lw) * 4] for r in range(lh))
        else:
            continue
        out.append((name.split("_")[0], name.split("_")[1], attrs.get(12, 1), w, h, data))
    return out


def png_rgba(path_or_bytes):
    """An 8-bit RGB(A) PNG's pixels as RGBA, CgBI aside (simulator output)."""
    data = path_or_bytes if isinstance(path_or_bytes, bytes) else open(path_or_bytes, "rb").read()
    at, idat, ihdr = 8, b"", None
    while at < len(data):
        n, kind = struct.unpack_from(">I4s", data, at)
        body = data[at + 8:at + 8 + n]
        if kind == b"IHDR":
            ihdr = struct.unpack(">IIBBBBB", body)
        elif kind == b"IDAT":
            idat += body
        elif kind == b"CgBI":
            raise ValueError("a CgBI PNG")
        at += 12 + n
    w, h, depth, ctype, _, _, interlace = ihdr
    if depth != 8 or ctype not in (2, 6) or interlace:
        raise ValueError("a PNG other than 8-bit RGB(A), not interlaced")
    bpp = 4 if ctype == 6 else 3
    raw, stride = zlib.decompress(idat), w * bpp
    prev, out, at = bytearray(stride), bytearray(), 0
    for _ in range(h):
        f, line = raw[at], bytearray(raw[at + 1:at + 1 + stride])
        at += 1 + stride
        for i in range(stride):
            a = line[i - bpp] if i >= bpp else 0
            b, c = prev[i], (prev[i - bpp] if i >= bpp else 0)
            if f == 1:
                line[i] = (line[i] + a) & 255
            elif f == 2:
                line[i] = (line[i] + b) & 255
            elif f == 3:
                line[i] = (line[i] + (a + b) // 2) & 255
            elif f == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                line[i] = (line[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        prev = line
        if bpp == 4:
            out += line
        else:
            for i in range(0, stride, 3):
                out += line[i:i + 3] + b"\xff"
    return w, h, bytes(out)


def _unpackbits(data, n):
    out, at = bytearray(), 0
    while len(out) < n:
        c = data[at]
        at += 1
        if c < 0x80:
            out += data[at:at + c + 1]
            at += c + 1
        else:
            out += bytes([data[at]]) * (c - 0x80 + 3)
            at += 1
    return bytes(out[:n]), at


def icns_members(path):
    """An icon family's members' pixels as RGBA: PNGs, and ARGB (`ARGB`, then
    each channel's planes run-length coded as icon families code them)."""
    data, out, at = open(path, "rb").read(), {}, 8
    while at < len(data):
        kind, n = struct.unpack_from(">4sI", data, at)
        body = data[at + 8:at + n]
        if body[:8] == b"\x89PNG\r\n\x1a\n":
            out[kind.decode()] = png_rgba(body)
        elif body[:4] == b"ARGB":
            side = {"ic04": 16, "ic05": 32}[kind.decode()]
            planes, p = [], 4
            for _ in range(4):
                plane, used = _unpackbits(body[p:], side * side)
                planes.append(plane)
                p += used
            a, r, g, b = planes
            out[kind.decode()] = (side, side, bytes(v for i in range(side * side) for v in (r[i], g[i], b[i], a[i])))
        at += n
    return out


def write_png(path, w, h, rgba):
    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body) & 0xffffffff)
    raw = b"".join(b"\0" + rgba[r * w * 4:(r + 1) * w * 4] for r in range(h))
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def bgra_to_rgba(bgra):
    out = bytearray(bgra)
    out[0::4], out[2::4] = bgra[2::4], bgra[0::4]
    return bytes(out)


def flatten(doc):
    doc = os.path.abspath(doc.rstrip("/"))
    name = os.path.splitext(os.path.basename(doc))[0]
    key = document_key(doc)
    images = {}
    with tempfile.TemporaryDirectory() as tmp:
        ios, mac = os.path.join(tmp, "ios"), os.path.join(tmp, "mac")
        os.makedirs(ios), os.makedirs(mac)
        actool(doc, ios, "iphonesimulator", ["iphone", "ipad"], "18.0")
        actool(doc, mac, "macosx", ["mac"], "26.0")
        for prefix, appearance, scale, w, h, bgra in car_rasters(os.path.join(ios, "Assets.car")):
            label = {"UIAppearanceAny": "any", "UIAppearanceDark": "dark", "ISAppearanceTintable": "tintable"}[appearance]
            images.setdefault("ios/" + label, ("rgba", w, h, bgra_to_rgba(bgra)))
        for f in ("60x60@2x", "76x76@2x~ipad"):
            w, h, rgba = png_rgba(os.path.join(ios, "%s%s.png" % (name, f)))
            images["ios/" + f] = ("rgba", w, h, rgba)
        for prefix, appearance, scale, w, h, bgra in car_rasters(os.path.join(mac, "Assets.car")):
            size = prefix[len(name):].split("x")[0]
            images["mac/%s@%dx" % (size, scale)] = ("bgra-premultiplied", w, h, bgra)
        for kind, (w, h, rgba) in icns_members(os.path.join(mac, name + ".icns")).items():
            images["mac/icns/" + kind] = ("rgba", w, h, rgba)
    version = subprocess.run(["xcodebuild", "-version"], check=True, capture_output=True, text=True).stdout.split()
    machine = subprocess.run(["sysctl", "-n", "hw.model"], capture_output=True, text=True).stdout.strip()
    macos = subprocess.run(["sw_vers", "-buildVersion"], capture_output=True, text=True).stdout.strip()
    out = os.path.join(doc, SIDECAR)
    os.makedirs(out, exist_ok=True)
    index = {"format": FORMAT, "document": key, "xcode": " ".join(version), "machine": machine, "macos": macos, "images": {}}
    for label, (layout, w, h, data) in sorted(images.items()):
        file = label.replace("/", "_") + ".png"
        write_png(os.path.join(out, file), w, h, data)
        index["images"][label] = {"file": file, "width": w, "height": h, "bytes": layout}
    with open(os.path.join(out, "index.json"), "w") as f:
        json.dump(index, f, indent=2, sort_keys=True)
        f.write("\n")
    print("%s: %d images, document %s" % (out, len(images), key[:12]))


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    for d in sys.argv[1:]:
        flatten(d)
