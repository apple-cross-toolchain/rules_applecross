#!/usr/bin/env python3
"""coremlc off a Mac: `coremlc compile` of a Core ML model, as rules_apple
runs it (`compile MODEL OUTDIR --output-partial-info-plist PLIST`); see
docs/clean-room/coremlc.md.

ML Programs (.mlpackage) compile to the files Xcode 27.0's coremlc writes,
read from the model's specification (Apple's published Core ML protobuf
schema). Neural networks, an older format, are refused; so is what hasn't
been established against Xcode, strict (the default). Relaxed
(RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed, --config=relaxed), refused
models fail all the same: there is no near match to write.
"""

import base64
import hashlib
import json
import os
import shutil
import struct
import sys


class Unestablished(Exception):
    pass


def unestablished(what):
    raise Unestablished(what)


# --- Protobuf wire format ------------------------------------------------------

def _varint(data, i):
    value = shift = 0
    while True:
        b = data[i]
        i += 1
        value |= (b & 0x7F) << shift
        shift += 7
        if b < 0x80:
            return value, i


class Message:
    """A decoded message: its fields by number, in order, as raw values
    (ints for varints and fixed widths, bytes for length-delimited)."""

    def __init__(self, data):
        self.data = data
        self.fields = []
        i = 0
        while i < len(data):
            key, i = _varint(data, i)
            number, wire = key >> 3, key & 7
            if wire == 0:
                value, i = _varint(data, i)
            elif wire == 1:
                value = data[i:i + 8]
                i += 8
            elif wire == 2:
                length, i = _varint(data, i)
                value = data[i:i + length]
                i += length
            elif wire == 5:
                value = data[i:i + 4]
                i += 4
            else:
                raise ValueError("wire type %d" % wire)
            self.fields.append((number, wire, value))

    def all(self, number):
        return [v for n, _, v in self.fields if n == number]

    def has(self, number):
        return any(n == number for n, _, _ in self.fields)

    def int(self, number, default=0):
        values = self.all(number)
        return values[-1] if values else default

    def bytes(self, number, default=b""):
        values = self.all(number)
        return values[-1] if values else default

    def str(self, number, default=""):
        return self.bytes(number, default.encode()).decode("utf-8")

    def msg(self, number):
        values = self.all(number)
        return Message(values[-1]) if values else None

    def msgs(self, number):
        return [Message(v) for v in self.all(number)]

    def map_str(self, number):
        """A map<string, string> field as [(key, value)], in the order written."""
        return [(e.str(1), e.str(2)) for e in self.msgs(number)]

    def map(self, number):
        """A map<string, Message> field as [(key, Message)], in the order
        written."""
        out = []
        for entry in self.msgs(number):
            out.append((entry.str(1), entry.msg(2) or Message(b"")))
        return out


# --- MIL (CoreML.Specification.MILSpec) ----------------------------------------

DATA_TYPES = {
    1: "bool", 2: "string", 10: "fp16", 11: "fp32", 12: "fp64", 13: "bf16",
    21: "int8", 22: "int16", 23: "int32", 24: "int64", 25: "int4",
    31: "uint8", 32: "uint16", 33: "uint32", 34: "uint64", 35: "uint4",
    36: "uint2", 37: "uint1", 38: "uint6", 39: "uint3",
}

OPSETS = {"CoreML5": "ios15", "CoreML6": "ios16", "CoreML7": "ios17", "CoreML8": "ios18"}


def hex_float(value):
    """`value` as C's printf("%a") writes it: the fewest hex digits."""
    if value != value:
        unestablished("a NaN constant")
    if value in (float("inf"), float("-inf")):
        unestablished("an infinite constant")
    if value == 0:
        return "-0x0p+0" if str(value).startswith("-") else "0x0p+0"
    text = value.hex()  # e.g. 0x1.9000000000000p+6
    mantissa, exponent = text.split("p")
    if "." in mantissa:
        mantissa = mantissa.rstrip("0").rstrip(".")
    return mantissa + "p" + exponent


def _declared(block, out):
    """Every name `block` and the blocks within it declare, in order."""
    for i in block.msgs(1):
        out.append(i.str(1))
    for op in block.msgs(3):
        for o in op.msgs(3):
            out.append(o.str(1))
        for b in op.msgs(4):
            _declared(b, out)


