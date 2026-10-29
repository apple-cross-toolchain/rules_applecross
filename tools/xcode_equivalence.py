#!/usr/bin/env python3
"""Compares outputs with Xcode's, allowing only what Xcode itself varies.

Some of Xcode's tools don't write the same bytes twice for the same input.
Each such variation is listed here, with how to put both sides in one order
before comparing them byte for byte; everything else must match exactly.
Our own tools are deterministic: where Xcode varies, they keep one order.

  python3 tools/xcode_equivalence.py [--approximate-icon-rasters] XCODE OURS

compares two directories (or two files), prints each file that differs
beyond those variations, and exits 1 if any do.

With --approximate-icon-rasters, the flattened images of an Icon Composer
document may also differ in their pixels: the relaxed actool renders them
itself (--config=relax_actool_icon_composer). Their renditions' keys, names
(but for Xcode's random parts), sizes, metadata and storage layout must
still match, as must every other byte of the catalog; so must the bundle's
PNG icons and the macOS .icns but for their pixel data.

Known variations (docs/clean-room/README.md#determinism):
- coremlc writes a model's user-defined metadata, a protobuf map, in a
  randomly seeded order: in `coremldata.bin` (the model description, or the
  whole specification of a model compiled to its specification or a
  pipeline part) and in `metadata.json`'s `userDefinedMetadata` (a
  package's Metadata overlay adds its creator-defined entries). So it
  writes the other maps of a specification it copies: a custom model's
  parameters, a categorical mapping's map and an imputer's dictionary.
- actool writes the blocks of `Assets.car`'s APPEARANCEKEYS tree (each
  appearance's name and id) in a random order when a catalog has more than
  one appearance.
- For an Icon Composer document, actool names each flattened image's
  rendition with a fresh UUID, its process id and a time
  (`NAME1024x1024_UIAppearanceAny_<UUID>-<pid>-<16 hex digits>.png`, cut to
  127 bytes), and gives the icon's facet in FACETKEYS the part of its icon
  images (220) or of its icon stacks (245), at random.
"""

import hashlib
import os
import re
import struct
import sys


# --- Protobuf -------------------------------------------------------------------

def _varint(data, i):
    value = shift = 0
    while True:
        b = data[i]
        i += 1
        value |= (b & 0x7F) << shift
        shift += 7
        if b < 0x80:
            return value, i


def _fields(data):
    """(number, raw bytes of the whole field, payload or None) of a message."""
    out, i = [], 0
    while i < len(data):
        start = i
        key, i = _varint(data, i)
        number, wire = key >> 3, key & 7
        payload = None
        if wire == 0:
            _, i = _varint(data, i)
        elif wire == 1:
            i += 8
        elif wire == 2:
            length, i = _varint(data, i)
            payload = data[i:i + length]
            i += length
        elif wire == 5:
            i += 4
        else:
            raise ValueError("wire type %d" % wire)
        if i > len(data):
            raise ValueError("truncated message")
        out.append((number, data[start:i], payload))
    return out


def _key(number, payload):
    out, key = bytearray(), (number << 3) | 2
    while True:
        out.append((key & 0x7F) | (0x80 if key > 0x7F else 0))
        key >>= 7
        if not key:
            break
    length = len(payload)
    while True:
        out.append((length & 0x7F) | (0x80 if length > 0x7F else 0))
        length >>= 7
        if not length:
            break
    return bytes(out) + payload


def _sorted_metadata(metadata):
    """Metadata (ModelDescription field 100) with its userDefined map (field
    100) in byte order, in place of the order it came in."""
    fields = _fields(metadata)
    entries = iter(sorted(raw for number, raw, _ in fields if number == 100))
    return b"".join(next(entries) if number == 100 else raw for number, raw, _ in fields)


def _sorted_description(description):
    return b"".join(_key(100, _sorted_metadata(payload)) if number == 100 and payload is not None else raw
                    for number, raw, payload in _fields(description))


def _sorted_entries(message, number):
    """`message` with the entries of its map field `number` in byte order."""
    fields = _fields(message)
    entries = iter(sorted(raw for n, raw, _ in fields if n == number))
    return b"".join(next(entries) if n == number else raw for n, raw, _ in fields)


# Maps in a model's own parameters, by Model field: (the map's field, or
# (the field holding it, the map's field)).
_MODEL_MAPS = {555: [30], 601: [(6, 1), (7, 1)], 606: [(1, 1), (2, 1)]}


