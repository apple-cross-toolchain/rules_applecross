#!/usr/bin/env python3
"""Compares outputs with Xcode's, allowing only what Xcode itself varies.

Some of Xcode's tools don't write the same bytes twice for the same input.
Each such variation is listed here, with how to put both sides in one order
before comparing them byte for byte; everything else must match exactly.
Our own tools are deterministic: where Xcode varies, they keep one order.

  python3 tools/xcode_equivalence.py XCODE OURS

compares two directories (or two files), prints each file that differs
beyond those variations, and exits 1 if any do.

Known variations (docs/clean-room/README.md#determinism):
- coremlc writes a model's user-defined metadata, a protobuf map, in a
  randomly seeded order: in `coremldata.bin` (the model description, or the
  whole specification of a text model or a pipeline part) and in
  `metadata.json`'s `userDefinedMetadata`.
- actool writes the blocks of `Assets.car`'s APPEARANCEKEYS tree (each
  appearance's name and id) in a random order when a catalog has more than
  one appearance.
"""

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


def _sorted_model(model):
    """A Model with its own description's metadata, and its pipeline's parts'
    (fields 200-202, PipelineClassifier/Regressor/Pipeline), sorted."""
    out = []
    for number, raw, payload in _fields(model):
        if number == 2 and payload is not None:
            out.append(_key(2, _sorted_description(payload)))
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
    """coremldata.bin with its metadata map sorted. Two archives hold it:
    - `spec`: a 72-byte header, then the whole specification;
    - `generic...`: a header naming the parts, then the length-prefixed
      model description (and, after it, what doesn't vary)."""
    if len(data) < 36:
        return data
    tag_length = struct.unpack_from("<Q", data, 28)[0]
    tag = data[36:36 + tag_length]
    if tag == b"spec":
        return data[:72] + _sorted_model(data[72:])
    if tag.startswith(b"generic"):
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


def equivalent(path, xcode, ours):
    """Whether `ours` is Xcode's output at `path`, up to Xcode's variations."""
    if xcode == ours:
        return True
    if os.path.basename(path) == "Assets.car":
        try:
            return _car_equivalent(xcode, ours)
        except (ValueError, IndexError, struct.error):
            return False
    return canonical(path, xcode) == canonical(path, ours)


def tree(root):
    out = {}
    for directory, _, files in os.walk(root):
        for name in files:
            path = os.path.join(directory, name)
            with open(path, "rb") as f:
                out[os.path.relpath(path, root)] = f.read()
    return out


def differences(xcode_root, ours_root):
    """The files of two directories that differ beyond Xcode's variations."""
    xcode, ours = tree(xcode_root), tree(ours_root)
    out = []
    for path in sorted(set(xcode) | set(ours)):
        if path not in xcode or path not in ours:
            out.append("only in %s: %s" % (xcode_root if path in xcode else ours_root, path))
        elif not equivalent(os.path.join(os.path.abspath(xcode_root), path), xcode[path], ours[path]):
            out.append("different: %s" % path)
    return out


def main(argv):
    if len(argv) != 2:
        print("usage: xcode_equivalence.py XCODE OURS", file=sys.stderr)
        return 2
    if os.path.isfile(argv[0]) and os.path.isfile(argv[1]):
        with open(argv[0], "rb") as a, open(argv[1], "rb") as b:
            same = equivalent(os.path.abspath(argv[0]), a.read(), b.read())
        diffs = [] if same else ["different: %s" % argv[1]]
    else:
        diffs = differences(argv[0], argv[1])
    for line in diffs:
        print(line)
    return 1 if diffs else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
