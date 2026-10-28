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


PRECISION_NAMES = {"fp16": "Float16", "fp32": "Float32", "fp64": "Float64", "int32": "Int32", "int64": "Int64"}


def program_facts(program, spec_version):
    """(histogram pairs in first-use order, compute precision, class labels)."""
    ops = []
    declared = []
    for _, function in sorted(program.map(2), key=lambda kv: kv[0]):
        declared += [i.msg(2) for i in function.msgs(1)]
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
    stored = set()
    for op in ops:
        for _, v in op.map(5):
            if v.has(5):
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
    storage = None if not storage else storage[0] if len(storage) == 1 else unestablished("weights stored as several types")
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


def main(argv):
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