def _sorted_parameters(number, payload):
    for where in _MODEL_MAPS[number]:
        if isinstance(where, int):
            payload = _sorted_entries(payload, where)
        else:
            outer, inner = where
            payload = b"".join(_key(outer, _sorted_entries(p, inner)) if n == outer and p is not None else r
                               for n, r, p in _fields(payload))
    return payload


def _sorted_model(model):
    """A Model with its own description's metadata, its own parameters'
    maps, and its pipeline's parts' (fields 200-202,
    PipelineClassifier/Regressor/Pipeline), sorted."""
    out = []
    for number, raw, payload in _fields(model):
        if number == 2 and payload is not None:
            out.append(_key(2, _sorted_description(payload)))
        elif number in _MODEL_MAPS and payload is not None:
            out.append(_key(number, _sorted_parameters(number, payload)))
        elif number in (200, 201) and payload is not None:
            out.append(_key(number, b"".join(
                _key(1, _sorted_pipeline(p)) if n == 1 and p is not None else r for n, r, p in _fields(payload))))
        elif number == 202 and payload is not None:
            out.append(_key(202, _sorted_pipeline(payload)))
        else:
            out.append(raw)
    return b"".join(out)


def _sorted_pipeline(pipeline):
    return b"".join(_key(1, _sorted_model(p)) if n == 1 and p is not None else r for n, r, p in _fields(pipeline))


# --- Core ML's compiled files ---------------------------------------------------

def _coremldata(data):
    """coremldata.bin with its metadata map sorted. Two kinds of archive
    hold it:
    - `spec`: a 72-byte header, then the whole specification;
    - `generic...` (`notreesyet` for k-nearest neighbors): a header naming
      the parts, then the length-prefixed model description (and, after
      it, what doesn't vary)."""
    if len(data) < 36:
        return data
    tag_length = struct.unpack_from("<Q", data, 28)[0]
    tag = data[36:36 + tag_length]
    if tag == b"spec":
        return data[:72] + _sorted_model(data[72:])
    if tag.startswith(b"generic") or tag == b"notreesyet":
        at = 36 + tag_length + 4 + 28
        if at + 8 > len(data):
            return data
        length = struct.unpack_from("<Q", data, at)[0]
        description = data[at + 8:at + 8 + length]
        return data[:at + 8] + _sorted_description(description) + data[at + 8 + length:]
    return data


_USER_DEFINED = re.compile(rb'(\n( *)"userDefinedMetadata" : \{\n)(.*?)(\n\2\})', re.S)


def _metadata_json(data):
    """metadata.json with the entries of `userDefinedMetadata` (one a line)
    sorted."""
    def sort(match):
        lines = [line.rstrip(b",") for line in match.group(3).split(b"\n")]
        lines.sort()
        return match.group(1) + b",\n".join(lines) + match.group(4)
    return _USER_DEFINED.sub(sort, data)


# --- Assets.car -----------------------------------------------------------------

def _bom(data):
    """(block table [(address, length)], variables {name: block}, the
    table's own range) of a BOMStore file."""
    if data[:8] != b"BOMStore":
        raise ValueError("not a BOM store")
    index_at, index_length, vars_at = struct.unpack_from(">III", data, 16)
    count = struct.unpack_from(">I", data, index_at)[0]
    blocks = [struct.unpack_from(">II", data, index_at + 4 + 8 * i) for i in range(count)]
    variables, i = {}, vars_at + 4
    for _ in range(struct.unpack_from(">I", data, vars_at)[0]):
        block, length = struct.unpack_from(">IB", data, i)
        variables[data[i + 5:i + 5 + length]] = block
        i += 5 + length
    return blocks, variables, (index_at, index_at + 4 + 8 * count)


def _appearance_blocks(data, blocks, variables):
    """The APPEARANCEKEYS tree's leaf, the blocks of its entries, and the
    entries (name, id) in order."""
    tree = variables.get(b"APPEARANCEKEYS")
    if tree is None:
        return None, set(), []
    at = blocks[tree][0]
    if data[at:at + 4] != b"tree":
        raise ValueError("APPEARANCEKEYS isn't a tree")
    leaf = struct.unpack_from(">I", data, at + 8)[0]
    at = blocks[leaf][0]
    is_leaf, count, forward = struct.unpack_from(">HHI", data, at)
    if not is_leaf or forward:
        raise ValueError("APPEARANCEKEYS has more than one leaf")
    owned, entries = set(), []
    for k in range(count):
        value, key = struct.unpack_from(">II", data, at + 12 + 8 * k)
        owned |= {value, key}
        (ka, kl), (va, vl) = blocks[key], blocks[value]
        entries.append((data[ka:ka + kl], data[va:va + vl]))
    return leaf, owned, entries