class Printer:
    def __init__(self, scalar_style):
        self.duplicates = set()
        self.seen = {}
        self.scope = [{}]
        # Program 1.3 and later write rank-0 tensors as their element type.
        self.scalar_style = scalar_style
        self.lines = []

    def dimension(self, d):
        if d.has(1):
            return str(d.msg(1).int(1))
        unestablished("a dimension other than a constant")

    def value_type(self, t):
        if t.has(1):
            tensor = t.msg(1)
            dtype = DATA_TYPES.get(tensor.int(1)) or unestablished("data type %d" % tensor.int(1))
            if tensor.has(4):
                unestablished("a tensor type with attributes")
            dims = [self.dimension(d) for d in tensor.msgs(3)]
            if tensor.int(2) != len(dims):
                unestablished("a tensor of unknown rank")
            if not dims and self.scalar_style:
                return dtype
            return "tensor<%s, [%s]>" % (dtype, ", ".join(dims))
        unestablished("a value type other than a tensor")

    def tensor_values(self, t, value):
        """The values of a TensorValue, written as model.mil writes them."""
        tensor = t.msg(1)
        dtype = DATA_TYPES.get(tensor.int(1))
        dims = [int(self.dimension(d)) for d in tensor.msgs(3)]
        if value.has(1):
            raw = value.msg(1).bytes(1)
            items = [hex_float(x) for x in struct.unpack("<%df" % (len(raw) // 4), raw)]
        elif value.has(2):
            raw = value.msg(2).bytes(1)
            ints = []
            i = 0
            while i < len(raw):
                v, i = _varint(raw, i)
                if v >= 1 << 63:
                    v -= 1 << 64
                ints.append(v)
            items = [str(v) for v in ints]
        elif value.has(3):
            raw = value.msg(3).bytes(1)
            items = ["true" if b else "false" for b in raw]
        elif value.has(4):
            items = ['"%s"' % self.escape(s.decode("utf-8")) for s in value.msg(4).all(1)]
        elif value.has(7):
            raw = value.msg(7).bytes(1)
            if dtype == "fp16":
                items = [hex_float(x) for x in struct.unpack("<%de" % (len(raw) // 2), raw)]
            else:
                unestablished("byte-packed %s values" % dtype)
        else:
            unestablished("a tensor value of another kind")
        if not dims:
            if len(items) != 1:
                unestablished("a scalar with %d values" % len(items))
            return items[0]
        return self.nest(items, dims)

    def nest(self, items, dims):
        if len(dims) == 1:
            return "[" + ", ".join(items) + "]"
        step = len(items) // dims[0] if dims[0] else 0
        return "[" + ", ".join(self.nest(items[k * step:(k + 1) * step], dims[1:]) for k in range(dims[0])) + "]"

    def escape(self, text):
        for ch in text:
            if ch in '"\\' or ord(ch) < 0x20 or ord(ch) >= 0x7F:
                unestablished("a string constant needing escapes")
        return text

    def value(self, v):
        t = v.msg(2)
        if v.has(5):
            blob = v.msg(5)
            return "%s(BLOBFILE(path = %s, offset = %s))" % (
                self.value_type(t), self.scalar_literal("string", '"%s"' % self.escape(blob.str(1))),
                self.scalar_literal("uint64", str(blob.int(2))))
        if not v.has(3):
            unestablished("a value without an immediate or blob")
        immediate = v.msg(3)
        if immediate.has(1):
            return "%s(%s)" % (self.value_type(t), self.tensor_values(t, immediate.msg(1)))
        if immediate.has(4):
            return self.dictionary(t, immediate.msg(4))
        unestablished("a tuple or list value")

    def scalar_literal(self, dtype, text):
        return ("%s(%s)" % (dtype, text)) if self.scalar_style else ("tensor<%s, []>(%s)" % (dtype, text))

    def dictionary(self, t, d):
        if not t.has(4):
            unestablished("a dictionary value without a dictionary type")
        dt = t.msg(4)
        pairs = []
        for kv in d.msgs(1):
            k, val = kv.msg(1), kv.msg(2)
            pairs.append("{%s, %s}" % (self.tensor_values(k.msg(2), k.msg(3).msg(1)), self.tensor_values(val.msg(2), val.msg(3).msg(1))))
        return "dict<%s, %s>({%s})" % (self.value_type(dt.msg(1)), self.value_type(dt.msg(2)), ", ".join(pairs))

    def declare(self, name):
        """The name a declaration is printed with: one declared more than
        once in the program is numbered by occurrence (name_0, name_1, ...)."""
        if name not in self.duplicates:
            return name
        k = self.seen.get(name, 0)
        self.seen[name] = k + 1
        self.scope[-1][name] = "%s_%d" % (name, k)
        return self.scope[-1][name]

    def ref(self, name):
        for scope in reversed(self.scope):
            if name in scope:
                return scope[name]
        return name

    def block(self, block, indent):
        replaced = {}
        for op in block.msgs(3):
            if op.str(1) == "classify":
                # The compiled program leaves classifying to Core ML: it
                # returns the probabilities in place of classify's outputs.
                probabilities = dict(op.map(2))["probabilities"].msgs(1)[0].str(1)
                outs = [o.str(1) for o in op.msgs(3)]
                replaced[outs[0]] = probabilities
                for o in outs[1:]:
                    replaced[o] = None
                continue
            self.operation(op, indent)
        outputs = []
        for o in block.all(2):
            name = replaced.get(o.decode(), o.decode())
            if name is not None and name not in outputs:
                outputs.append(name)
        return ", ".join(self.ref(n) for n in outputs)

    def operation(self, op, indent):
        pad = " " * indent
        outputs = op.msgs(3)
        if not outputs:
            unestablished("an operation without outputs")
        inputs = []
        for name, argument in sorted(op.map(2), key=lambda kv: kv[0]):
            bindings = argument.msgs(1)
            if not bindings or not all(b.has(1) for b in bindings):
                unestablished("an operation input bound to a value")
            names = [self.ref(b.str(1)) for b in bindings]
            # A variadic input lists its arguments in parentheses.
            inputs.append("%s = %s" % (name, names[0] if len(names) == 1 else "(%s)" % ", ".join(names)))
        attributes = ["%s = %s" % (name, self.value(v)) for name, v in sorted(op.map(5), key=lambda kv: kv[0])]
        head = "%s%s = %s(%s)[%s]" % (pad, "%s", op.str(1), ", ".join(inputs), ", ".join(attributes))

        def declared():
            # Several outputs are listed, each with its type.
            return ", ".join("%s %s" % (self.value_type(o.msg(2)), self.declare(o.str(1))) for o in outputs)
        blocks = op.msgs(4)
        if not blocks:
            self.lines.append(head % declared() + ";")
            return
        at = len(self.lines)
        self.lines.append(None)
        inner = " " * (indent + 4)
        for k, b in enumerate(blocks):
            self.scope.append({})
            params = ", ".join("%s %s" % (self.value_type(i.msg(2)), self.declare(i.str(1))) for i in b.msgs(1))
            self.lines.append(inner + ("(%s) {" % params if b.msgs(1) else "{"))
            returned = self.block(b, indent + 8)
            self.scope.pop()
            self.lines.append(inner + "} -> (%s)%s" % (returned, ";" if k == len(blocks) - 1 else ""))
        # The operation's output is declared after its blocks' names.
        self.lines[at] = head % declared()


def print_program(program, spec_version, extra_build_info):
    """model.mil for an MLProgram message."""
    version = program.int(1)
    if version != 1:
        unestablished("program version %d" % version)
    scalar_style = spec_version >= 9
    p = Printer(scalar_style)
    p.lines.append("program(%s)" % ("1.3" if scalar_style else "1.0"))
    attributes = program.map(4)
    if [k for k, _ in attributes] != ["buildInfo"]:
        unestablished("program attributes other than buildInfo")
    info = attributes[0][1]
    pairs = []
    for kv in info.msg(3).msg(4).msgs(1):
        k = kv.msg(1).msg(3).msg(1).msg(4).all(1)[0].decode()
        v = kv.msg(2).msg(3).msg(1).msg(4).all(1)[0].decode()
        pairs.append((k, v))
    pairs = sorted(dict(pairs + extra_build_info).items())
    kv_type = "string" if scalar_style else "tensor<string, []>"
    p.lines.append("[buildInfo = dict<%s, %s>({%s})]" % (kv_type, kv_type, ", ".join('{"%s", "%s"}' % (p.escape(k), p.escape(v)) for k, v in pairs)))
    p.lines.append("{")
    functions = sorted(program.map(2), key=lambda kv: kv[0])
    names = []
    for _, function in functions:
        names += [i.str(1) for i in function.msgs(1)]
        for _, b in function.map(3):
            _declared(b, names)
    p.duplicates = {n for n in names if names.count(n) > 1}
    for name, function in functions:
        opset = function.str(2)
        inputs = ", ".join("%s %s" % (p.value_type(i.msg(2)), i.str(1)) for i in sorted(function.msgs(1), key=lambda i: i.str(1)))
        p.lines.append("    func %s<%s>(%s) {" % (name, OPSETS.get(opset) or unestablished("opset " + opset), inputs))
        blocks = dict((k, v) for k, v in function.map(3))
        if list(blocks) != [opset]:
            unestablished("block specializations other than the opset's")
        outputs = p.block(blocks[opset], 12)
        p.lines.append("        } -> (%s);" % outputs)
    p.lines.append("}")
    return "\n".join(p.lines)



# Written into model.mil's buildInfo, as Xcode 27.0's coremlc writes it.
COREMLC_BUILD_INFO = [("coremlc-component-MIL", "3600.16.1"), ("coremlc-version", "3600.25.1")]


def model_mil(model, extra_build_info=COREMLC_BUILD_INFO):
    return print_program(model.msg(502), model.int(1), extra_build_info)


# --- CoreFoundation dictionaries (metadata.json) -------------------------------

_M64 = (1 << 64) - 1
_CF_SIZES = [3, 7, 13, 23, 41, 71, 127, 191, 251, 383, 631, 1087]
_CF_CAPACITIES = [3, 6, 11, 19, 32, 52, 85, 118, 155, 237, 390, 672]


def cf_string_hash(text):
    """CFHash of a string (the published CFString.c)."""
    raw = text.encode("utf-16-le")
    u = [raw[i] | (raw[i + 1] << 8) for i in range(0, len(raw), 2)]
    n = len(u)
    r = n

    def four(r, i):
        return (r * 67503105 + u[i] * 16974593 + u[i + 1] * 66049 + u[i + 2] * 257 + u[i + 3]) & _M64
    if n <= 96:
        i = 0
        while i < (n & ~3):
            r = four(r, i)
            i += 4
        while i < n:
            r = (r * 257 + u[i]) & _M64
            i += 1
    else:
        for start in (0, (n >> 1) - 16, n - 32):
            for i in range(start, start + 32, 4):
                r = four(r, i)
    return (r + (r << (n & 31))) & _M64


def cf_dictionary_order(keys):
    """The order a mutable CoreFoundation dictionary filled with `keys`, in
    that order, enumerates them: linear probing over the published
    CFBasicHash sizes, a full 3-bucket table growing before the next
    insertion and larger ones as soon as one takes them past their
    capacity, rehashing in enumeration order."""
    hashes = {k: cf_string_hash(k) for k in keys}
    level = 0
    table = [None] * _CF_SIZES[level]

    def place(tbl, k):
        i = hashes[k] % len(tbl)
        while tbl[i] is not None:
            i = (i + 1) % len(tbl)
        tbl[i] = k

    def grow():
        nonlocal level, table
        level += 1
        old = [x for x in table if x is not None]
        table = [None] * _CF_SIZES[level]
        for x in old:
            place(table, x)
    count = 0
    for k in keys:
        if _CF_SIZES[level] == 3 and count == 3:
            grow()
        place(table, k)
        count += 1
        if _CF_SIZES[level] > 3 and count > _CF_CAPACITIES[level]:
            grow()
    return [x for x in table if x is not None]


class CFDict:
    """A dictionary written in CoreFoundation's enumeration order for the
    order its keys were inserted in."""

    def __init__(self, pairs):
        self.pairs = list(pairs)

    def items(self):
        values = dict(self.pairs)
        return [(k, values[k]) for k in cf_dictionary_order([k for k, _ in self.pairs])]


def ns_json(value, indent=0):
    """NSJSONSerialization's pretty printing: two spaces, " : ", raw UTF-8,
    empty arrays split over two lines."""
    pad = "  " * indent
    if isinstance(value, CFDict):
        items = value.items()
        if not items:
            return "{\n\n" + pad + "}"
        return "{\n" + ",\n".join("%s  %s : %s" % (pad, _json_string(k), ns_json(v, indent + 1)) for k, v in items) + "\n" + pad + "}"
    if isinstance(value, list):
        if not value:
            return "[\n\n" + pad + "]"
        return "[\n" + ",\n".join(pad + "  " + ns_json(v, indent + 1) for v in value) + "\n" + pad + "]"
    if isinstance(value, bool):
        unestablished("a JSON boolean")
    if isinstance(value, int):
        return str(value)
    if isinstance(value, str):
        return _json_string(value)
    unestablished("a JSON %s" % type(value).__name__)


def _json_string(text):
    out = []
    for ch in text:
        if ch == '"':
            out.append('\\"')
        elif ch == "\\":
            out.append("\\\\")
        elif ch == "/":
            out.append("\\/")
        elif ord(ch) < 0x20:
            unestablished("a control character in metadata")
        else:
            out.append(ch)
    return '"' + "".join(out) + '"'


# --- Model description ----------------------------------------------------------

ARRAY_TYPES = {65568: "Float32", 65600: "Double", 131104: "Int32", 65552: "Float16"}
COLOR_SPACES = {10: ("Grayscale", "0"), 20: ("RGB", "1"), 30: ("BGR", "1")}

# Keys in the order coremlc inserts them (CoreFoundation's enumeration
# order depends on it where keys collide), fitted to Xcode 27.0's output.
TOP_KEYS = ["license", "classLabels", "outputSchema", "computePrecision", "modelType", "generatedClassName",
            "shortDescription", "author", "version", "metadataOutputVersion", "inputSchema", "stateSchema",
            "isUpdatable", "specificationVersion", "modelParameters", "mlProgramOperationTypeHistogram",
            "userDefinedMetadata", "method", "availability", "storagePrecision"]
SCHEMA_KEYS = ["isColor", "colorspace", "hasSizeFlexibility", "shape", "keyType", "type", "dataType", "width",
               "height", "name", "isOptional", "hasShapeFlexibility", "shortDescription", "formattedType"]

# Deployment availability by specification version.
AVAILABILITY = {
    6: {"macOS": "12.0", "tvOS": "15.0", "visionOS": "1.0", "watchOS": "8.0", "iOS": "15.0", "macCatalyst": "15.0"},
    7: {"macOS": "13.0", "tvOS": "16.0", "visionOS": "1.0", "watchOS": "9.0", "iOS": "16.0", "macCatalyst": "16.0"},
    8: {"macOS": "14.0", "tvOS": "17.0", "visionOS": "1.0", "watchOS": "10.0", "iOS": "17.0", "macCatalyst": "17.0"},
    9: {"macOS": "15.0", "tvOS": "18.0", "visionOS": "2.0", "watchOS": "11.0", "iOS": "18.0", "macCatalyst": "18.0"},
}
AVAILABILITY_KEYS = ["iOS", "macCatalyst", "macOS", "tvOS", "visionOS", "watchOS"]


def ordered(pairs, order):
    keys = [k for k, _ in pairs]
    unknown = [k for k in keys if k not in order]
    if unknown:
        unestablished("metadata keys %s" % ", ".join(unknown))
    values = dict(pairs)
    return CFDict([(k, values[k]) for k in order if k in values])


def feature(f):
    t = f.msg(3) or unestablished("a feature without a type")
    optional = "1" if t.int(1000) else "0"
    pairs = [("isOptional", optional), ("name", f.str(1)), ("shortDescription", f.str(2))]
    if t.has(5):
        a = t.msg(5)
        if a.has(21) or a.has(31) or a.has(41) or a.has(51) or a.has(61):
            unestablished("a multi-array with flexible shapes or a default value")
        dtype = ARRAY_TYPES.get(a.int(2)) or unestablished("array data type %d" % a.int(2))
        shape = []
        raw = a.all(1)
        for r in raw:
            if isinstance(r, bytes):
                i = 0
                while i < len(r):
                    v, i = _varint(r, i)
                    shape.append(v)
            else:
                shape.append(r)
        pairs += [("type", "MultiArray"), ("dataType", dtype), ("shape", "[" + ", ".join(map(str, shape)) + "]"),
                  ("hasShapeFlexibility", "0"),
                  ("formattedType", "MultiArray (%s)" % " ".join([dtype] + ([" × ".join(map(str, shape))] if shape else [])))]
    elif t.has(4):
        im = t.msg(4)
        if im.has(21) or im.has(31):
            unestablished("an image with flexible sizes")
        space, color = COLOR_SPACES.get(im.int(3)) or unestablished("color space %d" % im.int(3))
        w, h = im.int(1), im.int(2)
        pairs += [("type", "Image"), ("width", str(w)), ("height", str(h)), ("colorspace", space), ("isColor", color),
                  ("hasSizeFlexibility", "0"),
                  ("formattedType", "Image (%s %d × %d)" % ("Color" if color == "1" else "Grayscale", w, h))]
        if w != h:
            unestablished("a non-square image's formatted type")
    elif t.has(1):
        pairs += [("type", "Int64"), ("formattedType", "Int64")]
    elif t.has(3):
        pairs += [("type", "String"), ("formattedType", "String")]
    elif t.has(6):
        d = t.msg(6)
        key = "Int64" if d.has(1) else "String"
        pairs += [("type", "Dictionary"), ("keyType", key), ("formattedType", "Dictionary (%s → Double)" % key)]
    else:
        unestablished("a feature type other than a multi-array, image, Int64, String or dictionary")
    return ordered(pairs, SCHEMA_KEYS)


OP_NAMES = None


def histogram_name(op, spec_version):
    global OP_NAMES
    if OP_NAMES is None:
        with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "coremlc_ops.json")) as f:
            OP_NAMES = json.load(f)
    entry = OP_NAMES.get(op) or unestablished("operation %s's histogram name" % op)
    target = {6: 15, 7: 16, 8: 17, 9: 18}.get(spec_version) or unestablished("specification version %d" % spec_version)
    versions = [v for v in entry["versions"] if v <= target]
    if versions:
        return "Ios%d.%s" % (max(versions), entry["camel"])
    return entry["base"] or unestablished("operation %s's histogram name" % op)


def _walk_ops(block, out):
    for op in block.msgs(3):
        out.append(op)
        for b in op.msgs(4):
            _walk_ops(b, out)


PRECISION_NAMES = {"fp16": "Float16", "fp32": "Float32", "fp64": "Float64", "int32": "Int32", "int64": "Int64", "uint16": "UInt16"}


def program_facts(program, spec_version):
    """(histogram pairs in first-use order, compute precision, class labels)."""
    ops = []
    declared = []
    # Compute precision counts what operations produce (constants
    # included), not the function's inputs.
    for _, function in sorted(program.map(2), key=lambda kv: kv[0]):
        for _, b in function.map(3):
            _walk_ops(b, ops)
    counts = {}
    labels = None
    for op in ops:
        kind = op.str(1)
        declared += [o.msg(2) for o in op.msgs(3)]
        if kind == "const":
            continue
        if kind == "classify":
            labels = "classify"
            continue
        name = histogram_name(kind, spec_version)
        counts[name] = counts.get(name, 0) + 1
    # Storage precision counts the weights file's constants that an
    # operation reads; one only returned counts for nothing.
    read = set()
    for op in ops:
        for _, argument in op.map(2):
            for b in argument.msgs(1):
                if b.has(1):
                    read.add(b.str(1))
    stored = set()
    for op in ops:
        if op.str(1) != "const":
            continue
        out = op.msgs(3)[0]
        for _, v in op.map(5):
            if v.has(5) and out.str(1) in read:
                dtype = DATA_TYPES.get(v.msg(2).msg(1).int(1))
                stored.add(PRECISION_NAMES.get(dtype) or unestablished("weights stored as %s" % dtype))
    types = set()

    def collect(t):
        if t is None:
            return
        if t.has(1):
            dtype = DATA_TYPES.get(t.msg(1).int(1))
            if dtype not in ("bool", "string"):
                types.add(PRECISION_NAMES.get(dtype) or unestablished("compute precision with %s" % dtype))
        elif t.has(4):
            collect(t.msg(4).msg(1))
            collect(t.msg(4).msg(2))
        else:
            unestablished("compute precision with a list, tuple or state type")
    for t in declared:
        collect(t)
    names = sorted(types)
    precision = names[0] if len(names) == 1 else "Mixed (%s)" % ", ".join(names)
    storage = sorted(stored)
    if len(storage) > 1:
        unestablished("weights stored as %s" % ", ".join(storage))
    storage = storage[0] if storage else None
    return list(counts.items()), precision, labels, storage


def class_labels(program):
    for _, function in program.map(2):
        for _, b in function.map(3):
            ops = []
            _walk_ops(b, ops)
            consts = {}
            for op in ops:
                if op.str(1) == "const":
                    consts[op.msgs(3)[0].str(1)] = op
            for op in ops:
                if op.str(1) == "classify":
                    for name, argument in op.map(2):
                        if name == "classes":
                            binding = argument.msgs(1)[0]
                            value = binding.msg(2) if binding.has(2) else dict(consts[binding.str(1)].map(5))["val"]
                            immediate = value.msg(3)
                            if immediate.has(3):
                                # A list of scalars.
                                out = []
                                for item in immediate.msg(3).msgs(1):
                                    t = item.msg(3).msg(1)
                                    if t.has(4):
                                        out.append(t.msg(4).all(1)[0].decode())
                                    elif t.has(5):
                                        v, _ = _varint(t.msg(5).bytes(1), 0)
                                        out.append(v - (1 << 64) if v >= 1 << 63 else v)
                                    else:
                                        unestablished("class labels of another type")
                                return out
                            tensor = immediate.msg(1)
                            if tensor.has(4):
                                return [s.decode() for s in tensor.msg(4).all(1)]
                            if tensor.has(5):
                                raw = tensor.msg(5).bytes(1)
                                out, i = [], 0
                                while i < len(raw):
                                    v, i = _varint(raw, i)
                                    out.append(v - (1 << 64) if v >= 1 << 63 else v)
                                return out
                            unestablished("class labels of another type")
    unestablished("a classifier without a classify operation")


def metadata_json(model, name):
    spec_version = model.int(1)
    description = model.msg(2) or Message(b"")
    if description.has(13) or description.has(20) or description.has(50):
        unestablished("a model with states, functions or training inputs")
    meta = description.msg(100) or Message(b"")
    program = model.msg(502)
    histogram, precision, classifier, storage = program_facts(program, spec_version)
    if model.int(10):
        unestablished("an updatable model")
    pairs = [("metadataOutputVersion", "3.0")]
    for key, number in (("shortDescription", 1), ("author", 3), ("license", 4), ("version", 2)):
        if meta.has(number) and meta.str(number):
            pairs.append((key, meta.str(number)))
    pairs += [
        ("outputSchema", [feature(f) for f in description.msgs(10)]),
        ("modelParameters", []),
        ("specificationVersion", spec_version),
        ("mlProgramOperationTypeHistogram", CFDict(histogram)),
        ("computePrecision", precision),
        ("isUpdatable", "0"),
        ("stateSchema", []),
        ("availability", CFDict([(k, (AVAILABILITY.get(spec_version) or unestablished("availability for specification %d" % spec_version))[k]) for k in AVAILABILITY_KEYS])),
        ("modelType", CFDict([("name", "MLModelType_mlProgram")])),
        ("inputSchema", [feature(f) for f in description.msgs(1)]),
        ("generatedClassName", generated_class_name(name)),
        ("method", "predict"),
    ]
    # Xcode enumerates this map in a randomly seeded order, so its output
    # varies from run to run when it has more than one entry; this keeps the
    # model's own order.
    pairs.append(("userDefinedMetadata", CFDict(meta.map_str(100))))
    if classifier:
        pairs.append(("classLabels", class_labels(program)))
    if storage:
        pairs.append(("storagePrecision", storage))
    return "[\n  " + ns_json(ordered(pairs, TOP_KEYS), 1) + "\n]"


def generated_class_name(name):
    if not (name.replace("_", "a").isalnum() and name.isascii() and not name[0].isdigit()):
        unestablished("the generated class name of a model named %r" % name)
    return name


# --- Compiled files ---------------------------------------------------------------

def _u32(v):
    return struct.pack("<I", v)


def _u64(v):
    return struct.pack("<Q", v)


def canonical_description(model):
    """The description as coremlc re-encodes it: the model's own bytes; the
    user-defined metadata map (which Xcode writes in a randomly seeded
    order) is kept in the model's order."""
    for number, wire, value in model.fields:
        if number == 2:
            return value
    return b""


def coremldata(model):
    spec_version = model.int(1)
    header_version = {6: 5, 7: 5, 8: 8, 9: 9}.get(spec_version) or unestablished("specification version %d" % spec_version)
    description = canonical_description(model)
    head = (_u32(502) + _u32(header_version) + b"\0" * 20 + _u64(7) + b"generic" + _u32(spec_version) + b"\0" * 28
            + _u64(len(description)) + description)
    classifier = classifier_facts(model.msg(502))
    if classifier is None:
        return head + _u32(502) + b"\0" * 12
    # A classifier's labels and the output its probabilities come from.
    labels, probabilities = classifier
    if all(isinstance(x, str) for x in labels):
        body = _u64(100) + _u64(len(labels)) + b"".join(_u64(len(x.encode())) + x.encode() for x in labels)
    else:
        body = _u64(101) + _u64(len(labels)) + b"".join(struct.pack("<q", x) for x in labels)
    return head + _u32(403) + _u32(0) + body + _u64(len(probabilities.encode())) + probabilities.encode() + _u64(0)


def classifier_facts(program):
    """(labels, probabilities' output name) of an ML Program classifier, or None."""
    for _, function in program.map(2):
        for _, b in function.map(3):
            ops = []
            _walk_ops(b, ops)
            for op in ops:
                if op.str(1) == "classify":
                    return class_labels(program), dict(op.map(2))["probabilities"].msgs(1)[0].str(1)
    return None


def analytics(model_bytes, model_name):
    def s(text):
        raw = text.encode()
        return _u64(len(raw)) + raw
    sections = [
        ("NeuralNetworkModelDetails", [("containsCustomLayer", "0"), ("modelDimension", "0")]),
        ("SpecificationDetails", [("modelHash", base64.b64encode(hashlib.sha256(model_bytes).digest()).decode()), ("modelName", model_name)]),
    ]
    out = b""
    for name, entries in sections:
        out += s(name) + _u64(len(entries)) + b"".join(s(k) + s(v) for k, v in entries)
    return out


PARTIAL_INFO_PLIST = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>LSBundleContainsCoreMLmlmodelc</key>
	<true/>
</dict>
</plist>
"""


def compile_package(package, outdir):
    with open(os.path.join(package, "Manifest.json")) as f:
        manifest = json.load(f)
    root = manifest["itemInfoEntries"][manifest["rootModelIdentifier"]]
    model_path = os.path.join(package, "Data", root["path"])
    with open(model_path, "rb") as f:
        model_bytes = f.read()
    model = Message(model_bytes)
    if model.has(500) or model.has(403) or model.has(303):
        unestablished("a neural network (an older Core ML format; convert the model to an ML Program)")
    if not model.has(502):
        unestablished("a model that isn't an ML Program")
    name = os.path.splitext(os.path.basename(package.rstrip("/")))[0]
    files = {
        "model.mil": model_mil(model).encode("utf-8"),
        "coremldata.bin": coremldata(model),
        "metadata.json": metadata_json(model, name).encode("utf-8"),
        "analytics/coremldata.bin": analytics(model_bytes, os.path.splitext(os.path.basename(root["path"]))[0]),
    }
    bundle = os.path.join(outdir, name + ".mlmodelc")
    for rel, data in files.items():
        os.makedirs(os.path.dirname(os.path.join(bundle, rel)), exist_ok=True)
        with open(os.path.join(bundle, rel), "wb") as f:
            f.write(data)
    weights = os.path.join(os.path.dirname(model_path), "weights")
    if os.path.isdir(weights):
        shutil.copytree(weights, os.path.join(bundle, "weights"))



# --- coremlc generate (Swift) ----------------------------------------------------

AVAILABILITY_LINE = {
    6: "macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0",
    7: "macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0",
    8: "macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0",
    9: "macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0",
}

# Feature names Swift code can't use as they are, given a trailing "_".
SWIFT_RESERVED = {"out", "var"}

ARRAY_SWIFT = {"Float32": ("Float", "floats"), "Float16": ("Float16", "16-bit floats"), "Double": ("Double", "doubles"),
               "Int32": ("Int32", "32-bit integers")}


class Feature:
    def __init__(self, f):
        self.name = f.str(1)
        self.description = f.str(2)
        t = f.msg(3)
        if t.int(1000):
            unestablished("an optional feature's generated code")
        self.swift_name = self.name + "_" if self.name in SWIFT_RESERVED else self.name
        if not self.swift_name.replace("_", "a").isalnum() or not self.swift_name.isascii() or self.swift_name[0].isdigit():
            unestablished("generated code for a feature named %r" % self.name)
        label = self.description or self.name
        if t.has(5):
            a = t.msg(5)
            dtype = ARRAY_TYPES.get(a.int(2)) or unestablished("array data type %d" % a.int(2))
            self.element, words = ARRAY_SWIFT[dtype]
            shape = []
            for r in a.all(1):
                if isinstance(r, bytes):
                    i = 0
                    while i < len(r):
                        v, i = _varint(r, i)
                        shape.append(v)
                else:
                    shape.append(r)
            if len(shape) == 1:
                shaped = "%d element vector of %s" % (shape[0], words)
            elif len(shape) == 2:
                shaped = "%d by %d matrix of %s" % (shape[0], shape[1], words)
            elif len(shape) >= 3:
                shaped = "%s %d-dimensional array of %s" % (" × ".join(map(str, shape)), len(shape), words)
            else:
                shaped = "multidimensional array of %s" % words
            self.kind = "array"
            self.swift_type = "MLMultiArray"
            self.phrase = shaped
        elif t.has(4):
            im = t.msg(4)
            space = im.int(3)
            if space not in (10, 20, 30):
                unestablished("color space %d" % space)
            self.width, self.height = im.int(1), im.int(2)
            self.kind = "image"
            self.swift_type = "CVPixelBuffer"
            fmt = "kCVPixelFormatType_OneComponent8" if space == 10 else "kCVPixelFormatType_32BGRA"
            self.phrase = "%s (%s) image buffer, %d pixels wide by %d pixels high" % (
                "grayscale" if space == 10 else "color", fmt, self.width, self.height)
            self.cg_format = {10: "kCVPixelFormatType_OneComponent8", 20: "kCVPixelFormatType_32ARGB", 30: "kCVPixelFormatType_32BGRA"}[space]
        elif t.has(3):
            self.kind, self.swift_type, self.phrase = "string", "String", "string value"
        elif t.has(1):
            self.kind, self.swift_type, self.phrase = "int64", "Int64", "integer value"
        elif t.has(6):
            d = t.msg(6)
            key = "Int64" if d.has(1) else "String"
            self.kind = "dictionary"
            self.swift_type = "[%s : Double]" % key
            self.phrase = "dictionary of %s to doubles" % ("64-bit integers" if key == "Int64" else "strings")
        else:
            unestablished("generated code for this feature type")
        self.doc = "%s as %s" % (label, self.phrase)
        self.param_doc = ("%s as %s" % (self.description, self.phrase)) if self.description else self.phrase

    def value(self, expr):
        return {"array": "MLFeatureValue(multiArray: %s)", "image": "MLFeatureValue(pixelBuffer: %s)",
                "string": "MLFeatureValue(string: %s)", "int64": "MLFeatureValue(int64: %s)",
                "dictionary": "MLFeatureValue(dictionary: %s as [AnyHashable : NSNumber])"}[self.kind] % expr

    def read(self):
        return {"array": "provider.featureValue(for: \"%s\")!.multiArrayValue!",
                "image": "provider.featureValue(for: \"%s\")!.imageBufferValue!",
                "string": "provider.featureValue(for: \"%s\")!.stringValue",
                "int64": "provider.featureValue(for: \"%s\")!.int64Value",
                "dictionary": "provider.featureValue(for: \"%s\")!.dictionaryValue as! " + self.swift_type}[self.kind] % self.name


def swift_source(model, name):
    spec_version = model.int(1)
    description = model.msg(2) or Message(b"")
    if description.has(13) or description.has(20):
        unestablished("generated code for a model with states or functions")
    available = "@available(%s, *)" % (AVAILABILITY_LINE.get(spec_version) or unestablished("specification version %d" % spec_version))
    inputs = [Feature(f) for f in description.msgs(1)]
    outputs = [Feature(f) for f in description.msgs(10)]
    L = []
    w = L.append
    w("//\n// %s.swift\n//\n// This file was automatically generated and should not be edited.\n//\n\nimport CoreML\n\n" % name)
    # Input.
    w("/// Model Prediction Input Type\n%s\npublic class %sInput : MLFeatureProvider {\n" % (available, name))
    for f in inputs:
        w("    /// %s\n    public var %s: %s\n" % (f.doc, f.swift_name, f.swift_type))
    w("    public var featureNames: Set<String> { [%s] }\n" % ", ".join('"%s"' % f.name for f in inputs))
    w("    public func featureValue(for featureName: String) -> MLFeatureValue? {")
    for f in inputs:
        w("        if featureName == \"%s\" {\n            return %s\n        }" % (f.name, f.value(f.swift_name)))
    w("        return nil\n    }\n")
    w("    public init(%s) {" % ", ".join("%s: %s" % (f.swift_name, f.swift_type) for f in inputs))
    for f in inputs:
        w("        self.%s = %s" % (f.swift_name, f.swift_name))
    w("    }\n")
    arrays = [f for f in inputs if f.kind == "array"]
    if arrays:
        w("    public convenience init(%s) {" % ", ".join("%s: %s" % (f.swift_name, "MLShapedArray<%s>" % f.element if f.kind == "array" else f.swift_type) for f in inputs))
        w("        self.init(%s)" % ", ".join("%s: %s" % (f.swift_name, "MLMultiArray(%s)" % f.swift_name if f.kind == "array" else f.swift_name) for f in inputs))
        w("    }\n")
    images = [f for f in inputs if f.kind == "image"]
    if images:
        if len(inputs) != 1:
            unestablished("generated code for images among other inputs")
        f = images[0]
        mk = "MLFeatureValue(%s: %s, pixelsWide: %d, pixelsHigh: %d, pixelFormatType: %s, options: nil).imageBufferValue!"
        w("    public convenience init(%sWith %s: CGImage) throws {\n        self.init(%s: try %s)\n    }\n" % (f.swift_name, f.swift_name, f.swift_name, mk % ("cgImage", f.swift_name, f.width, f.height, f.cg_format)))
        w("    public convenience init(%sAt %s: URL) throws {\n        self.init(%s: try %s)\n    }\n" % (f.swift_name, f.swift_name, f.swift_name, mk % ("imageAt", f.swift_name, f.width, f.height, f.cg_format)))
        cap = f.swift_name[:1].upper() + f.swift_name[1:]
        w("    public func set%s(with %s: CGImage) throws  {\n        self.%s = try %s\n    }\n" % (cap, f.swift_name, f.swift_name, mk % ("cgImage", f.swift_name, f.width, f.height, f.cg_format)))
        w("    public func set%s(with %s: URL) throws  {\n        self.%s = try %s\n    }\n" % (cap, f.swift_name, f.swift_name, mk % ("imageAt", f.swift_name, f.width, f.height, f.cg_format)))
    w("}\n\n")
    # Output.
    w("/// Model Prediction Output Type\n%s\npublic class %sOutput : MLFeatureProvider {\n" % (available, name))
    w("    /// Source provided by CoreML\n    private let provider : MLFeatureProvider\n")
    for f in outputs:
        w("    /// %s\n    public var %s: %s {\n        %s\n    }\n" % (f.doc, f.swift_name, f.swift_type, f.read()))
        if f.kind == "array":
            w("    /// %s\n    public var %sShapedArray: MLShapedArray<%s> {\n        MLShapedArray<%s>(%s)\n    }\n" % (f.doc, f.swift_name, f.element, f.element, f.swift_name))
    w("    public var featureNames: Set<String> {\n        provider.featureNames\n    }\n")
    w("    public func featureValue(for featureName: String) -> MLFeatureValue? {\n        provider.featureValue(for: featureName)\n    }\n")
    w("    public init(%s) {" % ", ".join("%s: %s" % (f.swift_name, f.swift_type) for f in outputs))
    w("        self.provider = try! MLDictionaryFeatureProvider(dictionary: [%s])" % ", ".join('"%s" : %s' % (f.name, f.value(f.swift_name)) for f in outputs))
    w("    }\n")
    w("    public init(features: MLFeatureProvider) {\n        self.provider = features\n    }\n}\n\n")
    model_class = MODEL_CLASS.replace("{available}", available).replace("{N}", name).rstrip("\n") + "\n"
    if spec_version < 8:
        # Asynchronous prediction needs iOS 17 and its contemporaries.
        anchor = "    public func prediction(input: %sInput, options: MLPredictionOptions = MLPredictionOptions()) async throws" % name
        if model_class.count(anchor) != 1:
            unestablished("the asynchronous prediction of an older model")
        model_class = model_class.replace(anchor, "    @available(%s, *)\n" % AVAILABILITY_LINE[8] + anchor)
    w(model_class)
    # Convenience predictions.
    params = "\n".join("            - %s: %s" % (f.swift_name, f.param_doc) for f in inputs)
    doc = ("    /**\n        Make a prediction using the convenience interface\n\n        It uses the default function if the model has multiple functions.\n\n"
           "        - parameters:\n%s\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as %sOutput\n    */\n") % (params, name)
    call = "        let input_ = %sInput(%s)\n        return try prediction(input: input_)\n    }\n" % (name, ", ".join("%s: %s" % (f.swift_name, f.swift_name) for f in inputs))
    w(doc + "    public func prediction(%s) throws -> %sOutput {\n" % (", ".join("%s: %s" % (f.swift_name, f.swift_type) for f in inputs), name) + call)
    if arrays:
        w(doc + "\n    public func prediction(%s) throws -> %sOutput {\n" % (", ".join("%s: %s" % (f.swift_name, "MLShapedArray<%s>" % f.element if f.kind == "array" else f.swift_type) for f in inputs), name) + call)
    w(BATCH.replace("{N}", name))
    return "\n".join(L)


MODEL_CLASS = '/// Class for model loading and prediction\n{available}\npublic class {N} {\n    public let model: MLModel\n\n    /// URL of model assuming it was installed in the same bundle as this class\n    class var urlOfModelInThisBundle : URL {\n        let bundle = Bundle(for: self)\n        return bundle.url(forResource: "{N}", withExtension:"mlmodelc")!\n    }\n\n    /**\n        Construct {N} instance with an existing MLModel object.\n\n        Usually the application does not use this initializer unless it makes a subclass of {N}.\n        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `{N}.urlOfModelInThisBundle` to create a MLModel object to pass-in.\n\n        - parameters:\n          - model: MLModel object\n    */\n    init(model: MLModel) {\n        self.model = model\n    }\n\n    /**\n        Construct a model with configuration\n\n        - parameters:\n           - configuration: the desired model configuration\n\n        - throws: an NSError object that describes the problem\n    */\n    public convenience init(configuration: MLModelConfiguration = MLModelConfiguration()) throws {\n        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)\n    }\n\n    /**\n        Construct {N} instance with explicit path to mlmodelc file\n        - parameters:\n           - modelURL: the file url of the model\n\n        - throws: an NSError object that describes the problem\n    */\n    public convenience init(contentsOf modelURL: URL) throws {\n        try self.init(model: MLModel(contentsOf: modelURL))\n    }\n\n    /**\n        Construct a model with URL of the .mlmodelc directory and configuration\n\n        - parameters:\n           - modelURL: the file url of the model\n           - configuration: the desired model configuration\n\n        - throws: an NSError object that describes the problem\n    */\n    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {\n        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))\n    }\n\n    /**\n        Construct {N} instance asynchronously with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - configuration: the desired model configuration\n          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully\n    */\n    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<{N}, Error>) -> Void) {\n        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)\n    }\n\n    /**\n        Construct {N} instance asynchronously with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - configuration: the desired model configuration\n    */\n    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> {N} {\n        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)\n    }\n\n    /**\n        Construct {N} instance asynchronously with URL of the .mlmodelc directory with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - modelURL: the URL to the model\n          - configuration: the desired model configuration\n          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully\n    */\n    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<{N}, Error>) -> Void) {\n        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in\n            switch result {\n            case .failure(let error):\n                handler(.failure(error))\n            case .success(let model):\n                handler(.success({N}(model: model)))\n            }\n        }\n    }\n\n    /**\n        Construct {N} instance asynchronously with URL of the .mlmodelc directory with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - modelURL: the URL to the model\n          - configuration: the desired model configuration\n    */\n    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> {N} {\n        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)\n        return {N}(model: model)\n    }\n\n    /**\n        Make a prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - input: the input to the prediction as {N}Input\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as {N}Output\n    */\n    public func prediction(input: {N}Input) throws -> {N}Output {\n        try prediction(input: input, options: MLPredictionOptions())\n    }\n\n    /**\n        Make a prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - input: the input to the prediction as {N}Input\n           - options: prediction options\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as {N}Output\n    */\n    public func prediction(input: {N}Input, options: MLPredictionOptions) throws -> {N}Output {\n        let outFeatures = try model.prediction(from: input, options: options)\n        return {N}Output(features: outFeatures)\n    }\n\n    /**\n        Make an asynchronous prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - input: the input to the prediction as {N}Input\n           - options: prediction options\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as {N}Output\n    */\n    public func prediction(input: {N}Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> {N}Output {\n        let outFeatures = try await model.prediction(from: input, options: options)\n        return {N}Output(features: outFeatures)\n    }\n\n'

BATCH = '    /**\n        Make a batch prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - inputs: the inputs to the prediction as [{N}Input]\n           - options: prediction options\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as [{N}Output]\n    */\n    public func predictions(inputs: [{N}Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [{N}Output] {\n        let batchIn = MLArrayBatchProvider(array: inputs)\n        let batchOut = try model.predictions(from: batchIn, options: options)\n        var results : [{N}Output] = []\n        results.reserveCapacity(inputs.count)\n        for i in 0..<batchOut.count {\n            let outProvider = batchOut.features(at: i)\n            let result =  {N}Output(features: outProvider)\n            results.append(result)\n        }\n        return results\n    }\n}\n'


# --- coremlc generate (Objective-C) -----------------------------------------------

API_AVAILABLE = {
    6: "macos(12.0), ios(15.0), watchos(8.0), tvos(15.0)",
    7: "macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)",
    8: "macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)",
    9: "macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)",
}


def _objc(f):
    """(property attributes and type, parameter type, value expression, read expression) of a feature."""
    if f.kind == "array":
        return "(readwrite, nonatomic, strong) MLMultiArray * ", "MLMultiArray *", "featureValueWithMultiArray:self.%s", "(MLMultiArray *)[%%s featureValueForName:@\"%s\"].multiArrayValue"
    if f.kind == "image":
        return "(readwrite, nonatomic) CVPixelBufferRef ", "CVPixelBufferRef", "featureValueWithPixelBuffer:self.%s", "(CVPixelBufferRef)[%%s featureValueForName:@\"%s\"].imageBufferValue"
    if f.kind == "string":
        return "(readwrite, nonatomic, strong) NSString * ", "NSString *", "featureValueWithString:self.%s", "(NSString *)[%%s featureValueForName:@\"%s\"].stringValue"
    if f.kind == "int64":
        return "(readwrite, nonatomic) int64_t ", "int64_t", "featureValueWithInt64:self.%s", "[%%s featureValueForName:@\"%s\"].int64Value"
    key = "NSNumber *" if "Int64" in f.swift_type else "NSString *"
    t = "NSDictionary<%s, NSNumber *> *" % key
    return "(readwrite, nonatomic, strong) %s " % t, t, "featureValueWithDictionary:self.%s error:nil", "(" + t + ")[%%s featureValueForName:@\"%s\"].dictionaryValue"


def _selector(prefix, features, value):
    """`prefixFirst:(value) second:(value) ...`: the first part capitalizes
    the feature's own name, later parts use the name the code gives it."""
    parts = []
    for k, f in enumerate(features):
        label = prefix + f.name[:1].upper() + f.name[1:] if k == 0 else f.swift_name
        parts.append("%s:%s" % (label, value(f)))
    return " ".join(parts)


def objc_sources(model, name):
    spec_version = model.int(1)
    description = model.msg(2) or Message(b"")
    if description.has(13) or description.has(20):
        unestablished("generated code for a model with states or functions")
    available = "API_AVAILABLE(%s) __attribute__((visibility(\"hidden\")))" % (API_AVAILABLE.get(spec_version) or unestablished("specification version %d" % spec_version))
    inputs = [Feature(f) for f in description.msgs(1)]
    outputs = [Feature(f) for f in description.msgs(10)]
    for f in inputs + outputs:
        if not f.name[:1].isalpha():
            unestablished("generated code for a feature named %r" % f.name)
    images = [f for f in inputs if f.kind == "image"]
    if images and len(inputs) != 1:
        unestablished("generated code for images among other inputs")
    if any(f.kind == "image" for f in outputs):
        unestablished("generated code for image outputs")
    H = []
    H.append("//\n// %s.h\n//\n// This file was automatically generated and should not be edited.\n//\n\n#import <Foundation/Foundation.h>\n#import <CoreML/CoreML.h>\n#include <stdint.h>\n#include <os/log.h>\n\nNS_ASSUME_NONNULL_BEGIN\n" % name)
    for kind, features in (("Input", inputs), ("Output", outputs)):
        H.append("/// Model Prediction %s Type\n%s\n@interface %s%s : NSObject<MLFeatureProvider>\n" % (kind, available, name, kind))
        H.append("\n\n".join("/// %s\n@property %s%s;" % (f.doc, _objc(f)[0], f.swift_name) for f in features))
        H[-1] += "\n- (instancetype)init NS_UNAVAILABLE;\n- (instancetype)%s NS_DESIGNATED_INITIALIZER;\n" % _selector("initWith", features, lambda f: "(%s)%s" % (_objc(f)[1], f.swift_name))
        if kind == "Input" and images:
            f = images[0]
            cap = f.name[:1].upper() + f.name[1:]
            H.append("- (nullable instancetype)initWith%sFromCGImage:(CGImageRef)%s error:(NSError * _Nullable __autoreleasing * _Nullable)error;\n" % (cap, f.swift_name))
            H.append("- (nullable instancetype)initWith%sAtURL:(NSURL *)%sURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;\n" % (cap, f.swift_name))
            H.append("-(BOOL)set%sWithCGImage:(CGImageRef)%s error:(NSError * _Nullable __autoreleasing * _Nullable)error ;" % (cap, f.swift_name))
            H.append("-(BOOL)set%sWithURL:(NSURL *)%sURL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;" % (cap, f.swift_name))
        H.append("@end\n")
    header_model = H_MODEL.replace("{available}", available).replace("{N}", name)
    if spec_version < 8:
        for anchor in ("completionHandler:(void (^)(%sOutput * _Nullable output, NSError * _Nullable error))completionHandler;" % name,):
            if header_model.count(anchor) != 2:
                unestablished("the asynchronous predictions of an older model")
            header_model = header_model.replace(anchor, anchor[:-1] + " API_AVAILABLE(%s) __attribute__((visibility(\"hidden\")));" % API_AVAILABLE[8])
    H.append(header_model.rstrip("\n"))
    params = "\n".join("    @param %s %s" % (f.swift_name, f.param_doc) for f in inputs)
    H.append("\n/**\n    Make a prediction using the convenience interface\n%s\n    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.\n    @return the prediction as %sOutput\n*/\n- (nullable %sOutput *)%s error:(NSError * _Nullable __autoreleasing * _Nullable)error;\n" % (
        params, name, name, _selector("predictionFrom", inputs, lambda f: "(%s)%s" % (_objc(f)[1], f.swift_name))))
    H.append(H_BATCH.replace("{N}", name))
    header = "\n".join(H)

    def make_output(src):
        return "[[%sOutput alloc] %s]" % (name, _selector("initWith", outputs, lambda f: _objc(f)[3] % src % f.name if False else (_objc(f)[3] % f.name) % src))

    M = []
    M.append("//\n// %s.m\n//\n// This file was automatically generated and should not be edited.\n//\n\n#if !__has_feature(objc_arc)\n#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)\n#endif\n\n#import \"%s.h\"\n" % (name, name))
    for kind, features in (("Input", inputs), ("Output", outputs)):
        M.append("@implementation %s%s\n" % (name, kind))
        M.append("- (instancetype)%s {\n    self = [super init];\n    if (self) {" % _selector("initWith", features, lambda f: "(%s)%s" % (_objc(f)[1], f.swift_name)))
        for f in features:
            M.append("        _%s = %s;" % (f.swift_name, f.swift_name))
            if f.kind == "image":
                M.append("        CVPixelBufferRetain(_%s);" % f.swift_name)
        M.append("    }\n    return self;\n}\n")
        if kind == "Input" and images:
            M.append(_image_methods(images[0]))
        M.append("- (NSSet<NSString *> *)featureNames {\n    return [NSSet setWithArray:@[%s]];\n}\n" % ", ".join('@"%s"' % f.name for f in features))
        M.append("- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {")
        for f in features:
            M.append("    if ([featureName isEqualToString:@\"%s\"]) {\n        return [MLFeatureValue %s];\n    }" % (f.name, _objc(f)[2] % f.swift_name))
        M.append("    return nil;\n}\n\n@end\n")
    M.append(M_MODEL.replace("{N}", name).rstrip("\n") + "\n")
    M.append(("- (nullable {N}Output *)predictionFromFeatures:({N}Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n"
              "    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];\n}\n").replace("{N}", name))
    M.append(("- (nullable {N}Output *)predictionFromFeatures:({N}Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n"
              "    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];\n"
              "    if (!outFeatures) { return nil; }\n    return %s;\n}\n").replace("{N}", name) % make_output("outFeatures"))
    for opts in ("", " options:(MLPredictionOptions *)options"):
        call = " options:options" if opts else ""
        M.append(("- (void)predictionFromFeatures:({N}Input *)input%s completionHandler:(void (^)({N}Output * _Nullable output, NSError * _Nullable error))completionHandler {\n"
                  "    [self.model predictionFromFeatures:input%s completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {\n"
                  "        if (prediction != nil) {\n            {N}Output *output = %s;\n            completionHandler(output, predictionError);\n"
                  "        } else {\n            completionHandler(nil, predictionError);\n        }\n    }];\n}\n").replace("{N}", name) % (opts, call, make_output("prediction")))
    M.append("- (nullable %sOutput *)%s error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n    %sInput *input_ = [[%sInput alloc] %s];\n    return [self predictionFromFeatures:input_ error:error];\n}\n" % (
        name, _selector("predictionFrom", inputs, lambda f: "(%s)%s" % (_objc(f)[1], f.swift_name)), name, name, _selector("initWith", inputs, lambda f: f.swift_name)))
    M.append(("- (nullable NSArray<{N}Output *> *)predictionsFromInputs:(NSArray<{N}Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n"
              "    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];\n"
              "    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];\n"
              "    if (!outBatch) { return nil; }\n"
              "    NSMutableArray<{N}Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];\n"
              "    for (NSInteger i = 0; i < outBatch.count; i++) {\n"
              "        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];\n"
              "        {N}Output * result = %s;\n"
              "        [results addObject:result];\n    }\n    return results;\n}\n\n@end\n").replace("{N}", name) % make_output("resultProvider"))
    return header, "\n".join(M)


def _image_methods(f):
    cap = f.name[:1].upper() + f.name[1:]
    n = f.swift_name
    make = "[MLFeatureValue featureValueWith%s:%s pixelsWide:%d pixelsHigh:%d pixelFormatType:%s options:nil error:&localError]"
    out = []
    for source, label, arg in (("CGImage", "FromCGImage:(CGImageRef)%s" % n, n), ("ImageAtURL", "AtURL:(NSURL *)%sURL" % n, n + "URL")):
        out.append(("- (nullable instancetype)initWith%s%s error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n"
                    "    if (self) {\n        NSError *localError;\n        BOOL result = YES;\n        id retVal = nil;\n        @autoreleasepool {\n            do {\n"
                    "                MLFeatureValue * __%s = %s;\n                if (__%s == nil) {\n                    result = NO;\n                    break;\n                }\n"
                    "                retVal = [self initWith%s:(CVPixelBufferRef)__%s.imageBufferValue];\n            }\n            while(0);\n        }\n"
                    "        if (error != NULL) {\n            *error = localError;\n        }\n        return result ? retVal : nil;\n    }\n    return self;\n}\n") % (
                    cap, label, n, make % (source, arg, f.width, f.height, f.cg_format), n, cap, n))
    for source, label, arg in (("CGImage", "CGImage:(CGImageRef)%s" % n, n), ("ImageAtURL", "URL:(NSURL *)%sURL" % n, n + "URL")):
        out.append(("-(BOOL)set%sWith%s error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n    NSError *localError;\n    BOOL result = NO;\n    @autoreleasepool {\n"
                    "        MLFeatureValue * __%s = %s;\n        if (__%s != nil) {\n            CVPixelBufferRelease(self.%s);\n"
                    "            self.%s =  (CVPixelBufferRef)__%s.imageBufferValue;\n            CVPixelBufferRetain(self.%s);\n            result = YES;\n        }\n    }\n"
                    "    if (error != NULL) {\n        *error = localError;\n    }\n    return result;\n}\n") % (
                    cap, label, n, make % (source, arg, f.width, f.height, f.cg_format), n, n, n, n, n))
    return "- (void)dealloc {\n    CVPixelBufferRelease(_%s);\n}\n\n" % n + "\n".join(out)

H_MODEL = '/// Class for model loading and prediction\n{available}\n@interface {N} : NSObject\n@property (readonly, nonatomic, nullable) MLModel * model;\n\n/**\n    URL of the underlying .mlmodelc directory.\n*/\n+ (nullable NSURL *)URLOfModelInThisBundle;\n\n/**\n    Initialize {N} instance from an existing MLModel object.\n\n    Usually the application does not use this initializer unless it makes a subclass of {N}.\n    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.\n*/\n- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;\n\n/**\n    Initialize {N} instance with the model in this bundle.\n*/\n- (nullable instancetype)init;\n\n/**\n    Initialize {N} instance with the model in this bundle.\n\n    @param configuration The model configuration object\n    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.\n*/\n- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;\n\n/**\n    Initialize {N} instance from the model URL.\n\n    @param modelURL URL to the .mlmodelc directory for {N}.\n    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.\n*/\n- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;\n\n/**\n    Initialize {N} instance from the model URL.\n\n    @param modelURL URL to the .mlmodelc directory for {N}.\n    @param configuration The model configuration object\n    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.\n*/\n- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;\n\n/**\n    Construct {N} instance asynchronously with configuration.\n    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n    @param configuration The model configuration\n    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid {N} instance or NSError object.\n*/\n+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)({N} * _Nullable model, NSError * _Nullable error))handler;\n\n/**\n    Construct {N} instance asynchronously with URL of .mlmodelc directory and optional configuration.\n\n    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n    @param modelURL The model URL.\n    @param configuration The model configuration\n    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid {N} instance or NSError object.\n*/\n+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)({N} * _Nullable model, NSError * _Nullable error))handler;\n\n/**\n    Make a prediction using the standard interface\n    @param input an instance of {N}Input to predict from\n    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.\n    @return the prediction as {N}Output\n*/\n- (nullable {N}Output *)predictionFromFeatures:({N}Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;\n\n/**\n    Make a prediction using the standard interface\n    @param input an instance of {N}Input to predict from\n    @param options prediction options\n    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.\n    @return the prediction as {N}Output\n*/\n- (nullable {N}Output *)predictionFromFeatures:({N}Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;\n\n/**\n    Make an asynchronous prediction using the standard interface\n    @param input an instance of {N}Input to predict from\n    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.\n*/\n- (void)predictionFromFeatures:({N}Input *)input completionHandler:(void (^)({N}Output * _Nullable output, NSError * _Nullable error))completionHandler;\n\n/**\n    Make an asynchronous prediction using the standard interface\n    @param input an instance of {N}Input to predict from\n    @param options prediction options\n    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.\n*/\n- (void)predictionFromFeatures:({N}Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)({N}Output * _Nullable output, NSError * _Nullable error))completionHandler;\n\n'

H_BATCH = '/**\n    Batch prediction\n    @param inputArray array of {N}Input instances to obtain predictions from\n    @param options prediction options\n    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.\n    @return the predictions as NSArray<{N}Output *>\n*/\n- (nullable NSArray<{N}Output *> *)predictionsFromInputs:(NSArray<{N}Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;\n@end\n\nNS_ASSUME_NONNULL_END\n'

M_MODEL = '@implementation {N}\n\n\n/**\n    URL of the underlying .mlmodelc directory.\n*/\n+ (nullable NSURL *)URLOfModelInThisBundle {\n    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"{N}" ofType:@"mlmodelc"];\n    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load {N}.mlmodelc in the bundle resource"); return nil; }\n    return [NSURL fileURLWithPath:assetPath];\n}\n\n\n/**\n    Initialize {N} instance from an existing MLModel object.\n\n    Usually the application does not use this initializer unless it makes a subclass of {N}.\n    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.\n*/\n- (instancetype)initWithMLModel:(MLModel *)model {\n    if (model == nil) {\n        return nil;\n    }\n    self = [super init];\n    if (self != nil) {\n        _model = model;\n    }\n    return self;\n}\n\n\n/**\n    Initialize {N} instance with the model in this bundle.\n*/\n- (nullable instancetype)init {\n    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];\n}\n\n\n/**\n    Initialize {N} instance with the model in this bundle.\n\n    @param configuration The model configuration object\n    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.\n*/\n- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];\n}\n\n\n/**\n    Initialize {N} instance from the model URL.\n\n    @param modelURL URL to the .mlmodelc directory for {N}.\n    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.\n*/\n- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];\n    if (model == nil) { return nil; }\n    return [self initWithMLModel:model];\n}\n\n\n/**\n    Initialize {N} instance from the model URL.\n\n    @param modelURL URL to the .mlmodelc directory for {N}.\n    @param configuration The model configuration object\n    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.\n*/\n- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];\n    if (model == nil) { return nil; }\n    return [self initWithMLModel:model];\n}\n\n\n/**\n    Construct {N} instance asynchronously with configuration.\n    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n    @param configuration The model configuration\n    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid {N} instance or NSError object.\n*/\n+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)({N} * _Nullable model, NSError * _Nullable error))handler {\n    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]\n              configuration:configuration\n          completionHandler:handler];\n}\n\n\n/**\n    Construct {N} instance asynchronously with URL of .mlmodelc directory and optional configuration.\n\n    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n    @param modelURL The model URL.\n    @param configuration The model configuration\n    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid {N} instance or NSError object.\n*/\n+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)({N} * _Nullable model, NSError * _Nullable error))handler {\n    [MLModel loadContentsOfURL:modelURL\n                 configuration:configuration\n             completionHandler:^(MLModel *model, NSError *error) {\n        if (model != nil) {\n            {N} *typedModel = [[{N} alloc] initWithMLModel:model];\n            handler(typedModel, nil);\n        } else {\n            handler(nil, error);\n        }\n    }];\n}\n\n'


def read_package(package):
    with open(os.path.join(package, "Manifest.json")) as f:
        manifest = json.load(f)
    root = manifest["itemInfoEntries"][manifest["rootModelIdentifier"]]
    with open(os.path.join(package, "Data", root["path"]), "rb") as f:
        return Message(f.read())


def generate(argv):
    source, outdir, language, public = None, None, "Objective-C", False
    rest = list(argv)
    while rest:
        arg = rest.pop(0)
        if arg == "--public-access":
            public = True
        elif arg == "--language" and rest:
            language = rest.pop(0)
        elif source is None:
            source = arg
        elif outdir is None:
            outdir = arg
        else:
            print("error: coremlc: unsupported argument %s" % arg, file=sys.stderr)
            return 1
    try:
        if (language == "Swift") != public or language not in ("Swift", "Objective-C"):
            unestablished("generated %s%s code" % ("public " if public else "", language))
        if not os.path.isdir(source):
            unestablished("a .mlmodel file (only .mlpackage ML Programs are)")
        model = read_package(source)
        if not model.has(502):
            unestablished("a model that isn't an ML Program")
        name = generated_class_name(os.path.splitext(os.path.basename(source.rstrip("/")))[0])
        if language == "Swift":
            files = {name + ".swift": swift_source(model, name)}
        else:
            header, implementation = objc_sources(model, name)
            files = {name + ".h": header, name + ".m": implementation}
    except Unestablished as e:
        print("error: coremlc: %s is not generated as Xcode generates it" % e, file=sys.stderr)
        return 1
    os.makedirs(outdir, exist_ok=True)
    for filename, text in files.items():
        with open(os.path.join(outdir, filename), "w") as f:
            f.write(text)
    return 0


def main(argv):
    if argv and argv[0] == "generate":
        return generate(argv[1:])
    if len(argv) < 3 or argv[0] != "compile":
        print("error: coremlc: only `compile <model> <output-directory> [--output-partial-info-plist <plist>]` is supported off a Mac", file=sys.stderr)
        return 1
    source, outdir = argv[1], argv[2]
    plist = None
    rest = argv[3:]
    while rest:
        if rest[0] == "--output-partial-info-plist" and len(rest) > 1:
            plist = rest[1]
            rest = rest[2:]
        else:
            print("error: coremlc: unsupported option %s" % rest[0], file=sys.stderr)
            return 1
    try:
        if not os.path.isdir(source):
            unestablished("a .mlmodel file (only .mlpackage ML Programs are)")
        compile_package(source, outdir)
    except Unestablished as e:
        print("error: coremlc: %s is not compiled as Xcode compiles it" % e, file=sys.stderr)
        return 1
    if plist:
        with open(plist, "w") as f:
            f.write(PARTIAL_INFO_PLIST)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