def _car_equivalent(xcode, ours):
    """Two Assets.car files that differ at most in where actool put the
    blocks of the APPEARANCEKEYS tree (a random order when a catalog has
    more than one appearance): the same entries in the same order, the same
    blocks' contents, and every other byte the same."""
    if len(xcode) != len(ours):
        return False
    xb, xv, xi = _bom(xcode)
    ob, ov, oi = _bom(ours)
    if xv != ov or len(xb) != len(ob) or xi != oi:
        return False
    xl, xs, xe = _appearance_blocks(xcode, xb, xv)
    ol, os_, oe = _appearance_blocks(ours, ob, ov)
    if xe != oe or xs != os_ or xl != ol:
        return False
    if sorted(xcode[a:a + n] for a, n in (xb[i] for i in xs)) != sorted(ours[a:a + n] for a, n in (ob[i] for i in os_)):
        return False
    # Everything else is the same: the leaf but for its entries' block
    # numbers, and the file outside the entries' blocks and table entries.
    mask = bytearray(len(xcode))
    if xl is not None:
        a = xb[xl][0] + 12
        mask[a:a + 8 * len(xe)] = b"\1" * (8 * len(xe))
    for table in (xb, ob):
        for i in xs:
            a, n = table[i]
            mask[a:a + n] = b"\1" * n
    for i in xs:
        at = xi[0] + 4 + 8 * i
        mask[at:at + 8] = b"\1" * 8
    return all(m or x == o for m, x, o in zip(mask, xcode, ours))


# --- Icon Composer documents ------------------------------------------------------

_ICON_STACK, _ICON_IMAGE, _PACKED = 245, 220, 181
_RANDOM = re.compile(rb"_[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}-[0-9]+-[0-9A-F]{16}\.png$")
_APPEARANCE_NAME = re.compile(rb"^(.*_(?:UIAppearance|NSAppearanceName|ISAppearance)[A-Za-z]+)_")


def _layout(blocks, variables, capacity):
    """The bytes of a BOM store holding `blocks` (index 1 on, as contents)
    and `variables` [(name, block)], as actool lays them out: the blocks in
    order from 0x200, each 16-byte aligned, then the variables, then the
    block table with room for `capacity` entries and an empty free list."""
    out = bytearray(0x200)
    addresses = [(0, 0)]
    for b in blocks[1:]:
        out += b"\0" * (-len(out) % 16)
        addresses.append((len(out), len(b)))
        out += b
    out += b"\0" * (-len(out) % 16)
    vars_at = len(out)
    out += struct.pack(">I", len(variables))
    for name, index in variables:
        out += struct.pack(">IB", index, len(name)) + name
    vars_length = len(out) - vars_at
    out += b"\0" * (-len(out) % 16)
    index_at = len(out)
    out += struct.pack(">I", capacity)
    for i in range(capacity):
        out += struct.pack(">II", *(addresses[i] if i < len(addresses) else (0, 0)))
    out += b"\0" * 20
    out[:32] = b"BOMStore" + struct.pack(">6I", 1, len(blocks) - 1, index_at, len(out) - index_at, vars_at, vars_length)
    return bytes(out)


def _store(data):
    """A BOM store's blocks (contents, index 0 empty) and variables in
    order, if the file is exactly what actool's layout of them gives."""
    if data[:8] != b"BOMStore":
        raise ValueError("not a BOM store")
    used, index_at, _, vars_at = struct.unpack_from(">IIII", data, 12)
    capacity = struct.unpack_from(">I", data, index_at)[0]
    blocks = [b""]
    for i in range(1, used + 1):
        at, length = struct.unpack_from(">II", data, index_at + 4 + 8 * i)
        blocks.append(data[at:at + length])
    variables, i = [], vars_at + 4
    for _ in range(struct.unpack_from(">I", data, vars_at)[0]):
        index, length = struct.unpack_from(">IB", data, i)
        variables.append((data[i + 5:i + 5 + length], index))
        i += 5 + length
    if _layout(blocks, variables, capacity) != data:
        raise ValueError("not laid out as actool lays out its BOM stores")
    return blocks, variables, capacity


def _tree_entries(blocks, header):
    """A single-leaf tree's entries: (value block, key block) pairs."""
    h = blocks[header]
    if h[:4] != b"tree":
        raise ValueError("not a tree")
    leaf = blocks[struct.unpack_from(">I", h, 8)[0]]
    is_leaf, count, forward = struct.unpack_from(">HHI", leaf, 0)
    if not is_leaf or forward:
        raise ValueError("a tree of more than one leaf")
    return [struct.unpack_from(">II", leaf, 12 + 8 * k) for k in range(count)]


def _key_attributes(blocks, variables, key):
    keyformat = blocks[dict(variables)[b"KEYFORMAT"]]
    n = struct.unpack_from("<I", keyformat, 8)[0]
    attributes = struct.unpack_from("<%dI" % n, keyformat, 12)
    return dict(zip(attributes, struct.unpack_from("<%dH" % n, key, 0)))


def _raster_name(name):
    """A flattened image's name without what Xcode makes up each run: its
    UUID, process id and time, or as much of them as the 127 bytes kept."""
    name = name.split(b"\0")[0]
    if _RANDOM.search(name):
        return _RANDOM.sub(b"", name)
    m = _APPEARANCE_NAME.match(name)
    if m and re.fullmatch(rb"_[0-9A-F-]*(-[0-9]+(-[0-9A-F]*)?)?", name[len(m.group(1)):]) and len(name) == 127:
        return m.group(1)
    return name


def _mlec_layout(payload):
    """How an LZFSE-compressed bitmap is stored, its compressed bytes
    aside: its version and method, and its chunks' rows."""
    if payload[:4] != b"MLEC":
        raise ValueError("not an MLEC bitmap")
    version, method, count = struct.unpack_from("<III", payload, 4)
    if method != 4:
        raise ValueError("not LZFSE")
    if version in (0, 2):
        return version, method
    rows, at = [], 16
    for _ in range(count):
        if payload[at:at + 4] != b"KCBC":
            raise ValueError("a malformed chunk")
        zero1, zero2, n, length = struct.unpack_from("<4I", payload, at + 4)
        rows.append((zero1, zero2, n))
        at += 20 + length
    if at != len(payload):
        raise ValueError("bytes after the last chunk")
    return version, method, rows


def _raster_signature(block, approximate):
    """A flattened image's rendition but for what isn't Xcode's to keep: the
    name's random part and, when `approximate`, its pixels (keeping the CSI
    header, the TLVs and how the bitmap is stored)."""
    if len(block) < 184:
        raise ValueError("a short rendition")
    tlvs = struct.unpack_from("<I", block, 168)[0]
    if not approximate:
        return ("raster", block[:40], _raster_name(block[40:168]), _digest(block[168:]))
    if struct.unpack_from("<I", block, 180)[0] != len(block) - 184 - tlvs:
        raise ValueError("a rendition whose payload length is wrong")
    return ("raster~", block[:40], _raster_name(block[40:168]), block[168:180], block[184:184 + tlvs], _mlec_layout(block[184 + tlvs:]))


def _is_icon_catalog(blocks, variables):
    names = dict(variables)
    if b"RENDITIONS" not in names or b"KEYFORMAT" not in names:
        return False
    return any(_key_attributes(blocks, variables, blocks[k]).get(2) == _ICON_STACK for _, k in _tree_entries(blocks, names[b"RENDITIONS"]))


def _is_icon_atlas(attributes, block):
    """The atlas of a macOS document's smallest flattened images: 2x, no
    appearance, translucent colour. A catalog's own atlases beside it are
    exact (actool refuses catalog images that would share this one)."""
    return (attributes.get(1) == 9 and attributes.get(2) == _PACKED and attributes.get(12) == 2 and not attributes.get(7)
            and block[40:168].split(b"\0")[0] == b"ZZZZPackedAsset-2.0.0-gamut0")


def _digest(data):
    return hashlib.sha256(data).hexdigest()


def icon_car_digest(data, approximate_rasters):
    """An Icon Composer document's Assets.car, as what an equivalent one must
    share: laid out as actool lays out its stores, the same variables and
    block count, and block by block the same bytes but for what Xcode varies
    (the APPEARANCEKEYS blocks' order, the flattened images' names and the
    icon facet's part) and, when `approximate_rasters`, the flattened images'
    pixels. Two cars are equivalent when their digests are equal."""
    blocks, variables, capacity = _store(data)
    if not _is_icon_catalog(blocks, variables):
        raise ValueError("not an Icon Composer document's catalog")
    names = dict(variables)
    signature = {}
    # Flattened images and atlases, and their links.
    for value, key in _tree_entries(blocks, names[b"RENDITIONS"]):
        attributes = _key_attributes(blocks, variables, blocks[key])
        block = blocks[value]
        layout = struct.unpack_from("<H", block, 36)[0] if len(block) >= 40 else None
        if attributes.get(2) == _ICON_IMAGE and layout == 1003:
            signature[value] = ("link", block[:40], _raster_name(block[40:168]), _digest(block[168:]))
        elif (attributes.get(2) == _ICON_IMAGE and layout == 12) or _is_icon_atlas(attributes, block):
            signature[value] = _raster_signature(block, approximate_rasters)
    # The icon's facet (a hotspot of three attributes: element 85, part 220
    # or 245, identifier), its part aside.
    for value, _ in _tree_entries(blocks, names[b"FACETKEYS"]):
        facet = blocks[value]
        if len(facet) == 18 and struct.unpack_from("<6H", facet, 0)[2:] == (3, 1, 85, 2) and struct.unpack_from("<H", facet, 12)[0] in (_ICON_IMAGE, _ICON_STACK):
            signature[value] = ("facet", _digest(facet[:12] + facet[14:]))
    # The appearances: their entries in order, their blocks in any order, and
    # the leaf but for its entries' block numbers.
    appearances = None
    if b"APPEARANCEKEYS" in names:
        entries = _tree_entries(blocks, names[b"APPEARANCEKEYS"])
        leaf = struct.unpack_from(">I", blocks[names[b"APPEARANCEKEYS"]], 8)[0]
        n = len(entries)
        appearances = ([(_digest(blocks[k]), _digest(blocks[v])) for v, k in entries],
                       sorted(_digest(blocks[i]) for v, k in entries for i in (v, k)),
                       _digest(blocks[leaf][:12] + blocks[leaf][12 + 8 * n:]))
        for v, k in entries:
            signature[v] = signature[k] = ("appearance",)
        signature[leaf] = ("appearance leaf",)
    return (capacity, [(name.decode(), index) for name, index in variables], appearances,
            [signature.get(i) or _digest(blocks[i]) for i in range(1, len(blocks))])


def _icon_car_equivalent(xcode, ours, approximate_rasters):
    try:
        return icon_car_digest(xcode, approximate_rasters) == icon_car_digest(ours, approximate_rasters)
    except (ValueError, struct.error):
        return False


# --- PNG and icon family files ------------------------------------------------------

def _png_chunks(data):
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG")
    out, at = [], 8
    while at < len(data):
        length = struct.unpack_from(">I", data, at)[0]
        out.append((data[at + 4:at + 8], data[at + 8:at + 8 + length]))
        at += 12 + length
    if at != len(data):
        raise ValueError("a truncated PNG")
    return out


def _png_layout(data):
    """A PNG but for its pixels: its chunks in order, the image data as one
    entry however many IDAT chunks carry it, and a CgBI PNG's iDOT (its
    segments' rows, then where in the file each segment's data starts) but
    for those offsets."""
    out = []
    for kind, body in _png_chunks(data):
        if kind == b"IDAT":
            if not out or out[-1] != b"IDAT":
                out.append(b"IDAT")
        elif kind == b"iDOT" and len(body) % 4 == 0:
            fields = list(struct.unpack(">%dI" % (len(body) // 4), body))
            out.append((kind, tuple(None if i >= 6 and i % 3 == 0 else v for i, v in enumerate(fields))))
        else:
            out.append((kind, body))
    return out


def _icns_layout(data):
    if data[:4] != b"icns" or struct.unpack_from(">I", data, 4)[0] != len(data):
        raise ValueError("not an icon family")
    out, at = [], 8
    while at < len(data):
        kind, length = data[at:at + 4], struct.unpack_from(">I", data, at + 4)[0]
        body = data[at + 8:at + length]
        if body[:8] == b"\x89PNG\r\n\x1a\n":
            out.append((kind, _png_layout(body)))
        elif body[:4] == b"ARGB":
            out.append((kind, b"ARGB"))
        else:
            out.append((kind, body))
        at += length
    if at != len(data):
        raise ValueError("a truncated icon family")
    return out


def _rasters_equivalent(path, xcode, ours):
    """A bundle's PNG icon or .icns but for its pixels."""
    try:
        if path.endswith(".png"):
            return _png_layout(xcode) == _png_layout(ours)
        if path.endswith(".icns"):
            return _icns_layout(xcode) == _icns_layout(ours)
    except (ValueError, struct.error):
        pass
    return False


def _jsonable(v):
    if isinstance(v, bytes):
        return v.hex()
    if isinstance(v, (tuple, list)):
        return [_jsonable(x) for x in v]
    return v


def icon_output_digest(path, data):
    """What an Icon Composer document's output at `path` must share with
    Xcode's, the flattened images' pixels aside: comparable as JSON, so a
    test can keep Xcode's in a few kilobytes rather than its megabytes."""
    name = os.path.basename(path)
    if name == "Assets.car":
        return _jsonable(icon_car_digest(data, True))
    if name.endswith(".png"):
        return _jsonable(_png_layout(data))
    if name.endswith(".icns"):
        return _jsonable(_icns_layout(data))
    return _digest(data)


def canonical(path, data):
    """`data`, the file at `path`, with what Xcode varies put in one order."""
    parts = path.replace(os.sep, "/").split("/")
    if not any(p.endswith(".mlmodelc") for p in parts[:-1]):
        return data
    try:
        if parts[-1] == "coremldata.bin" and "analytics" not in parts:
            return _coremldata(data)
        if parts[-1] == "metadata.json":
            return _metadata_json(data)
    except (ValueError, IndexError, struct.error):
        pass
    return data


def is_icon_document_catalog(data):
    """Whether an Assets.car holds an Icon Composer document's renditions."""
    try:
        blocks, variables, _ = _store(data)
        return _is_icon_catalog(blocks, variables)
    except (ValueError, IndexError, KeyError, struct.error):
        return False


def equivalent(path, xcode, ours, approximate_icon_rasters=False):
    """Whether `ours` is Xcode's output at `path`, up to Xcode's variations;
    with `approximate_icon_rasters`, `path` (an Assets.car, a PNG or an
    .icns) being an Icon Composer document's output, also up to its
    flattened images' pixels."""
    if xcode == ours:
        return True
    if os.path.basename(path) == "Assets.car":
        try:
            if _car_equivalent(xcode, ours):
                return True
        except (ValueError, IndexError, struct.error):
            pass
        try:
            return _icon_car_equivalent(xcode, ours, approximate_icon_rasters)
        except (ValueError, IndexError, KeyError, struct.error):
            return False
    if approximate_icon_rasters and (path.endswith(".png") or path.endswith(".icns")):
        return _rasters_equivalent(path, xcode, ours)
    return canonical(path, xcode) == canonical(path, ours)


def tree(root):
    out = {}
    for directory, _, files in os.walk(root):
        for name in files:
            path = os.path.join(directory, name)
            with open(path, "rb") as f:
                out[os.path.relpath(path, root)] = f.read()
    return out


def differences(xcode_root, ours_root, approximate_icon_rasters=False):
    """The files of two directories that differ beyond Xcode's variations.
    With `approximate_icon_rasters`, a directory whose Assets.car (Xcode's)
    holds an Icon Composer document may differ in its flattened images'
    pixels, there and in the PNG and .icns files beside it."""
    xcode, ours = tree(xcode_root), tree(ours_root)
    icon_dirs = {os.path.dirname(p) for p, data in xcode.items()
                 if approximate_icon_rasters and os.path.basename(p) == "Assets.car" and is_icon_document_catalog(data)}
    out = []
    for path in sorted(set(xcode) | set(ours)):
        if path not in xcode or path not in ours:
            out.append("only in %s: %s" % (xcode_root if path in xcode else ours_root, path))
        elif not equivalent(os.path.join(os.path.abspath(xcode_root), path), xcode[path], ours[path],
                            os.path.dirname(path) in icon_dirs):
            out.append("different: %s" % path)
    return out


def main(argv):
    approximate = "--approximate-icon-rasters" in argv
    argv = [a for a in argv if a != "--approximate-icon-rasters"]
    if len(argv) != 2:
        print("usage: xcode_equivalence.py [--approximate-icon-rasters] XCODE OURS", file=sys.stderr)
        return 2
    if os.path.isfile(argv[0]) and os.path.isfile(argv[1]):
        with open(argv[0], "rb") as a, open(argv[1], "rb") as b:
            same = equivalent(os.path.abspath(argv[0]), a.read(), b.read(), approximate)
        diffs = [] if same else ["different: %s" % argv[1]]
    else:
        diffs = differences(argv[0], argv[1], approximate)
    for line in diffs:
        print(line)
    return 1 if diffs else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
