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
import re
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

# Byte-packed integer constants, written in decimal.
PACKED_INTEGERS = {"int8": "b", "uint8": "B", "int16": "h", "uint16": "H", "uint32": "I"}

# Integers narrower than a byte, packed from each byte's least significant
# bit up (coremltools's packing): their width and whether they're signed.
SUB_BYTE_INTEGERS = {"int4": (4, True), "uint1": (1, False), "uint2": (2, False), "uint3": (3, False),
                     "uint4": (4, False), "uint6": (6, False)}


def unpack_bits(raw, width, signed, count):
    bits = int.from_bytes(raw, "little")
    if len(raw) != (width * count + 7) // 8:
        unestablished("packed values of another length than their shape's")
    out = []
    for k in range(count):
        v = (bits >> (k * width)) & ((1 << width) - 1)
        out.append(v - (1 << width) if signed and v >> (width - 1) else v)
    return out

OPSETS = {"CoreML5": "ios15", "CoreML6": "ios16", "CoreML7": "ios17", "CoreML8": "ios18"}


def hex_float(value):
    """`value` as C's printf("%a") writes it: the fewest hex digits."""
    # Xcode writes every NaN, whatever its sign or payload, as nan.
    if value != value:
        return "nan"
    if value in (float("inf"), float("-inf")):
        return "inf" if value > 0 else "-inf"
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


# The modes resample was seen compiled with (Xcode refuses others).
RESAMPLE_MODES = {
    "coordinates_mode": ("normalized_minus_one_to_one", "normalized_zero_to_one", "unnormalized"),
    "padding_mode": ("constant", "border", "reflection", "symmetric"),
    "sampling_mode": ("bilinear", "nearest"),
}


class Printer:
    def __init__(self, scalar_style):
        self.duplicates = set()
        self.seen = {}
        self.scope = [{}]
        # Program 1.3 and later write rank-0 tensors as their element type.
        self.scalar_style = scalar_style
        self.lines = []
        # Each constant string's values by name, and the function's opset.
        self.strings = {}
        self.opset = None

    def dimension(self, d):
        if d.has(1):
            return str(d.msg(1).int(1))
        if d.has(2) and not d.msg(2).int(1):
            return "?"
        unestablished("a dimension other than a constant or unknown")

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
        if t.has(5):
            return "state<%s>" % self.value_type(t.msg(5).msg(1))
        if t.has(2):
            # A list gives its elements' type and its length.
            lst = t.msg(2)
            if not lst.has(2):
                unestablished("a list without a length")
            return "list<%s, %s>" % (self.value_type(lst.msg(1)), self.dimension(lst.msg(2)))
        unestablished("a value type other than a tensor, list or state")

    def tensor_values(self, t, value):
        """The values of a TensorValue, written as model.mil writes them."""
        tensor = t.msg(1)
        dtype = DATA_TYPES.get(tensor.int(1))
        dims = [self.dimension(d) for d in tensor.msgs(3)]
        if "?" in dims:
            unestablished("a constant of unknown shape")
        dims = [int(d) for d in dims]
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
            elif dtype in PACKED_INTEGERS:
                code = PACKED_INTEGERS[dtype]
                items = [str(x) for x in struct.unpack("<%d%s" % (len(raw) // struct.calcsize(code), code), raw)]
            elif dtype in SUB_BYTE_INTEGERS:
                count = 1
                for d in dims:
                    count *= d
                items = [str(x) for x in unpack_bits(raw, *SUB_BYTE_INTEGERS[dtype], count)]
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

    def string_input(self, op, name):
        argument = dict(op.map(2)).get(name)
        bindings = argument.msgs(1) if argument else []
        if len(bindings) == 1:
            b = bindings[0]
            if b.has(1):
                values = self.strings.get(b.str(1))
            elif b.msg(2).has(3) and b.msg(2).msg(3).has(1) and b.msg(2).msg(3).msg(1).has(4):
                values = [v.decode("utf-8") for v in b.msg(2).msg(3).msg(1).msg(4).all(1)]
            else:
                values = None
            if values and len(values) == 1:
                return values[0]
        unestablished("%s's %s other than a constant string" % (op.str(1), name))

    def resample_inputs(self, op):
        """From iOS 16 coremlc gives resample the coordinates' format and
        type, which coremltools leaves out; it refuses modes it doesn't know."""
        for name, known in RESAMPLE_MODES.items():
            if self.string_input(op, name) not in known:
                unestablished("resample's %s %r" % (name, self.string_input(op, name)))
        given = {name for name, _ in op.map(2)}
        if given & {"coordinates_format", "coordinates_type"}:
            unestablished("resample with its coordinates' format or type")
        if self.opset == "CoreML5":
            return []
        return [("coordinates_format", self.scalar_literal("string", '"xy"')),
                ("coordinates_type", self.scalar_literal("string", '"absolute"'))]

    def operation(self, op, indent):
        pad = " " * indent
        outputs = op.msgs(3)
        if not outputs and (op.str(1) != "write_state" or op.has(4)):
            unestablished("an operation without outputs")
        inputs = []
        for name, argument in op.map(2):
            bindings = argument.msgs(1)
            if not bindings or not all(b.has(1) or b.has(2) for b in bindings):
                unestablished("an operation input bound to nothing")
            # An input bound to a value is written as the value.
            names = [self.ref(b.str(1)) if b.has(1) else self.value(b.msg(2)) for b in bindings]
            # A variadic input lists its arguments in parentheses.
            inputs.append((name, names[0] if len(names) == 1 else "(%s)" % ", ".join(names)))
        if op.str(1) == "const" and len(outputs) == 1:
            val = dict(op.map(5)).get("val")
            if val is not None and val.has(3) and val.msg(3).has(1) and val.msg(3).msg(1).has(4):
                self.strings[outputs[0].str(1)] = [v.decode("utf-8") for v in val.msg(3).msg(1).msg(4).all(1)]
        if op.str(1) == "resample":
            inputs += self.resample_inputs(op)
        inputs = ["%s = %s" % kv for kv in sorted(inputs, key=lambda kv: kv[0])]
        attributes = ["%s = %s" % (name, self.value(v)) for name, v in sorted(op.map(5), key=lambda kv: kv[0])]
        head = "%s%s = %s(%s)[%s]" % (pad, "%s", op.str(1), ", ".join(inputs), ", ".join(attributes))

        def declared():
            # Several outputs are listed, each with its type.
            return ", ".join("%s %s" % (self.value_type(o.msg(2)), self.declare(o.str(1))) for o in outputs)
        blocks = op.msgs(4)
        if not outputs:
            # write_state declares nothing.
            self.lines.append(head.replace("%s = ", "", 1) + ";")
            return
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


def print_program(program, spec_version, extra_build_info, function_attributes):
    """model.mil for an MLProgram message; `function_attributes` gives each
    function's attributes coremlc derives from the model description."""
    version = program.int(1)
    if version != 1:
        unestablished("program version %d" % version)
    scalar_style = spec_version >= 9
    p = Printer(scalar_style)
    p.lines.append("program(%s)" % ("1.3" if scalar_style else "1.0"))
    attributes = program.map(4)
    if [k for k, _ in attributes] not in (["buildInfo"], []):
        unestablished("program attributes other than buildInfo")
    # A program without a buildInfo gets coremlc's own.
    info = attributes[0][1] if attributes else Message(b"")
    pairs = []
    for kv in ((info.msg(3) or Message(b"")).msg(4) or Message(b"")).msgs(1):
        k = kv.msg(1).msg(3).msg(1).msg(4).all(1)[0].decode()
        v = kv.msg(2).msg(3).msg(1).msg(4).all(1)[0].decode()
        pairs.append((k, v))
    pairs = sorted(dict(pairs + extra_build_info).items())
    kv_type = "string" if scalar_style else "tensor<string, []>"
    p.lines.append("[buildInfo = dict<%s, %s>({%s})]" % (kv_type, kv_type, ", ".join('{"%s", "%s"}' % (p.escape(k), p.escape(v)) for k, v in pairs)))
    p.lines.append("{")
    functions = sorted(program.map(2), key=lambda kv: kv[0])
    for name, function in functions:
        # Names are numbered within their function.
        names = [i.str(1) for i in function.msgs(1)]
        for _, b in function.map(3):
            _declared(b, names)
        p.duplicates = {n for n in names if names.count(n) > 1}
        p.seen = {}
        opset = function.str(2)
        p.opset = opset
        p.strings = {}
        if function.has(4):
            unestablished("a function with attributes")
        inputs = ", ".join("%s %s" % (p.value_type(i.msg(2)), i.str(1)) for i in sorted(function.msgs(1), key=lambda i: i.str(1)))
        attributes = function_attributes.get(name)
        p.lines.append("    func %s<%s>(%s)%s {" % (name, OPSETS.get(opset) or unestablished("opset " + opset), inputs,
                                                   " [%s]" % ", ".join(attributes) if attributes else ""))
        blocks = dict((k, v) for k, v in function.map(3))
        if list(blocks) != [opset]:
            unestablished("block specializations other than the opset's")
        outputs = p.block(blocks[opset], 12)
        p.lines.append("        } -> (%s);" % outputs)
    p.lines.append("}")
    return "\n".join(p.lines)



# Written into model.mil's buildInfo, as Xcode 27.0's coremlc writes it.
COREMLC_BUILD_INFO = [("coremlc-component-MIL", "3600.16.1"), ("coremlc-version", "3600.25.1")]


def enumerated_image_inputs(model):
    """Xcode refuses an image with enumerated sizes whose program input
    fixes its height or width (a different shape from the image's)."""
    program = dict(model.msg(502).map(2))
    for name, inputs, _, _ in functions(model):
        for f in inputs:
            t = f.msg(3)
            if not (t.has(4) and t.msg(4).has(21)):
                continue
            params = [i for i in program[name].msgs(1) if i.str(1) == f.str(1)] if name in program else []
            dims = params[0].msg(2).msg(1).msgs(3) if params and params[0].msg(2).has(1) else None
            if dims is None or len(dims) != 4 or any(d.has(1) for d in dims[2:]):
                unestablished("an image with enumerated sizes whose program input fixes its height or width")


def model_mil(model, extra_build_info=COREMLC_BUILD_INFO):
    enumerated_image_inputs(model)
    attributes = {name: function_attributes(inputs, states, model.int(1)) for name, inputs, _, states in functions(model)}
    return print_program(model.msg(502), model.int(1), extra_build_info, attributes)


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


# NSJSONSerialization's escapes for control characters; it writes the others
# as \u00XX.
JSON_ESCAPES = {"\b": "\\b", "\f": "\\f", "\n": "\\n", "\r": "\\r", "\t": "\\t"}


def _json_string(text):
    out = []
    for ch in text:
        if ch == '"':
            out.append('\\"')
        elif ch == "\\":
            out.append("\\\\")
        elif ch == "/":
            out.append("\\/")
        elif ch in JSON_ESCAPES:
            out.append(JSON_ESCAPES[ch])
        elif ord(ch) < 0x20:
            out.append("\\u%04x" % ord(ch))
        else:
            out.append(ch)
    return '"' + "".join(out) + '"'


# --- Model description ----------------------------------------------------------

ARRAY_TYPES = {65568: "Float32", 65600: "Double", 131104: "Int32", 65552: "Float16"}
COLOR_SPACES = {10: ("Grayscale", "0"), 20: ("RGB", "1"), 30: ("BGR", "1"), 40: ("Grayscale16Half", "0")}
# Images' formatted types name their color space so.
IMAGE_KINDS = {10: "Grayscale", 20: "Color", 30: "Color", 40: "Grayscale16Half"}


def _ints(values):
    """A repeated integer field's values, packed or not."""
    out = []
    for r in values:
        if isinstance(r, bytes):
            i = 0
            while i < len(r):
                v, i = _varint(r, i)
                out.append(v)
        else:
            out.append(r)
    return out


def _signed(v):
    return v - (1 << 64) if v >= 1 << 63 else v


def functions(model):
    """[(name, inputs, outputs, states)] of an ML Program's functions, as its
    description gives them (a single-function model's is "main")."""
    description = model.msg(2) or Message(b"")
    if not description.has(20):
        return [("main", description.msgs(1), description.msgs(10), description.msgs(13))]
    out = []
    for f in description.msgs(20):
        if f.has(4) or f.has(5):
            unestablished("a classifier function")
        out.append((f.str(1), f.msgs(2), f.msgs(3), f.msgs(6)))
    return out


def default_value(f):
    """An optional multi-array input's default, as coremlc writes it (%.9g of a
    float; 0 when the model gives none)."""
    t = f.msg(3)
    if not t.has(5):
        unestablished("an optional feature other than a multi-array")
    a = t.msg(5)
    dtype = ARRAY_TYPES.get(a.int(2))
    # Xcode refuses a default of another type than the array's.
    if a.has(61) or (a.has(51) and dtype not in ("Float32", "Float16")) or (a.has(41) and dtype != "Int32"):
        unestablished("a default value of another type than its multi-array")
    value = struct.unpack("<f", a.bytes(51))[0] if a.has(51) else float(struct.unpack("<i", struct.pack("<I", a.int(41) & 0xFFFFFFFF))[0]) if a.has(41) else 0.0
    if value != value or value in (float("inf"), float("-inf")):
        unestablished("a default value that isn't finite")
    return "%.9g" % value


def _mil_name(text):
    if not text or any(ch in '"\\' or ord(ch) < 0x20 or ord(ch) >= 0x7F for ch in text):
        unestablished("a feature named %r in model.mil" % text)
    return '"%s"' % text


def flexibility(f):
    """(default shape, [(low, high)] or None, [shapes] or None) of an input as
    model.mil gives it: an image as [1, channels, height, width]."""
    t = f.msg(3)
    if t.has(5):
        a = t.msg(5)
        shape = _ints(a.all(1))
        ranges = [(r.int(1), _signed(r.int(2))) for r in a.msg(31).msgs(1)] if a.has(31) else None
        shapes = [_ints(e.all(1)) for e in a.msg(21).msgs(1)] if a.has(21) else None
    elif t.has(4):
        im = t.msg(4)
        c = 1 if im.int(3) in (10, 40) else 3
        shape = [1, c, im.int(2), im.int(1)]
        ranges = shapes = None
        if im.has(31):
            w, h = im.msg(31).msg(1) or Message(b""), im.msg(31).msg(2) or Message(b"")
            ranges = [(1, 1), (c, c), (h.int(1), _signed(h.int(2))), (w.int(1), _signed(w.int(2)))]
        if im.has(21):
            shapes = [[1, c, e.int(2), e.int(1)] for e in im.msg(21).msgs(1)]
    else:
        return None, None, None
    if ranges is not None and (len(ranges) != len(shape) or any(hi < lo for lo, hi in ranges)):
        unestablished("an unbounded or malformed shape range")
    if shapes is not None and (not shapes or any(len(x) != len(shape) for x in shapes)):
        unestablished("malformed enumerated shapes")
    return shape, ranges, shapes


def input_defaults(inputs):
    """[(name, default)] of a function's optional inputs."""
    defaults = sorted((f.str(1), default_value(f)) for f in inputs if f.msg(3) and f.msg(3).int(1000))
    if inputs and len(defaults) == len(inputs):
        # Xcode refuses a model whose inputs are all optional.
        unestablished("a function whose inputs are all optional")
    return defaults


def function_attributes(inputs, states, spec_version):
    """The attributes coremlc gives a function in model.mil: its optional
    inputs' default values and its inputs' flexible shapes."""
    scalar_style = spec_version >= 9
    S = "string" if scalar_style else "tensor<string, []>"
    out = []
    defaults = input_defaults(inputs)
    if defaults:
        out.append("CoreML_InputDefaultValues = dict<%s, %s>({%s})" % (S, "fp32" if scalar_style else "tensor<fp32, []>",
                                                                       ", ".join("{%s, %s}" % (_mil_name(n), v) for n, v in defaults)))
    flex = {f.str(1): flexibility(f) for f in inputs}
    ranged = sorted(n for n, (_, r, _) in flex.items() if r is not None)
    enumerated = sorted(n for n, (_, _, e) in flex.items() if e is not None)
    if not ranged and not enumerated:
        return out

    def shape(dims):
        return "[%s]" % ", ".join(map(str, dims))
    default_shapes = "(\"DefaultShapes\", {%s})" % ", ".join("{%s, %s}" % (_mil_name(n), shape(flex[n][0])) for n in ranged + enumerated)
    if ranged and enumerated:
        # Xcode refuses a model with both.
        unestablished("a mixture of enumerated and range shape flexibility")
    if ranged:
        out.append("FlexibleShapeInformation = tuple<tuple<%s, dict<%s, tensor<int32, [?]>>>, tuple<%s, dict<%s, list<tensor<int32, [2]>, ?>>>>((%s, (\"RangeDims\", {%s})))" % (
            S, S, S, S, default_shapes, ", ".join("{%s, [%s]}" % (_mil_name(n), ", ".join("[%d, %d]" % r for r in flex[n][1])) for n in ranged)))
        return out
    # Several inputs' enumerated shapes (specification 9) go together by
    # their position.
    counts = {len(flex[n][2]) for n in enumerated}
    if len(enumerated) > 1 and (not scalar_style or len(counts) != 1):
        unestablished("several inputs with enumerated shapes before specification 9 or of different counts")
    for f in inputs:
        if spec_version == 7 and f.msg(3).has(4) and f.msg(3).msg(4).int(3) == 40 and f.str(1) in enumerated:
            # At specification 7 Xcode gives a Grayscale16Half image's
            # enumerated sizes three channels (its default keeps one).
            n = f.str(1)
            flex[n] = (flex[n][0], None, [[1, 3] + e[2:] for e in flex[n][2]])
    # Each enumerated shape is named for every input's and state's shape,
    # padded to rank 5 (name_1_1_1_2_3_name2_...), from specification 9 by
    # that name's SHA-256, eight hex digits.
    for f in states:
        flex[f.str(1)] = (_ints(f.msg(3).msg(8).msg(1).all(1)), None, None)
    names = sorted(flex)
    if any(flex[n][0] is None or len(flex[n][0]) > 5 for n in names):
        unestablished("enumerated shapes beside an input of another kind or rank")
    entries = {}
    for k in range(counts.pop()):
        shapes = [(n, flex[n][2][k] if n in enumerated else flex[n][0]) for n in names]
        key = "".join("%s_%s_" % (n, "_".join(map(str, [1] * (5 - len(d)) + d))) for n, d in shapes)
        if scalar_style:
            key = hashlib.sha256(key.encode()).hexdigest()[:8]
        if key in entries:
            unestablished("repeated enumerated shapes")
        entries[key] = "{%s}" % ", ".join("{%s, %s}" % (_mil_name(n), shape(d)) for n, d in shapes)
    out.append("FlexibleShapeInformation = tuple<tuple<%s, dict<%s, tensor<int32, [?]>>>, tuple<%s, dict<%s, dict<%s, tensor<int32, [?]>>>>>((%s, (\"EnumeratedShapes\", {%s})))" % (
        S, S, S, S, S, default_shapes, ", ".join("{%s, %s}" % (_mil_name(k), v) for k, v in sorted(entries.items()))))
    return out

# Keys in the order coremlc inserts them (CoreFoundation's enumeration
# order depends on it where keys collide), fitted to Xcode 27.0's output.
TOP_KEYS = ["author", "inputSchema", "modelParameters", "shortDescription", "version", "method", "modelType",
            "license", "metadataOutputVersion", "outputSchema", "defaultFunctionName", "stateSchema", "isUpdatable",
            "classLabels", "specificationVersion", "availability", "generatedClassName", "userDefinedMetadata",
            "storagePrecision", "functions", "mlProgramOperationTypeHistogram", "computePrecision"]
SCHEMA_KEYS = ["sizeFlexibility", "enumeratedSizes", "isColor", "colorspace", "hasSizeFlexibility", "shape", "keyType", "type",
            "sizeRange", "dataType", "width", "height", "name", "isOptional", "hasShapeFlexibility", "shortDescription",
            "formattedType", "valueType"]
# A flexible multi-array's keys go in in another order.
FLEXIBLE_ARRAY_KEYS = ["shapeRange", "enumeratedShapes", "shapeFlexibility", "shape", "type", "name", "dataType",
                       "hasShapeFlexibility", "isOptional", "shortDescription", "formattedType"]
# A multi-function model's functions.
FUNCTION_KEYS = ["name", "inputSchema", "outputSchema", "stateSchema", "storagePrecision", "mlProgramOperationTypeHistogram",
                 "computePrecision"]

# Deployment availability by specification version.
AVAILABILITY = {
    1: {"macOS": "10.13", "tvOS": "11.0", "visionOS": "1.0", "watchOS": "4.0", "iOS": "11.0", "macCatalyst": "11.0"},
    3: {"macOS": "10.14", "tvOS": "12.0", "visionOS": "1.0", "watchOS": "5.0", "iOS": "12.0", "macCatalyst": "12.0"},
    4: {"macOS": "10.15", "tvOS": "13.0", "visionOS": "1.0", "watchOS": "6.0", "iOS": "13.0", "macCatalyst": "13.0"},
    5: {"macOS": "11.0", "tvOS": "14.0", "visionOS": "1.0", "watchOS": "7.0", "iOS": "14.0", "macCatalyst": "14.0"},
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


def _dims(dims):
    return " × ".join(map(str, dims))


def feature(f):
    t = f.msg(3) or unestablished("a feature without a type")
    optional = "1" if t.int(1000) else "0"
    if t.int(1000) and not t.has(5):
        unestablished("an optional feature other than a multi-array")
    pairs = [("isOptional", optional), ("name", f.str(1)), ("shortDescription", f.str(2))]
    if t.has(5):
        a = t.msg(5)
        dtype = ARRAY_TYPES.get(a.int(2)) or unestablished("array data type %d" % a.int(2))
        shape, ranges, shapes = flexibility(f)
        pairs += [("type", "MultiArray"), ("dataType", dtype), ("shape", "[" + ", ".join(map(str, shape)) + "]"),
                  ("hasShapeFlexibility", "1" if ranges or shapes else "0"),
                  # An optional feature's type ends in "?".
                  ("formattedType", "MultiArray (%s)%s" % (" ".join([dtype] + ([_dims(shape)] if shape else [])), "?" if t.int(1000) else ""))]
        if ranges and shapes:
            unestablished("a multi-array with both shape ranges and enumerated shapes")
        if ranges:
            pairs += [("shapeRange", "[%s]" % ", ".join("[%d, %d]" % r for r in ranges)),
                      ("shapeFlexibility", " × ".join(str(lo) if lo == hi else "%d...%d" % (lo, hi) for lo, hi in ranges))]
        if shapes:
            pairs += [("enumeratedShapes", "[%s]" % ", ".join("[%s]" % ", ".join(map(str, x)) for x in shapes)),
                      ("shapeFlexibility", " | ".join(_dims(x) for x in shapes))]
        if ranges or shapes:
            return ordered(pairs, FLEXIBLE_ARRAY_KEYS)
    elif t.has(4):
        im = t.msg(4)
        space, color = COLOR_SPACES.get(im.int(3)) or unestablished("color space %d" % im.int(3))
        w, h = im.int(1), im.int(2)
        pairs += [("type", "Image"), ("width", str(w)), ("height", str(h)), ("colorspace", space), ("isColor", color),
                  ("hasSizeFlexibility", "1" if im.has(21) or im.has(31) else "0"),
                  ("formattedType", "Image (%s %d × %d)" % (IMAGE_KINDS[im.int(3)], w, h))]
        if im.has(21) and im.has(31):
            unestablished("an image with both size ranges and enumerated sizes")
        if im.has(31):
            ranges = [(r.int(1), _signed(r.int(2))) for r in (im.msg(31).msg(1) or Message(b""), im.msg(31).msg(2) or Message(b""))]
            if any(hi < -1 or -1 < hi < lo for lo, hi in ranges):
                unestablished("an image size range out of order")
            # Width first; an unbounded range is written "1...".
            pairs += [("sizeRange", "[%s]" % ", ".join("[%d, %d]" % r for r in ranges)),
                      ("sizeFlexibility", " × ".join(str(lo) if lo == hi else "%d...%s" % (lo, "" if hi < 0 else hi) for lo, hi in ranges))]
        if im.has(21):
            sizes = [(e.int(1), e.int(2)) for e in im.msg(21).msgs(1)]
            pairs += [("enumeratedSizes", "[%s]" % ", ".join("[%d, %d]" % x for x in sizes)),
                      ("sizeFlexibility", " | ".join("%d × %d" % x for x in sizes))]
    elif t.has(8):
        a = t.msg(8).msg(1) or unestablished("a state that isn't a multi-array")
        if any(a.has(n) for n in (21, 31, 41, 51, 61)):
            unestablished("a state with flexible shapes or a default value")
        dtype = ARRAY_TYPES.get(a.int(2)) or unestablished("array data type %d" % a.int(2))
        shape = _ints(a.all(1))
        pairs += [
            ("type", "State"), ("dataType", dtype), ("shape", "[" + ", ".join(map(str, shape)) + "]"),
            ("formattedType", "State (%s %s)" % (dtype, _dims(shape)))]
    elif t.has(1):
        pairs += [("type", "Int64"), ("formattedType", "Int64")]
    elif t.has(2):
        pairs += [("type", "Double"), ("formattedType", "Double")]
    elif t.has(3):
        pairs += [("type", "String"), ("formattedType", "String")]
    elif t.has(6):
        d = t.msg(6)
        key = "Int64" if d.has(1) else "String"
        pairs += [("type", "Dictionary"), ("keyType", key), ("formattedType", "Dictionary (%s → Double)" % key)]
    elif t.has(7):
        q = t.msg(7)
        value = "Int64" if q.has(1) else "String" if q.has(3) else unestablished("a sequence of another type")
        size = q.msg(101) or unestablished("a sequence without a size range")
        low, high = size.int(1), size.int(2)
        if high >= 1 << 63:
            unestablished("an unbounded sequence")
        pairs += [("type", "Sequence"), ("valueType", value), ("sizeRange", "[%d, %d]" % (low, high)),
                  ("formattedType", "Sequence (%s %d...%d)" % (value, low, high))]
    else:
        unestablished("a feature type other than a multi-array, image, state, Int64, String or dictionary")
    return ordered(pairs, SCHEMA_KEYS)


OP_NAMES = None


def histogram_name(op, spec_version):
    global OP_NAMES
    if OP_NAMES is None:
        with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "coremlc_ops.json")) as f:
            OP_NAMES = json.load(f)
    entry = OP_NAMES.get(op) or unestablished("operation %s's histogram name" % op)
    target = {6: 15, 7: 16, 8: 17, 9: 18}.get(spec_version) or unestablished("specification version %d" % spec_version)
    # An entry that lists its targets was compiled for those alone.
    if target not in entry.get("targets", [target]):
        unestablished("operation %s's histogram name for iOS %d" % (op, target))
    versions = [v for v in entry["versions"] if v <= target]
    if versions:
        return "Ios%d.%s" % (max(versions), entry["camel"])
    return entry["base"] or unestablished("operation %s's histogram name" % op)


def _walk_ops(block, out):
    for op in block.msgs(3):
        out.append(op)
        for b in op.msgs(4):
            _walk_ops(b, out)


PRECISION_NAMES = {"fp16": "Float16", "fp32": "Float32", "fp64": "Float64", "int32": "Int32", "int64": "Int64", "uint16": "UInt16",
                   "int8": "Int8", "uint8": "UInt8", "uint32": "UInt32"}

# Weights narrower than a byte, in storagePrecision; of them only UInt1 (a
# sparse mask an operation produces) was seen in computePrecision.
SUB_BYTE_PRECISION_NAMES = {"uint1": "UInt1", "uint2": "UInt2", "uint3": "UInt3", "uint4": "UInt4", "uint6": "UInt6",
                            "int4": "Int4"}

# iOS 16's compression operations, which hold their weights as attributes:
# what they add to storagePrecision is their own description (below), not
# the types of those weights.
ATTRIBUTE_WEIGHTS = ("constexpr_affine_dequantize", "constexpr_cast", "constexpr_lut_to_dense", "constexpr_sparse_to_dense")


def program_facts(program, spec_version, only=None):
    """(histogram pairs in first-use order, compute precision, class labels)
    of the program, or of its function `only`."""
    ops = []
    declared = []
    # Compute precision counts what operations produce (constants
    # included), not the function's inputs.
    for name, function in sorted(program.map(2), key=lambda kv: kv[0]):
        if only is not None and name != only:
            continue
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
    named = {o.str(1): o.msg(2) for op in ops for o in op.msgs(3)}

    def dtype_of(t):
        if t is None or not t.has(1):
            unestablished("compressed weights of a type other than a tensor")
        dtype = DATA_TYPES.get(t.msg(1).int(1))
        return PRECISION_NAMES.get(dtype) or SUB_BYTE_PRECISION_NAMES.get(dtype) or unestablished("weights stored as %s" % dtype)

    def store(v):
        stored.add(dtype_of(v.msg(2)))

    def operand(op, name):
        """The type of `op`'s input or attribute `name`."""
        attributes = dict(op.map(5))
        if name in attributes:
            return attributes[name].msg(2)
        argument = dict(op.map(2)).get(name)
        bindings = argument.msgs(1) if argument else []
        if len(bindings) != 1:
            unestablished("%s without one %s" % (op.str(1), name))
        b = bindings[0]
        return b.msg(2).msg(2) if b.has(2) else named.get(b.str(1)) or unestablished("%s's %s from a function input" % (op.str(1), name))
    for op in ops:
        kind = op.str(1)
        if kind != "const":
            # Other operations' weights count when bound to an input from
            # the weights file.
            for _, argument in op.map(2):
                for b in argument.msgs(1):
                    if b.has(2) and b.msg(2).has(5):
                        store(b.msg(2))
            if any(v.has(5) for _, v in op.map(5)) and kind not in ATTRIBUTE_WEIGHTS:
                unestablished("weights held as an attribute of %s" % kind)
            # Compression operations add their own description, from
            # their weights' types and shapes however they're held.
            if kind == "constexpr_lut_to_dense":
                # The bits that would index every entry of every table.
                t = operand(op, "lut")
                entries = 1
                for d in t.msg(1).msgs(3):
                    entries *= d.msg(1).int(1) if d.has(1) else unestablished("a lookup table of unknown shape")
                stored.add("Palettized (%d bits)" % (entries - 1).bit_length())
            elif kind == "constexpr_sparse_to_dense":
                stored.add("Sparse")
            elif kind == "constexpr_affine_dequantize":
                stored.add(dtype_of(operand(op, "quantized_data")))
            elif kind == "constexpr_blockwise_shift_scale":
                stored.add(dtype_of(operand(op, "data")))
            continue
        out = op.msgs(3)[0]
        for _, v in op.map(5):
            if v.has(5) and out.str(1) in read:
                store(v)
    types = set()

    def collect(t):
        if t is None:
            return
        if t.has(1):
            dtype = DATA_TYPES.get(t.msg(1).int(1))
            if dtype not in ("bool", "string"):
                types.add(PRECISION_NAMES.get(dtype) or (dtype == "uint1" and SUB_BYTE_PRECISION_NAMES[dtype]) or unestablished("compute precision with %s" % dtype))
        elif t.has(4):
            collect(t.msg(4).msg(1))
            collect(t.msg(4).msg(2))
        elif t.has(2):
            collect(t.msg(2).msg(1))
        else:
            unestablished("compute precision with a tuple or state type")
    for t in declared:
        collect(t)
    names = sorted(types)
    precision = names[0] if len(names) == 1 else "Mixed (%s)" % ", ".join(names)
    storage = sorted(stored)
    storage = (storage[0] if len(storage) == 1 else "Mixed (%s)" % ", ".join(storage)) if storage else None
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
    if description.has(50):
        unestablished("a model with training inputs")
    if not model.has(502) and (description.has(13) or description.has(20)):
        unestablished("states or functions outside an ML Program")
    meta = description.msg(100) or Message(b"")
    program = model.msg(502)
    inputs, outputs, states = description.msgs(1), description.msgs(10), description.msgs(13)
    if program is not None:
        fns = functions(model)
        default = default_function(model)
        histogram, precision, classifier, storage = program_facts(program, spec_version, default if description.has(20) else None)
        _, inputs, outputs, states = next(f for f in fns if f[0] == default)
        model_type = "MLModelType_mlProgram"
    elif pipeline_parts(model) is not None:
        kind, parts = pipeline_parts(model)
        model_type = CFDict([("name", "MLModelType_" + ("imageClassifier" if image_classifier(kind, parts) else PIPELINE_TYPES[kind])),
                             ("structure", [CFDict([("name", "MLModelType_" + PART_TYPES[pk])]) for pk, _ in parts])])
        labels = None
        for pk, part in parts:
            if pk in (400, 402):
                labels = model_class_labels(Message(part).msg(pk))
    elif tree_kind(model) is not None:
        kind = tree_kind(model)
        model_type = "MLModelType_" + TREE_TYPES[kind]
        labels = model_class_labels(model.msg(kind)) if kind == 402 else None
    else:
        kind = SPEC_ARCHIVED.get(next((n for n, _, _ in model.fields if n in SPEC_ARCHIVED), None))
        if kind is None:
            unestablished("this model type's metadata")
        model_type = kind
        labels = [v.decode() for v in model.msg(SPEC_TYPE_FIELD[kind]).msg(200).all(1)] if model.msg(SPEC_TYPE_FIELD[kind]).has(200) else unestablished("a text model without labels")
    if model.int(10):
        unestablished("an updatable model")
    vision = vision_model(model)
    pairs = [("metadataOutputVersion", "3.0")]
    for key, number in (("shortDescription", 1), ("author", 3), ("license", 4), ("version", 2)):
        if meta.has(number) and meta.str(number):
            pairs.append((key, meta.str(number)))
    pairs += [
        ("outputSchema", [feature(f) for f in outputs]),
        ("modelParameters", []),
        ("specificationVersion", spec_version),
    ]
    if program is not None:
        pairs += [("mlProgramOperationTypeHistogram", CFDict(histogram)), ("computePrecision", precision)]
    pairs += [
        ("isUpdatable", "0"),
        ("stateSchema", [feature(f) for f in states]),
        ("availability", CFDict([(k, "unavailable" if k == "watchOS" and vision else (AVAILABILITY.get(spec_version) or unestablished("availability for specification %d" % spec_version))[k]) for k in AVAILABILITY_KEYS])),
        ("modelType", model_type if isinstance(model_type, CFDict) else CFDict([("name", model_type)])),
        ("inputSchema", [feature(f) for f in inputs]),
        ("generatedClassName", generated_class_name(name)),
        ("method", "predict"),
    ]
    # Xcode enumerates this map in a randomly seeded order, so its output
    # varies from run to run when it has more than one entry; this keeps the
    # model's own order.
    pairs.append(("userDefinedMetadata", CFDict(meta.map_str(100))))
    if program is None:
        if labels is not None:
            pairs.append(("classLabels", labels))
    else:
        if classifier:
            pairs.append(("classLabels", class_labels(program)))
        if storage:
            pairs.append(("storagePrecision", storage))
        if description.has(20):
            entries = []
            for fname, finputs, foutputs, fstates in fns:
                fhistogram, fprecision, _, fstorage = program_facts(program, spec_version, fname)
                fpairs = [("name", fname), ("inputSchema", [feature(f) for f in finputs]), ("outputSchema", [feature(f) for f in foutputs]),
                          ("stateSchema", [feature(f) for f in fstates]), ("mlProgramOperationTypeHistogram", CFDict(fhistogram)),
                          ("computePrecision", fprecision)] + ([("storagePrecision", fstorage)] if fstorage else [])
                entries.append(ordered(fpairs, FUNCTION_KEYS))
            pairs += [("functions", entries), ("defaultFunctionName", default)]
    return "[\n  " + ns_json(ordered(pairs, TOP_KEYS), 1) + "\n]"


def default_function(model):
    """The function a multi-function model's predictions use; "main" otherwise."""
    description = model.msg(2) or Message(b"")
    if not description.has(20):
        if description.has(21):
            unestablished("a default function without functions")
        return "main"
    names = [f.str(1) for f in description.msgs(20)]
    program = [n for n, _ in model.msg(502).map(2)]
    if sorted(names) != sorted(program) or len(set(names)) != len(names) or description.str(21) not in names:
        unestablished("functions that don't match the program's")
    if description.has(1) or description.has(10) or description.has(13) or description.has(11) or description.has(12):
        unestablished("a multi-function model with its own features")
    return description.str(21)


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
            # A description without metadata gets an empty one.
            return value if Message(value).has(100) else value + b"\xa2\x06\x00"
    return b""


def coremldata(model):
    spec_version = model.int(1)
    header_version = {6: 5, 7: 5, 8: 8, 9: 9}.get(spec_version) or unestablished("specification version %d" % spec_version)
    description = canonical_description(model)
    head = (_u32(502) + _u32(header_version) + b"\0" * 20 + _u64(7) + b"generic" + _u32(spec_version) + b"\0" * 28
            + _u64(len(description)) + description)
    classifier = classifier_facts(model.msg(502))
    if classifier is None:
        # One per function.
        return head + (_u32(502) + b"\0" * 12) * len(functions(model))
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


# Model types by their field in the Model message, for messages.
MODEL_TYPES = {
    200: "pipeline classifier", 201: "pipeline regressor", 202: "pipeline", 300: "GLM regressor",
    301: "support vector regressor", 302: "tree ensemble regressor", 303: "neural network regressor",
    304: "Bayesian probit regressor", 400: "GLM classifier", 401: "support vector classifier",
    402: "tree ensemble classifier", 403: "neural network classifier", 404: "k-nearest neighbors classifier",
    500: "neural network", 501: "item similarity recommender", 502: "ML Program", 555: "custom model",
    556: "linked model", 560: "class confidence thresholding", 2000: "text classifier", 2001: "word tagger",
    2002: "vision feature print", 2003: "sound analysis preprocessing", 2004: "gazetteer", 2005: "word embedding",
    2006: "audio feature print",
}


# Models coremlc compiles to their own specification behind a header: their
# Model field and the name metadata.json gives their type.
SPEC_ARCHIVED = {2000: "MLModelType_textClassifier", 2001: "MLModelType_wordTagger"}
SPEC_TYPE_FIELD = {v: k for k, v in SPEC_ARCHIVED.items()}


def compile_spec_model(model_bytes, name, outdir):
    """A text classifier or word tagger: coremldata.bin is a header and the
    model's own bytes."""
    model = Message(model_bytes)
    kind = next(n for n, _, _ in model.fields if n in SPEC_ARCHIVED)
    spec_version = model.int(1)
    if spec_version != 3:
        unestablished("specification version %d of a %s" % (spec_version, MODEL_TYPES[kind]))
    header = _u32(kind) + _u32(1) + b"\0" * 20 + _u64(4) + b"spec" + _u32(spec_version) + b"\0" * 28
    def s(text):
        raw = text.encode()
        return _u64(len(raw)) + raw
    files = {
        "coremldata.bin": header + model_bytes,
        "metadata.json": metadata_json(model, name).encode("utf-8"),
        "analytics/coremldata.bin": s("SpecificationDetails") + _u64(2) + s("modelHash") + s(base64.b64encode(hashlib.sha256(model_bytes).digest()).decode()) + s("modelName") + s(name),
    }
    bundle = os.path.join(outdir, name + ".mlmodelc")
    for rel, data in files.items():
        os.makedirs(os.path.dirname(os.path.join(bundle, rel)), exist_ok=True)
        with open(os.path.join(bundle, rel), "wb") as f:
            f.write(data)


PIPELINE_TYPES = {200: "pipelineClassifier", 201: "pipelineRegressor"}
# Pipeline parts compiled to a header and their own specification, by Model
# field, and the names metadata.json gives them.
PART_TYPES = {300: "glmRegressor", 400: "glmClassifier", 600: "oneHotEncoder", 602: "featureVectorizer", 2002: "visionFeaturePrint",
              302: "treeEnsembleRegressor", 402: "treeEnsembleClassifier"}
# Tree ensembles, compiled to their own packed form.
TREE_TYPES = {302: "treeEnsembleRegressor", 402: "treeEnsembleClassifier"}


def tree_kind(model):
    """A standalone tree ensemble's Model field, or None."""
    return next((n for n, _, _ in model.fields if n in TREE_TYPES), None)


def model_class_labels(classifier):
    """A linear or tree classifier's labels: strings (field 100) or 64-bit
    integers (field 101)."""
    if classifier.has(100):
        return [v.decode() for v in classifier.msg(100).all(1)]
    if classifier.has(101):
        raw, labels, i = classifier.msg(101).bytes(1), [], 0
        while i < len(raw):
            v, i = _varint(raw, i)
            labels.append(v - (1 << 64) if v >= 1 << 63 else v)
        return labels
    return None


def pipeline_parts(model):
    """(the pipeline's Model field, [(part's field, part's bytes)]), or None."""
    kind = next((n for n, _, _ in model.fields if n in PIPELINE_TYPES), None)
    if kind is None:
        return None
    parts = []
    for part in model.msg(kind).msg(1).all(1):
        fields = [n for n, _, _ in Message(part).fields if n not in (1, 2, 10)]
        if len(fields) != 1 or fields[0] not in PART_TYPES:
            unestablished("a pipeline with a %s" % (MODEL_TYPES.get(fields[0], "type %d" % fields[0]) if fields else "typeless model"))
        parts.append((fields[0], part))
    if any(pk == 2002 for pk, _ in parts) and not image_classifier(kind, parts):
        unestablished("a pipeline with a vision feature print that isn't Create ML's image classifier")
    return kind, parts


def vision_model(model):
    """Whether the model uses Vision's feature prints, which watchOS lacks."""
    parts = pipeline_parts(model)
    return parts is not None and image_classifier(*parts)


def image_classifier(kind, parts):
    """Create ML's image classifier: a scene print, then a linear classifier."""
    return (kind == 200 and [pk for pk, _ in parts] == [2002, 400]
            and Message(parts[0][1]).msg(2002).has(20))


# --- Tree ensembles ---------------------------------------------------------------
#
# A tree ensemble (Model field 302 or 402) compiles to `coremldata.bin` and
# `_B0000.DAT`, the trees in a packed binary form. The layout was fitted
# black-box to Xcode's output for constructed ensembles; the bracketed probe
# names below are those of docs/clean-room/coremlc-trees/README.md.


def _wire_fields(b):
    """Yield (field, wiretype, value, raw_bytes_of_whole_field)."""
    o = 0
    while o < len(b):
        start = o
        key, o = _varint(b, o)
        f, wt = key >> 3, key & 7
        if wt == 0:
            v, o = _varint(b, o)
        elif wt == 1:
            v = b[o:o + 8]; o += 8
        elif wt == 2:
            n, o = _varint(b, o); v = b[o:o + n]; o += n
        elif wt == 5:
            v = b[o:o + 4]; o += 4
        else:
            raise ValueError('unsupported wire type %d' % wt)
        if o > len(b):
            raise ValueError('truncated field')
        yield f, wt, v, b[start:o]


def _enc_varint(v):
    out = bytearray()
    while True:
        c = v & 0x7f; v >>= 7
        if v:
            out.append(c | 0x80)
        else:
            out.append(c); return bytes(out)


def _s64(v):
    return v - (1 << 64) if v >= 1 << 63 else v


def _dbl(v):
    return struct.unpack('<d', v)[0]


def _packed_or_single(wt, v, kind):
    """Repeated scalar: either one element or a packed run."""
    if kind == 'double':
        if wt == 1:
            return [_dbl(v)]
        if wt == 2:
            if len(v) % 8:
                raise ValueError('bad packed double')
            return list(struct.unpack('<%dd' % (len(v) // 8), v))
    if kind == 'varint':
        if wt == 0:
            return [v]
        if wt == 2:
            out, o = [], 0
            while o < len(v):
                x, o = _varint(v, o); out.append(x)
            return out
    raise ValueError('unexpected wire type %d for %s' % (wt, kind))


# ------------------------------------------------------------- spec model --

NEGATE = {0: 3, 3: 0, 1: 2, 2: 1, 4: 5, 5: 4}   # LTE<->GT, LT<->GTE, EQ<->NE


def _parse_node(b):
    n = dict(treeId=0, nodeId=0, beh=0, fi=0, fv=0.0, t=0, f=0, miss=False, ev=[])
    for f, wt, v, _ in _wire_fields(b):
        if f == 1: n['treeId'] = v
        elif f == 2: n['nodeId'] = v
        elif f == 3: n['beh'] = v
        elif f == 10: n['fi'] = v
        elif f == 11: n['fv'] = _dbl(v)
        elif f == 12: n['t'] = v
        elif f == 13: n['f'] = v
        elif f == 14: n['miss'] = bool(v)
        elif f == 20:
            idx, val = 0, 0.0
            for g, _wt, w, _ in _wire_fields(v):
                if g == 1: idx = w
                elif g == 2: val = _dbl(w)
            n['ev'].append((idx, val))
        elif f == 30: pass   # relativeHitRate: no effect on output (probe H_hit)
        else:
            raise Unestablished('TreeNode field %d' % f)
    if n['beh'] > 6:
        raise Unestablished('TreeNode behavior %d' % n['beh'])
    return n


def _parse_params(b):
    nodes, npd, base = [], 0, []
    for f, wt, v, _ in _wire_fields(b):
        if f == 1: nodes.append(_parse_node(v))
        elif f == 2: npd = v
        elif f == 3: base += _packed_or_single(wt, v, 'double')
        else: raise Unestablished('TreeEnsembleParameters field %d' % f)
    return nodes, npd, base


def _parse_model(b):
    m = dict(sv=0, desc=None, kind=None, tf=0, strs=None, ints=None, params=None)
    for f, wt, v, _ in _wire_fields(b):
        if f == 1: m['sv'] = v
        elif f == 2: m['desc'] = v
        elif f in (302, 402):
            if m['kind'] is not None:
                raise Unestablished('two model types')
            m['kind'] = f
            for g, wt2, w, _ in _wire_fields(v):
                if g == 1: m['params'] = w
                elif g == 2: m['tf'] = w
                elif f == 402 and g == 100:
                    m['strs'] = [s for h, _a, s, _b in _wire_fields(w) if h == 1]
                elif f == 402 and g == 101:
                    vals = []
                    for h, wt3, s, _b in _wire_fields(w):
                        if h == 1: vals += _packed_or_single(wt3, s, 'varint')
                    m['ints'] = [_s64(x) for x in vals]
                else:
                    raise Unestablished('tree ensemble field %d.%d' % (f, g))
        elif f == 10:   # isUpdatable: not probed
            raise Unestablished('Model.isUpdatable')
        else:
            raise Unestablished('Model type field %d (not a tree ensemble)' % f)
    if m['kind'] is None:
        raise Unestablished('not a tree ensemble model')
    if m['tf'] > 3:
        raise Unestablished('postEvaluationTransform %d' % m['tf'])
    return m


# -------------------------------------------------------------- helpers --

def _tree_width(m):
    """Record-field width rule (NOTES.md 'W'): 0 for 0, else bitlen(m-1)+1."""
    return 0 if m == 0 else (m - 1).bit_length() + 1


def _bl1(m):
    return max(1, m.bit_length())


def _f32_exact(x):
    if x != x:
        return False
    try:
        return struct.unpack('<f', struct.pack('<f', x))[0] == x
    except OverflowError:
        return False


def _packed_array(vals, w):
    """[u32 0][u32 w][u64 n] then, if n: [u64 a][u64 a][a words]."""
    if not vals:                       # empty arrays always carry width 1
        return struct.pack('<IIQQ', 0, 1, 0, 0)
    out = struct.pack('<IIQ', 0, w, len(vals))
    a = ((len(vals) - 1) * w) // 64 + 3
    acc = 0
    for i, v in enumerate(vals):
        if v >> w:
            raise ValueError('value does not fit width')
        acc |= v << (i * w)
    out += struct.pack('<QQ', a, a)
    out += b''.join(struct.pack('<Q', (acc >> (64 * i)) & (2**64 - 1)) for i in range(a))
    return out


def _pad8(b):
    return b + b'\0' * (-len(b) % 8)


# ----------------------------------------------------------------- DAT --

def _encode_dat(nodes, npd, base, tf):
    if npd < 1:
        raise Unestablished('numPredictionDimensions %d' % npd)
    if len(base) not in (0, npd):
        raise Unestablished('basePredictionValue length %d != %d' % (len(base), npd))
    base = list(base) if base else [0.0] * npd

    trees = {}
    for n in nodes:
        t = trees.setdefault(n['treeId'], {})
        if n['nodeId'] in t:
            raise Unestablished('duplicate node (%d,%d)' % (n['treeId'], n['nodeId']))
        t[n['nodeId']] = n
        if n['beh'] == 6:
            if not n['ev']:
                raise Unestablished('leaf without evaluation values')
            dims = [i for i, _ in n['ev']]
            if len(set(dims)) != len(dims):
                raise Unestablished('leaf with duplicate evaluation index')
            if max(dims) >= npd:
                raise Unestablished('evaluation index >= numPredictionDimensions')

    # double precision if any branch threshold or any leaf value (spec values,
    # including single-leaf trees) is not exactly a float32  (D_*, F_*, FoldF)
    dbl = any(not _f32_exact(n['fv']) for n in nodes if n['beh'] != 6) or \
        any(not _f32_exact(v) for n in nodes if n['beh'] == 6 for _, v in n['ev'])

    # trees in treeId order (S2_rev, Tree_order, FoldA); single-leaf trees fold
    # into base in that order, after basePredictionValue (FoldB, Fold2)
    recs, roots = [], []
    tree_leaves = []          # per kept tree: list of leaf ev lists (in order)
    all_branch = []
    for tid in sorted(trees):
        t = trees[tid]
        children = set()
        for n in t.values():
            if n['beh'] != 6:
                children.update((n['t'], n['f']))
        rootc = [i for i in t if i not in children]
        if len(rootc) != 1:
            raise Unestablished('tree %d: cannot determine unique root' % tid)
        root = t[rootc[0]]
        if root['beh'] == 6:
            if len(t) != 1:
                raise Unestablished('tree %d: unreachable nodes' % tid)
            for i, v in root['ev']:
                base[i] += v
            continue
        roots.append(len(recs))
        lv = []
        seen = set()

        def visit(n):
            if n['nodeId'] in seen:
                raise Unestablished('tree %d: node reached twice' % tid)
            seen.add(n['nodeId'])
            for c in (n['t'], n['f']):
                if c not in t:
                    raise Unestablished('tree %d: missing child %d' % (tid, c))
            tn, fn = t[n['t']], t[n['f']]
            beh, miss = n['beh'], n['miss']
            all_branch.append(n)
            tl, fl = tn['beh'] == 6, fn['beh'] == 6
            if tl and not fl:      # leaf must be the false child (Chain3, Swap_*)
                tn, fn, tl, fl = fn, tn, fl, tl
                beh, miss = NEGATE[beh], not miss
            rec = dict(fi=n['fi'], fv=n['fv'], beh=beh, miss=miss, tl=tl, fl=fl)
            recs.append(rec)
            if tl:
                seen.add(tn['nodeId']); rec['leaf'] = len(lv); lv.append(tn['ev'])
            if fl:
                seen.add(fn['nodeId'])
                if not tl: rec['leaf'] = len(lv)
                lv.append(fn['ev'])
            if not tl:
                me = len(recs) - 1
                visit(tn)
                if not fl:
                    rec['off'] = len(recs) - me
                    visit(fn)
        visit(root)
        if len(seen) != len(t):
            raise Unestablished('tree %d: unreachable nodes' % tid)
        tree_leaves.append(lv)

    nrec = len(recs)
    # value-layout choice counts every leaf node of the spec, including the
    # single-leaf trees folded into base (RND44, RND55, RND62, RND65)
    L_all = [n['ev'] for n in nodes if n['beh'] == 6]
    n_vals = sum(len(ev) for ev in L_all)
    D = len(L_all) * npd
    if npd == 1:
        kind = 3
    elif n_vals < (3 * D) // 4:            # grid G_*, GT_* (180 points)
        kind = 4 if all(len(ev) == 1 for ev in L_all) else 5
    else:
        kind = 6

    # comparison-mode family (j) and flag bits (d)
    fams = set()
    for n in all_branch:
        fams.add({0: 'le', 3: 'le', 1: 'lt', 2: 'lt'}.get(n['beh'], 'eq'))
    allmiss = all(n['miss'] for n in all_branch)
    if fams <= {'le'}:
        j, d = (1 if allmiss else 0), 4
    elif fams == {'lt'}:
        j, d = (3 if allmiss else 2), 4
    else:
        j, d = 4, 6

    maxL = max((len(lv) for lv in tree_leaves), default=0)
    hdim = _tree_width(npd - 1)
    if nrec == 0:
        c, e, g = 65, 65, 0            # Bal0, Fold2, N2_leaf
    else:
        c = _tree_width(max(r['fi'] for r in recs))
        g = _tree_width(nrec)
        if kind == 5:
            e = _tree_width(maxL * npd)
        else:
            e = _tree_width(maxL)
    # S5_2t, S5_lf, GT_3_*, RND62: width over all spec leaves x npd, k=5 only
    i_w = _tree_width(len(L_all) * npd) if kind == 5 else 0
    h = hdim + (1 if kind == 5 else 0)
    tagbits = c + d + e
    T = (tagbits + 7) // 8
    vbytes = 8 if dbl else 4
    recbytes = T + vbytes

    # per-leaf child field value: leaf index, or value offset within tree (k=5)
    rec_list = []
    for lv in tree_leaves:
        offs, acc = [], 0
        for ev in lv:
            offs.append(acc); acc += len(ev)
        rec_list.append(offs)
    # assign records to trees for value offsets
    tree_of_rec = []
    for ti, r0 in enumerate(roots):
        r1 = roots[ti + 1] if ti + 1 < len(roots) else nrec
        tree_of_rec += [ti] * (r1 - r0)

    body = bytearray()
    for idx, r in enumerate(recs):
        tag = r['fi']
        sh = c
        tag |= int(r['tl']) << sh; tag |= int(r['fl']) << (sh + 1)
        tag |= int(r['miss']) << (sh + 2)
        if d == 4:
            tag |= int(r['beh'] in (2, 3)) << (sh + 3)
        else:
            tag |= r['beh'] << (sh + 3)
        if 'leaf' in r:
            fld = rec_list[tree_of_rec[idx]][r['leaf']] if kind == 5 else r['leaf']
        else:
            fld = r.get('off', 0)
        if fld >> e:
            raise ValueError('child field overflow')
        tag |= fld << (8 * T - e)
        body += tag.to_bytes(T, 'little')
        body += struct.pack('<d' if dbl else '<f', r['fv'])
    body += b'\0' * recbytes          # trailing zero record

    out = bytearray()
    out += struct.pack('<QQQ', 0, kind | (8 if dbl else 0), npd)
    out += struct.pack('<%dd' % npd, *base)
    out += struct.pack('<I', 0)
    out += struct.pack('<HBBBBBBBBH', recbytes, c, d, e, 0, g, h, i_w, j, 0)
    out += struct.pack('<QQQ', 0, npd, len(body))
    out += body
    out = bytearray(_pad8(bytes(out)))
    out += _packed_array(roots, _bl1(max(roots, default=0)))

    vals, idxs, tail = [], [], []
    for lv in tree_leaves:
        tail.append(len(vals) if kind == 5 else 0)
        for ev in lv:
            evs = sorted(ev)
            if kind == 3:
                vals.append(evs[0][1])
            elif kind == 4:
                idxs.append(evs[0][0]); vals.append(evs[0][1])
            elif kind == 5:
                for k2, (dim, v) in enumerate(evs):
                    idxs.append(dim | ((k2 == len(evs) - 1) << hdim)); vals.append(v)
            else:
                dense = [0.0] * npd
                for dim, v in evs: dense[dim] = v
                vals += dense
    if kind == 4:
        out += _packed_array(idxs, _bl1(max(idxs, default=0)))
    elif kind == 5:
        out += _packed_array(idxs, h)
    else:
        out += _packed_array([], 1)
    out += struct.pack('<Q', len(vals))
    out += struct.pack('<%d%s' % (len(vals), 'd' if dbl else 'f'), *vals)
    out = bytearray(_pad8(bytes(out)))
    if kind == 5:
        out += _packed_array(tail, _bl1(max(tail, default=0)))
    else:
        out += _packed_array([], 1)
    out += struct.pack('<I', tf)
    return bytes(out)


# ------------------------------------------------------- coremldata.bin --

def _canon_description(desc):
    """Re-serialize ModelDescription as Xcode does (MD1, MD2, MD3): fields in
    field-number order, empty proto3 strings dropped, metadata (100) always
    present. Nested messages are copied; they must already be canonical."""
    fl = []
    for f, wt, v, raw in _wire_fields(desc or b''):
        if f in (11, 12) and wt == 2 and len(v) == 0:
            continue
        if f in (1, 10, 50):
            last = 0
            for g, _wt, _v, _r in _wire_fields(v):
                if g < last:
                    raise Unestablished('non-canonical FeatureDescription field order')
                last = g
        fl.append((f, raw))
    if not any(f == 100 for f, _ in fl):
        fl.append((100, b'\xa2\x06\x00'))
    fl.sort(key=lambda x: x[0])            # stable
    return b''.join(r for _, r in fl)


def _encode_coremldata(m, npd):
    desc = _canon_description(m['desc'])
    out = struct.pack('<II', m['kind'], 1) + b'\0' * 20
    out += struct.pack('<Q', 7) + b'generic'
    out += struct.pack('<I', m['sv']) + b'\0' * 28
    out += struct.pack('<Q', len(desc)) + desc
    out += struct.pack('<Q', 0)
    out += struct.pack('<Q', 10) + b'_B0000.DAT'
    out += struct.pack('<Q', npd)
    if m['kind'] == 402:
        strs = m['strs'] or []
        ints = m['ints'] or []
        if not strs and not ints:
            raise Unestablished('classifier without class labels')
        out += struct.pack('<Q', len(strs))
        for s in strs:
            out += struct.pack('<Q', len(s)) + s
        out += struct.pack('<Q', len(ints))
        out += b''.join(struct.pack('<q', x) for x in ints)
    return out


def encode_tree_part(part_model_bytes: bytes) -> dict:
    m = _parse_model(part_model_bytes)
    nodes, npd, base = _parse_params(m['params'] or b'')
    return {
        'coremldata.bin': _encode_coremldata(m, npd),
        '_B0000.DAT': _encode_dat(nodes, npd, base, m['tf']),
    }


def compile_pipeline(model_bytes, name, outdir):
    """A pipeline of models compiled each to model<N>/, behind an archive
    naming them."""
    model = Message(model_bytes)
    kind, parts = pipeline_parts(model)
    spec_version = model.int(1)
    if spec_version not in ((1, 3, 8) if image_classifier(kind, parts) else (1,)):
        unestablished("a pipeline of specification %d" % spec_version)
    tag = ("generic." + ".".join("model%dv1.0.0%s" % (i, "generic" if pk in TREE_TYPES else "spec") for i, (pk, _) in enumerate(parts))).encode()
    description = canonical_description(model)
    files = {
        "coremldata.bin": (_u32(kind) + _u32(1) + b"\0" * 12 + _u32(1) + _u32(0) + _u64(len(tag)) + tag + _u32(spec_version) + b"\0" * 28
                           + _u64(len(description)) + description + _u64(len(parts)) + _u64(kind)),
        "metadata.json": metadata_json(model, name).encode("utf-8"),
    }

    def s(text):
        raw = text.encode()
        return _u64(len(raw)) + raw
    files["analytics/coremldata.bin"] = (s("PipelineModelDetails") + _u64(1) + s("modelDimension") + s(str(len(parts)))
                                         + s("SpecificationDetails") + _u64(2) + s("modelHash")
                                         + s(base64.b64encode(hashlib.sha256(model_bytes).digest()).decode()) + s("modelName") + s(name))
    for i, (pk, part) in enumerate(parts):
        if pk in TREE_TYPES:
            for rel, data in encode_tree_part(part).items():
                files["model%d/%s" % (i, rel)] = data
            continue
        part_version = Message(part).int(1)
        files["model%d/coremldata.bin" % i] = _u32(pk) + _u32(1) + b"\0" * 20 + _u64(4) + b"spec" + _u32(part_version) + b"\0" * 28 + part
    bundle = os.path.join(outdir, name + ".mlmodelc")
    for rel, data in files.items():
        os.makedirs(os.path.dirname(os.path.join(bundle, rel)), exist_ok=True)
        with open(os.path.join(bundle, rel), "wb") as f:
            f.write(data)


def compile_tree(model_bytes, name, outdir):
    """A tree ensemble on its own: its packed trees, behind the archive a
    pipeline's tree part has."""
    model = Message(model_bytes)
    if model.int(1) not in AVAILABILITY:
        unestablished("a tree ensemble of specification %d" % model.int(1))

    def s(text):
        raw = text.encode()
        return _u64(len(raw)) + raw
    files = dict(encode_tree_part(model_bytes))
    files["metadata.json"] = metadata_json(model, name).encode("utf-8")
    files["analytics/coremldata.bin"] = (s("SpecificationDetails") + _u64(2) + s("modelHash")
                                         + s(base64.b64encode(hashlib.sha256(model_bytes).digest()).decode()) + s("modelName") + s(name))
    bundle = os.path.join(outdir, name + ".mlmodelc")
    for rel, data in files.items():
        os.makedirs(os.path.dirname(os.path.join(bundle, rel)), exist_ok=True)
        with open(os.path.join(bundle, rel), "wb") as f:
            f.write(data)


def refuse_model_file(model):
    """A .mlmodel file's refusal: ML Programs come as .mlpackage, and the
    other model types aren't reproduced yet."""
    kinds = [MODEL_TYPES.get(n, "type %d" % n) for n, _, _ in model.fields if n not in (1, 2, 10)]
    kind = kinds[0] if kinds else "an unknown"
    if model.has(500) or model.has(403) or model.has(303):
        unestablished("a neural network (an older Core ML format; convert the model to an ML Program)")
    unestablished("a %s model (.mlmodel)" % kind)


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
    1: "macOS 10.13, iOS 11.0, tvOS 11.0, watchOS 4.0, visionOS 1.0",
    3: "macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0",
    6: "macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0",
    7: "macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0",
    8: "macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0",
    9: "macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0",
}

# Feature names the generated code (Swift and Objective-C alike) gives a
# trailing "_", observed from Xcode 27.0's coremlc. Names outside this set
# were seen kept as they are.
SWIFT_RESERVED = set("""
out var in inout self class let import return if for while struct protocol id int float double char void BOOL YES NO
nil default switch case operator typealias extension init deinit subscript static public private internal Type Protocol
super where as is break continue fallthrough enum union const signed unsigned long short auto register extern volatile
goto sizeof typedef oneway bycopy byref restrict inline Self do else convenience dynamic final lazy optional required
override weak unowned mutating nonmutating prefix postfix infix get set didSet NULL Class SEL IMP nonatomic atomic
error _Bool dynamicType left right
""".split())

ARRAY_SWIFT = {"Float32": ("Float", "floats"), "Float16": ("Float16", "16-bit floats"), "Double": ("Double", "doubles"),
               "Int32": ("Int32", "32-bit integers")}


class Feature:
    def __init__(self, f):
        self.name = f.str(1)
        self.description = f.str(2)
        t = f.msg(3)
        self.optional = bool(t.int(1000))
        if self.optional and not t.has(5):
            unestablished("an optional feature other than a multi-array's generated code")
        self.swift_name = self.name + "_" if self.name in SWIFT_RESERVED else self.name
        if not self.swift_name.replace("_", "a").isalnum() or not self.swift_name.isascii() or self.swift_name[0].isdigit():
            unestablished("generated code for a feature named %r" % self.name)
        label = self.description or self.name
        if t.has(5):
            a = t.msg(5)
            dtype = ARRAY_TYPES.get(a.int(2)) or unestablished("array data type %d" % a.int(2))
            self.element, words = ARRAY_SWIFT[dtype]
            shape = _ints(a.all(1))
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
            if space not in (10, 20, 30, 40):
                unestablished("color space %d" % space)
            self.width, self.height = im.int(1), im.int(2)
            self.kind = "image"
            self.swift_type = "CVPixelBuffer"
            fmt = {10: "kCVPixelFormatType_OneComponent8", 40: "kCVPixelFormatType_OneComponent16Half"}.get(space, "kCVPixelFormatType_32BGRA")
            self.phrase = "%s (%s) image buffer, %d pixels wide by %d pixels high" % (
                "grayscale" if space in (10, 40) else "color", fmt, self.width, self.height)
            self.cg_format = {10: "kCVPixelFormatType_OneComponent8", 20: "kCVPixelFormatType_32ARGB", 30: "kCVPixelFormatType_32BGRA",
                              40: "kCVPixelFormatType_OneComponent16Half"}[space]
        elif t.has(3):
            self.kind, self.swift_type, self.phrase = "string", "String", "string value"
        elif t.has(1):
            self.kind, self.swift_type, self.phrase = "int64", "Int64", "integer value"
        elif t.has(2):
            self.kind, self.swift_type, self.phrase = "double", "Double", "double value"
        elif t.has(6):
            d = t.msg(6)
            key = "Int64" if d.has(1) else "String"
            self.kind = "dictionary"
            self.swift_type = "[%s : Double]" % key
            self.phrase = "dictionary of %s to doubles" % ("64-bit integers" if key == "Int64" else "strings")
        elif t.has(7):
            q = t.msg(7)
            if q.has(3):
                self.kind, self.swift_type, self.phrase, self.sequence = "sequence", "[String]", "sequence of strings", "string"
            elif q.has(1):
                self.kind, self.swift_type, self.phrase, self.sequence = "sequence", "[Int64]", "sequence of 64-bit integers", "int64"
            else:
                unestablished("generated code for a sequence of another type")
        else:
            unestablished("generated code for this feature type")
        if self.optional:
            self.phrase = "optional " + self.phrase
        self.doc = "%s as %s" % (label, self.phrase)
        self.param_doc = ("%s as %s" % (self.description, self.phrase)) if self.description else self.phrase

    def value(self, expr):
        if self.kind == "sequence":
            return "MLFeatureValue(sequence: %s)" % ("MLSequence(strings: %s)" if self.sequence == "string" else "MLSequence(int64s: %s as [NSNumber])") % expr
        return {"array": "MLFeatureValue(multiArray: %s)", "image": "MLFeatureValue(pixelBuffer: %s)",
                "string": "MLFeatureValue(string: %s)", "int64": "MLFeatureValue(int64: %s)", "double": "MLFeatureValue(double: %s)",
                "dictionary": "MLFeatureValue(dictionary: %s as [AnyHashable : NSNumber])"}[self.kind] % expr

    def read(self):
        if self.kind == "sequence":
            return ("provider.featureValue(for: \"%s\")!.sequenceValue!.stringValues" if self.sequence == "string"
                    else "provider.featureValue(for: \"%s\")!.sequenceValue!.int64Values as! [Int64]") % self.name
        return {"array": "provider.featureValue(for: \"%s\")!.multiArrayValue!",
                "image": "provider.featureValue(for: \"%s\")!.imageBufferValue!",
                "string": "provider.featureValue(for: \"%s\")!.stringValue",
                "int64": "provider.featureValue(for: \"%s\")!.int64Value",
                "double": "provider.featureValue(for: \"%s\")!.doubleValue",
                "dictionary": "provider.featureValue(for: \"%s\")!.dictionaryValue as! " + self.swift_type}[self.kind] % self.name


def interface(model):
    """(inputs, outputs, states) the generated classes use: a multi-function
    model's default function's."""
    description = model.msg(2) or Message(b"")
    if not model.has(502):
        if description.has(13) or description.has(20):
            unestablished("states or functions outside an ML Program")
        return [Feature(f) for f in description.msgs(1)], [Feature(f) for f in description.msgs(10)], []
    default = default_function(model)
    out = None
    for fname, inputs, outputs, states in functions(model):
        # Xcode checks every function, not just the default.
        input_defaults(inputs)
        features = [Feature(f) for f in inputs], [Feature(f) for f in outputs], [state_name(f) for f in states]
        if fname == default:
            out = features
    return out


def state_name(f):
    """(name, the name the generated code gives it) of a state."""
    t = f.msg(3)
    if not (t and t.has(8)) or t.int(1000):
        unestablished("a state that isn't a multi-array")
    name = f.str(1)
    swift = name + "_" if name in SWIFT_RESERVED else name
    if not swift.replace("_", "a").isalnum() or not swift.isascii() or swift[0].isdigit():
        unestablished("generated code for a state named %r" % name)
    return name, swift


IMAGE_SOURCES = (("cgImage", "With", "CGImage"), ("imageAt", "At", "URL"))


def swift_source(model, name):
    spec_version = model.int(1)
    available = "@available(%s, *)" % (AVAILABILITY_LINE.get(spec_version) or unestablished("specification version %d" % spec_version))
    if vision_model(model):
        available = re.sub(r", watchOS [0-9.]+", "", available) + "\n@available(watchOS, unavailable)"
    inputs, outputs, states = interface(model)
    # Initializers and predictions take optional inputs last.
    params = [f for f in inputs if not f.optional] + [f for f in inputs if f.optional]
    # MLShapedArray came with iOS 15; MLShapedArray<Float16> isn't on Intel
    # Macs, and on others came with macOS 15.
    ios15 = "    @available(%s, *)\n" % AVAILABILITY_LINE[6] if spec_version < 6 else ""
    half = ("    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)\n    @available(macOS, unavailable)\n    @available(macCatalyst, unavailable)\n"
            + ("    #else\n    @available(macOS 15.0, *)\n" if spec_version < 9 else "") + "    #endif\n")

    def shaped(features):
        return ios15 + (half if any(f.kind == "array" and f.element == "Float16" for f in features) else "")
    mk = "MLFeatureValue(%s: %s, pixelsWide: %d, pixelsHigh: %d, pixelFormatType: %s, options: nil).imageBufferValue!"

    def param(f, shape=False, default=True, source=None):
        if source and f.kind == "image":
            return "%s%s %s: %s" % (f.swift_name, source[1], f.swift_name, source[2])
        t = "MLShapedArray<%s>" % f.element if shape and f.kind == "array" else f.swift_type
        return "%s: %s%s" % (f.swift_name, t, ("? = nil" if default else "?") if f.optional else "")

    def arg(f, shape=False, source=None):
        n = f.swift_name
        if source and f.kind == "image":
            return "%s: try %s" % (n, mk % (source[0], n, f.width, f.height, f.cg_format))
        if shape and f.kind == "array":
            return "%s: %s" % (n, ("%s != nil ? MLMultiArray(%s!) : nil" % (n, n)) if f.optional else "MLMultiArray(%s)" % n)
        return "%s: %s" % (n, n)
    L = []
    w = L.append
    w("//\n// %s.swift\n//\n// This file was automatically generated and should not be edited.\n//\n\nimport CoreML\n\n" % name)
    # Input.
    w("/// Model Prediction Input Type\n%s\npublic class %sInput : MLFeatureProvider {\n" % (available, name))
    for f in inputs:
        w("    /// %s\n    public var %s: %s%s\n" % (f.doc, f.swift_name, f.swift_type, "? = nil" if f.optional else ""))
    w("    public var featureNames: Set<String> { [%s] }\n" % ", ".join('"%s"' % f.name for f in inputs))
    w("    public func featureValue(for featureName: String) -> MLFeatureValue? {")
    for f in inputs:
        value = ("%s == nil ? nil : %s" % (f.swift_name, f.value(f.swift_name + "!"))) if f.optional else f.value(f.swift_name)
        w("        if featureName == \"%s\" {\n            return %s\n        }" % (f.name, value))
    w("        return nil\n    }\n")
    w("    public init(%s) {" % ", ".join(param(f) for f in params))
    for f in params:
        w("        self.%s = %s" % (f.swift_name, f.swift_name))
    w("    }\n")
    arrays = [f for f in inputs if f.kind == "array"]
    if arrays:
        w(shaped(inputs) + "    public convenience init(%s) {" % ", ".join(param(f, True) for f in params))
        w("        self.init(%s)" % ", ".join(arg(f, True) for f in params))
        w("    }\n")
    images = [f for f in inputs if f.kind == "image"]
    if images:
        # Images from CGImage and URL came with iOS 13.
        ios13 = "    @available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)\n" if spec_version < 4 else ""
        # From each source, with arrays as MLMultiArray and then as MLShapedArray.
        for source in IMAGE_SOURCES:
            for shape in (False, True) if arrays else (False,):
                w(ios13 + (shaped(inputs) if shape else "") + "    public convenience init(%s) throws {\n        self.init(%s)\n    }\n" % (
                    ", ".join(param(f, shape, source=source) for f in params), ", ".join(arg(f, shape, source) for f in params)))
        for source in IMAGE_SOURCES:
            for f in images:
                cap = f.swift_name[:1].upper() + f.swift_name[1:]
                w(ios13 + "    public func set%s(with %s: %s) throws  {\n        self.%s = try %s\n    }\n" % (
                    cap, f.swift_name, source[2], f.swift_name, mk % (source[0], f.swift_name, f.width, f.height, f.cg_format)))
    w("}\n\n")
    # Output.
    w("/// Model Prediction Output Type\n%s\npublic class %sOutput : MLFeatureProvider {\n" % (available, name))
    w("    /// Source provided by CoreML\n    private let provider : MLFeatureProvider\n")
    for f in outputs:
        w("    /// %s\n    public var %s: %s {\n        %s\n    }\n" % (f.doc, f.swift_name, f.swift_type, f.read()))
        if f.kind == "array":
            w("    /// %s\n" % f.doc + shaped([f]) + "    public var %sShapedArray: MLShapedArray<%s> {\n        MLShapedArray<%s>(%s)\n    }\n" % (f.swift_name, f.element, f.element, f.swift_name))
    w("    public var featureNames: Set<String> {\n        provider.featureNames\n    }\n")
    w("    public func featureValue(for featureName: String) -> MLFeatureValue? {\n        provider.featureValue(for: featureName)\n    }\n")
    w("    public init(%s) {" % ", ".join("%s: %s" % (f.swift_name, f.swift_type) for f in outputs))
    w("        self.provider = try! MLDictionaryFeatureProvider(dictionary: [%s])" % ", ".join('"%s" : %s' % (f.name, f.value(f.swift_name)) for f in outputs))
    w("    }\n")
    w("    public init(features: MLFeatureProvider) {\n        self.provider = features\n    }\n}\n\n")
    if states:
        L[-1] = L[-1].rstrip("\n") + "\n\n" + (SWIFT_STATE.replace("{available}", available).replace("{N}", name).replace(
            "{cases}", "\n".join('        case %s = "%s"' % (swift, n) for n, swift in states)))
    template = MODEL_CLASS_SPEC3 if spec_version < 6 else MODEL_CLASS
    model_class = template.replace("{available}", available).replace("{N}", name).rstrip("\n") + "\n"
    if states:
        # Predictions use and update a state.
        for old, new, count in SWIFT_STATE_PREDICTIONS:
            old, new = old.replace("{N}", name), new.replace("{N}", name).replace("{first}", states[0][1])
            if model_class.count(old) != count:
                unestablished("the predictions of a model with states")
            model_class = model_class.replace(old, new)
    if spec_version < 3:
        # Configurations and batches need iOS 12 and its contemporaries.
        ios12 = "    @available(%s, *)\n" % AVAILABILITY_LINE[3]
        for anchor in ("    public convenience init(configuration: MLModelConfiguration) throws {",
                       "    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {"):
            if model_class.count(anchor) != 1:
                unestablished("the configuration initializers of an older model")
            model_class = model_class.replace(anchor, ios12 + anchor)
    if 6 <= spec_version < 8:
        # Asynchronous prediction needs iOS 17 and its contemporaries.
        anchor = "    public func prediction(input: %sInput, options: MLPredictionOptions = MLPredictionOptions()) async throws" % name
        if model_class.count(anchor) != 1:
            unestablished("the asynchronous prediction of an older model")
        model_class = model_class.replace(anchor, "    @available(%s, *)\n" % AVAILABILITY_LINE[8] + anchor)
    w(model_class)
    # Convenience predictions.
    docs = "\n".join("            - %s: %s" % (f.swift_name, f.param_doc) for f in params)
    if states:
        docs += "\n            - state: the state that the prediction will use and update."
    doc = ("    /**\n        Make a prediction using the convenience interface\n\n        It uses the default function if the model has multiple functions.\n\n"
           "        - parameters:\n%s\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as %sOutput\n    */\n") % (docs, name)
    using = ", using state: %sState" % name if states else ""
    call = "        let input_ = %sInput(%s)\n        return try prediction(input: input_%s)\n    }\n" % (
        name, ", ".join("%s: %s" % (f.swift_name, f.swift_name) for f in params), ", using: state" if states else "")
    w(doc + "    public func prediction(%s%s) throws -> %sOutput {\n" % (", ".join(param(f, default=False) for f in params), using, name) + call)
    if arrays:
        w(doc + "\n" + shaped(inputs) + "    public func prediction(%s%s) throws -> %sOutput {\n" % (", ".join(param(f, True, default=False) for f in params), using, name) + call)
    if states:
        # No batch predictions with a state.
        L[-1] += "}\n"
        return "\n".join(L)
    batch = (BATCH_SPEC3 if spec_version < 6 else BATCH).replace("{N}", name)
    if spec_version < 3:
        anchor = "    public func predictions(inputs: [%sInput]" % name
        if batch.count(anchor) != 1:
            unestablished("the batch predictions of an older model")
        batch = batch.replace(anchor, "    @available(%s, *)\n" % AVAILABILITY_LINE[3] + anchor)
    w(batch)
    return "\n".join(L)


SWIFT_STATE = '/// Model Prediction State Type\n{available}\npublic final class {N}State {\n    public enum Name: String, CaseIterable {\n{cases}\n    }\n\n    let handle: MLState\n\n    init(handle: MLState) {\n        self.handle = handle\n    }\n\n    public func withMultiArray<R>(for stateName: Name, _ body: (MLMultiArray) throws -> R) rethrows -> R {\n        try handle.withMultiArray(for: stateName.rawValue, body)\n    }\n}\n'

# (text, its replacement, occurrences) turning MODEL_CLASS's predictions into
# those of a model with states.
SWIFT_STATE_PREDICTIONS = [
    ('    /**\n        Make a prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - input: the input to the prediction as {N}Input\n\n',
     '    /**\n        Make a new state.\n\n        Core ML framework will allocate the state buffers declared in the model.\n\n        The allocated state buffers are initialized to zeros. To initialize with different values, use `.withMultiArray(for:)` to get the mutable `MLMultiArray`-view to the state buffer.\n\n        ```swift\n        let state = model.makeState()\n        state.withMultiArray(for: .{first}) { stateMultiArray in\n            stateMultiArray[0] = 0.42\n        }\n        ```\n    */\n    public func makeState() -> {N}State {\n        {N}State(handle: model.makeState())\n    }\n\n    /**\n        Make a prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - input: the input to the prediction as {N}Input\n           - state: the state that the prediction will use and update.\n\n', 1),
    ('           - input: the input to the prediction as {N}Input\n           - options: prediction options\n',
     '           - input: the input to the prediction as {N}Input\n           - state: the state that the prediction will use and update.\n           - options: prediction options\n', 2),
    ('    public func prediction(input: {N}Input) throws -> {N}Output {\n        try prediction(input: input, options: MLPredictionOptions())',
     '    public func prediction(input: {N}Input, using state: {N}State) throws -> {N}Output {\n        try prediction(input: input, using: state, options: MLPredictionOptions())', 1),
    ('prediction(input: {N}Input, options:', 'prediction(input: {N}Input, using state: {N}State, options:', 2),
    ('prediction(from: input, options: options)', 'prediction(from: input, using: state.handle, options: options)', 2),
]

MODEL_CLASS = '/// Class for model loading and prediction\n{available}\npublic class {N} {\n    public let model: MLModel\n\n    /// URL of model assuming it was installed in the same bundle as this class\n    class var urlOfModelInThisBundle : URL {\n        let bundle = Bundle(for: self)\n        return bundle.url(forResource: "{N}", withExtension:"mlmodelc")!\n    }\n\n    /**\n        Construct {N} instance with an existing MLModel object.\n\n        Usually the application does not use this initializer unless it makes a subclass of {N}.\n        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `{N}.urlOfModelInThisBundle` to create a MLModel object to pass-in.\n\n        - parameters:\n          - model: MLModel object\n    */\n    init(model: MLModel) {\n        self.model = model\n    }\n\n    /**\n        Construct a model with configuration\n\n        - parameters:\n           - configuration: the desired model configuration\n\n        - throws: an NSError object that describes the problem\n    */\n    public convenience init(configuration: MLModelConfiguration = MLModelConfiguration()) throws {\n        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)\n    }\n\n    /**\n        Construct {N} instance with explicit path to mlmodelc file\n        - parameters:\n           - modelURL: the file url of the model\n\n        - throws: an NSError object that describes the problem\n    */\n    public convenience init(contentsOf modelURL: URL) throws {\n        try self.init(model: MLModel(contentsOf: modelURL))\n    }\n\n    /**\n        Construct a model with URL of the .mlmodelc directory and configuration\n\n        - parameters:\n           - modelURL: the file url of the model\n           - configuration: the desired model configuration\n\n        - throws: an NSError object that describes the problem\n    */\n    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {\n        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))\n    }\n\n    /**\n        Construct {N} instance asynchronously with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - configuration: the desired model configuration\n          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully\n    */\n    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<{N}, Error>) -> Void) {\n        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)\n    }\n\n    /**\n        Construct {N} instance asynchronously with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - configuration: the desired model configuration\n    */\n    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> {N} {\n        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)\n    }\n\n    /**\n        Construct {N} instance asynchronously with URL of the .mlmodelc directory with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - modelURL: the URL to the model\n          - configuration: the desired model configuration\n          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully\n    */\n    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<{N}, Error>) -> Void) {\n        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in\n            switch result {\n            case .failure(let error):\n                handler(.failure(error))\n            case .success(let model):\n                handler(.success({N}(model: model)))\n            }\n        }\n    }\n\n    /**\n        Construct {N} instance asynchronously with URL of the .mlmodelc directory with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - modelURL: the URL to the model\n          - configuration: the desired model configuration\n    */\n    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> {N} {\n        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)\n        return {N}(model: model)\n    }\n\n    /**\n        Make a prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - input: the input to the prediction as {N}Input\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as {N}Output\n    */\n    public func prediction(input: {N}Input) throws -> {N}Output {\n        try prediction(input: input, options: MLPredictionOptions())\n    }\n\n    /**\n        Make a prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - input: the input to the prediction as {N}Input\n           - options: prediction options\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as {N}Output\n    */\n    public func prediction(input: {N}Input, options: MLPredictionOptions) throws -> {N}Output {\n        let outFeatures = try model.prediction(from: input, options: options)\n        return {N}Output(features: outFeatures)\n    }\n\n    /**\n        Make an asynchronous prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - input: the input to the prediction as {N}Input\n           - options: prediction options\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as {N}Output\n    */\n    public func prediction(input: {N}Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> {N}Output {\n        let outFeatures = try await model.prediction(from: input, options: options)\n        return {N}Output(features: outFeatures)\n    }\n\n'

# The model class of specification 3 models (iOS 12 and its contemporaries).
MODEL_CLASS_SPEC3 = '/// Class for model loading and prediction\n{available}\npublic class {N} {\n    public let model: MLModel\n\n    /// URL of model assuming it was installed in the same bundle as this class\n    class var urlOfModelInThisBundle : URL {\n        let bundle = Bundle(for: self)\n        return bundle.url(forResource: "{N}", withExtension:"mlmodelc")!\n    }\n\n    /**\n        Construct {N} instance with an existing MLModel object.\n\n        Usually the application does not use this initializer unless it makes a subclass of {N}.\n        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `{N}.urlOfModelInThisBundle` to create a MLModel object to pass-in.\n\n        - parameters:\n          - model: MLModel object\n    */\n    init(model: MLModel) {\n        self.model = model\n    }\n\n    /**\n        Construct {N} instance by automatically loading the model from the app\'s bundle.\n    */\n    @available(*, deprecated, message: "Use init(configuration:) instead and handle errors appropriately.")\n    public convenience init() {\n        try! self.init(contentsOf: type(of:self).urlOfModelInThisBundle)\n    }\n\n    /**\n        Construct a model with configuration\n\n        - parameters:\n           - configuration: the desired model configuration\n\n        - throws: an NSError object that describes the problem\n    */\n    public convenience init(configuration: MLModelConfiguration) throws {\n        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)\n    }\n\n    /**\n        Construct {N} instance with explicit path to mlmodelc file\n        - parameters:\n           - modelURL: the file url of the model\n\n        - throws: an NSError object that describes the problem\n    */\n    public convenience init(contentsOf modelURL: URL) throws {\n        try self.init(model: MLModel(contentsOf: modelURL))\n    }\n\n    /**\n        Construct a model with URL of the .mlmodelc directory and configuration\n\n        - parameters:\n           - modelURL: the file url of the model\n           - configuration: the desired model configuration\n\n        - throws: an NSError object that describes the problem\n    */\n    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {\n        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))\n    }\n\n    /**\n        Construct {N} instance asynchronously with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - configuration: the desired model configuration\n          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully\n    */\n    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)\n    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<{N}, Error>) -> Void) {\n        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)\n    }\n\n    /**\n        Construct {N} instance asynchronously with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - configuration: the desired model configuration\n    */\n    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)\n    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> {N} {\n        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)\n    }\n\n    /**\n        Construct {N} instance asynchronously with URL of the .mlmodelc directory with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - modelURL: the URL to the model\n          - configuration: the desired model configuration\n          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully\n    */\n    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)\n    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<{N}, Error>) -> Void) {\n        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in\n            switch result {\n            case .failure(let error):\n                handler(.failure(error))\n            case .success(let model):\n                handler(.success({N}(model: model)))\n            }\n        }\n    }\n\n    /**\n        Construct {N} instance asynchronously with URL of the .mlmodelc directory with optional configuration.\n\n        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.\n\n        - parameters:\n          - modelURL: the URL to the model\n          - configuration: the desired model configuration\n    */\n    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)\n    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> {N} {\n        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)\n        return {N}(model: model)\n    }\n\n    /**\n        Make a prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - input: the input to the prediction as {N}Input\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as {N}Output\n    */\n    public func prediction(input: {N}Input) throws -> {N}Output {\n        try prediction(input: input, options: MLPredictionOptions())\n    }\n\n    /**\n        Make a prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - input: the input to the prediction as {N}Input\n           - options: prediction options\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as {N}Output\n    */\n    public func prediction(input: {N}Input, options: MLPredictionOptions) throws -> {N}Output {\n        let outFeatures = try model.prediction(from: input, options: options)\n        return {N}Output(features: outFeatures)\n    }\n\n    /**\n        Make an asynchronous prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - input: the input to the prediction as {N}Input\n           - options: prediction options\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as {N}Output\n    */\n    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)\n    public func prediction(input: {N}Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> {N}Output {\n        let outFeatures = try await model.prediction(from: input, options: options)\n        return {N}Output(features: outFeatures)\n    }\n\n'

BATCH_SPEC3 = '    /**\n        Make a batch prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - inputs: the inputs to the prediction as [{N}Input]\n           - options: prediction options\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as [{N}Output]\n    */\n    public func predictions(inputs: [{N}Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [{N}Output] {\n        let batchIn = MLArrayBatchProvider(array: inputs)\n        let batchOut = try model.predictions(from: batchIn, options: options)\n        var results : [{N}Output] = []\n        results.reserveCapacity(inputs.count)\n        for i in 0..<batchOut.count {\n            let outProvider = batchOut.features(at: i)\n            let result =  {N}Output(features: outProvider)\n            results.append(result)\n        }\n        return results\n    }\n}\n'

BATCH = '    /**\n        Make a batch prediction using the structured interface\n\n        It uses the default function if the model has multiple functions.\n\n        - parameters:\n           - inputs: the inputs to the prediction as [{N}Input]\n           - options: prediction options\n\n        - throws: an NSError object that describes the problem\n\n        - returns: the result of the prediction as [{N}Output]\n    */\n    public func predictions(inputs: [{N}Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [{N}Output] {\n        let batchIn = MLArrayBatchProvider(array: inputs)\n        let batchOut = try model.predictions(from: batchIn, options: options)\n        var results : [{N}Output] = []\n        results.reserveCapacity(inputs.count)\n        for i in 0..<batchOut.count {\n            let outProvider = batchOut.features(at: i)\n            let result =  {N}Output(features: outProvider)\n            results.append(result)\n        }\n        return results\n    }\n}\n'


# --- coremlc generate (Objective-C) -----------------------------------------------

API_AVAILABLE = {
    1: "macos(10.13), ios(11.0), watchos(4.0), tvos(11.0)",
    3: "macos(10.14), ios(12.0), watchos(5.0), tvos(12.0)",
    6: "macos(12.0), ios(15.0), watchos(8.0), tvos(15.0)",
    7: "macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)",
    8: "macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)",
    9: "macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)",
}


def _objc(f):
    """(property attributes and type, parameter type, value expression, read expression) of a feature."""
    if f.kind == "array":
        out = "(readwrite, nonatomic, strong) MLMultiArray * ", "MLMultiArray *", "featureValueWithMultiArray:self.%s", "(MLMultiArray *)[%%s featureValueForName:@\"%s\"].multiArrayValue"
    elif f.kind == "image":
        out = "(readwrite, nonatomic) CVPixelBufferRef ", "CVPixelBufferRef", "featureValueWithPixelBuffer:self.%s", "(CVPixelBufferRef)[%%s featureValueForName:@\"%s\"].imageBufferValue"
    elif f.kind == "string":
        out = "(readwrite, nonatomic, strong) NSString * ", "NSString *", "featureValueWithString:self.%s", "(NSString *)[%%s featureValueForName:@\"%s\"].stringValue"
    elif f.kind == "int64":
        out = "(readwrite, nonatomic) int64_t ", "int64_t", "featureValueWithInt64:self.%s", "[%%s featureValueForName:@\"%s\"].int64Value"
    elif f.kind == "double":
        out = "(readwrite, nonatomic) double ", "double", "featureValueWithDouble:self.%s", "[%%s featureValueForName:@\"%s\"].doubleValue"
    elif f.kind == "sequence":
        element = "NSString *" if f.sequence == "string" else "NSNumber *"
        t = "NSArray<%s> *" % element
        make = "sequenceWithStringArray" if f.sequence == "string" else "sequenceWithInt64Array"
        values = "stringValues" if f.sequence == "string" else "int64Values"
        out = ("(readwrite, nonatomic, strong) %s " % t, t, "featureValueWithSequence:[MLSequence %s:self.%%s]" % make,
               "(" + t + ")[%%s featureValueForName:@\"%s\"].sequenceValue." + values)
    else:
        key = "NSNumber *" if "Int64" in f.swift_type else "NSString *"
        t = "NSDictionary<%s, NSNumber *> *" % key
        out = "(readwrite, nonatomic, strong) %s " % t, t, "featureValueWithDictionary:self.%s error:nil", "(" + t + ")[%%s featureValueForName:@\"%s\"].dictionaryValue"
    if f.optional:
        # An optional multi-array is nullable.
        out = (out[0].replace(") ", ", nullable) ", 1), "nullable " + out[1],
               "self.%s == nil ? nil : [MLFeatureValue featureValueWithMultiArray:(MLMultiArray *)self.%s]", out[3])
    return out


def _selector(prefix, features, value, source=None):
    """`prefixFirst:(value) second:(value) ...`: the first part capitalizes
    the feature's own name, later parts use the name the code gives it; an
    image from a CGImage or URL says so."""
    parts = []
    for k, f in enumerate(features):
        label = prefix + f.name[:1].upper() + f.name[1:] if k == 0 else f.swift_name
        # The convenience prediction keeps clear of the standard
        # predictionFromFeatures:.
        if label == "predictionFromFeatures":
            label += "_"
        if source and f.kind == "image":
            label += ("FromCGImage" if k == 0 else "CGImage") if source[0] == "cgImage" else "AtURL"
        parts.append("%s:%s" % (label, value(f)))
    return " ".join(parts)


def _objc_param(f, source=None):
    if source and f.kind == "image":
        return "(CGImageRef)%s" % f.swift_name if source[0] == "cgImage" else "(NSURL *)%sURL" % f.swift_name
    return "(%s)%s" % (_objc(f)[1], f.swift_name)


def objc_sources(model, name):
    spec_version = model.int(1)
    available = "API_AVAILABLE(%s) __attribute__((visibility(\"hidden\")))" % (API_AVAILABLE.get(spec_version) or unestablished("specification version %d" % spec_version))
    if vision_model(model):
        available = re.sub(r", watchos\([0-9.]+\)", "", available).replace(") __attribute__", ") API_UNAVAILABLE(watchos)  __attribute__", 1)
    inputs, outputs, states = interface(model)
    for f in inputs + outputs:
        if not (f.name[:1].isalpha() or f.name[:1] == "_"):
            unestablished("generated code for a feature named %r" % f.name)
    images = [f for f in inputs if f.kind == "image"]
    # Initializers without the optional inputs come first.
    required = [f for f in inputs if not f.optional]
    reduced = required if len(required) < len(inputs) else None
    ios13 = " API_AVAILABLE(macos(10.15), ios(13.0), watchos(6.0), tvos(13.0)) __attribute__((visibility(\"hidden\")))" if spec_version < 4 else ""
    error = " error:(NSError * _Nullable __autoreleasing * _Nullable)error"
    H = []
    H.append("//\n// %s.h\n//\n// This file was automatically generated and should not be edited.\n//\n\n#import <Foundation/Foundation.h>\n#import <CoreML/CoreML.h>\n#include <stdint.h>\n#include <os/log.h>\n\nNS_ASSUME_NONNULL_BEGIN\n" % name)
    for kind, features in (("Input", inputs), ("Output", outputs)):
        H.append("/// Model Prediction %s Type\n%s\n@interface %s%s : NSObject<MLFeatureProvider>\n" % (kind, available, name, kind))
        H.append("\n\n".join("/// %s\n@property %s%s;" % (f.doc, _objc(f)[0], f.swift_name) for f in features))
        H[-1] += "\n- (instancetype)init NS_UNAVAILABLE;\n"
        if kind == "Input" and reduced is not None:
            H[-1] += "- (instancetype)%s;\n" % _selector("initWith", reduced, _objc_param)
        H[-1] += "- (instancetype)%s NS_DESIGNATED_INITIALIZER;\n" % _selector("initWith", features, _objc_param)
        if kind == "Input" and images:
            for source in IMAGE_SOURCES:
                H.append("".join("- (nullable instancetype)%s%s%s;\n" % (_selector("initWith", fs, lambda f: _objc_param(f, source), source), error, ios13)
                                 for fs in ([reduced] if reduced is not None else []) + [inputs]))
            for source in IMAGE_SOURCES:
                for f in images:
                    cap = f.swift_name[:1].upper() + f.swift_name[1:]
                    H.append("-(BOOL)set%sWith%s:%s%s %s;" % (cap, source[2], _objc_param(f, source), error, ios13))
        H.append("@end\n")
    if states:
        H[-1] += "\n/// Model Prediction State Names\ntypedef NS_ENUM(NSInteger, %sStateName) {\n%s\n} %s;\n" % (
            name, "\n".join("    %sStateName%s," % (name, swift[:1].upper() + swift[1:]) for _, swift in states), available)
        H[-1] += "\n/// Model Prediction State Type\n%s\n@interface %sState : NSObject\n@property (readonly, strong, nonatomic) MLState *state;\n" % (available, name)
        H[-1] += "- (void)getMultiArrayForState:(%sStateName)stateName\n                      handler:(void (NS_NOESCAPE ^)(MLMultiArray *buffer))handler;\n@end\n" % name
    header_model = H_MODEL.replace("{available}", available).replace("{N}", name)
    if states:
        # Predictions use and update a state.
        for old, new, count in OBJC_STATE_PREDICTIONS:
            old, new = old.replace("{N}", name), new.replace("{N}", name)
            if header_model.count(old) != count:
                unestablished("the predictions of a model with states")
            header_model = header_model.replace(old, new)
    if spec_version < 6:
        # Loading with a completion handler needs iOS 14 and its contemporaries.
        for anchor in ("completionHandler:(void (^)(%s * _Nullable model, NSError * _Nullable error))handler;" % name,):
            if header_model.count(anchor) != 2:
                unestablished("the asynchronous loading of an older model")
            header_model = header_model.replace(anchor, anchor[:-1] + " API_AVAILABLE(macos(11.0), ios(14.0), watchos(7.0), tvos(14.0)) __attribute__((visibility(\"hidden\")));")
    if spec_version < 8:
        for anchor in ("completionHandler:(void (^)(%sOutput * _Nullable output, NSError * _Nullable error))completionHandler;" % name,):
            if header_model.count(anchor) != 2:
                unestablished("the asynchronous predictions of an older model")
            header_model = header_model.replace(anchor, anchor[:-1] + " API_AVAILABLE(%s) __attribute__((visibility(\"hidden\")));" % API_AVAILABLE[8])
    if spec_version < 3:
        # Configurations came with iOS 12.
        for anchor in ("- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;",
                       "- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;"):
            if header_model.count(anchor) != 1:
                unestablished("the configurations of an older model")
            header_model = header_model.replace(anchor, anchor[:-1] + " API_AVAILABLE(%s) __attribute__((visibility(\"hidden\")));" % API_AVAILABLE[3])
    H.append(header_model.rstrip("\n"))
    params = "\n".join("    @param %s %s" % (f.swift_name, f.param_doc) for f in inputs) + ("\n    @param state prediction state" if states else "")
    using = " usingState:(%sState *)state" % name if states else ""
    H.append("\n/**\n    Make a prediction using the convenience interface\n%s\n    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.\n    @return the prediction as %sOutput\n*/\n- (nullable %sOutput *)%s%s error:(NSError * _Nullable __autoreleasing * _Nullable)error;\n" % (
        params, name, name, _selector("predictionFrom", inputs, _objc_param), using))
    if states:
        # No batch predictions with a state.
        H[-1] = H[-1].rstrip("\n") + "\n@end\n\nNS_ASSUME_NONNULL_END\n"
    else:
        batch = H_BATCH.replace("{N}", name)
        if spec_version < 3:
            batch = batch.replace("error:(NSError * _Nullable __autoreleasing * _Nullable)error;\n@end", "error:(NSError * _Nullable __autoreleasing * _Nullable)error API_AVAILABLE(%s) __attribute__((visibility(\"hidden\")));\n@end" % API_AVAILABLE[3], 1)
        H.append(batch)
    header = "\n".join(H)

    def make_output(src):
        return "[[%sOutput alloc] %s]" % (name, _selector("initWith", outputs, lambda f: (_objc(f)[3] % f.name) % src))

    M = []
    M.append("//\n// %s.m\n//\n// This file was automatically generated and should not be edited.\n//\n\n#if !__has_feature(objc_arc)\n#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)\n#endif\n\n#import \"%s.h\"\n" % (name, name))
    for kind, features in (("Input", inputs), ("Output", outputs)):
        M.append("@implementation %s%s\n" % (name, kind))
        designated = "- (instancetype)%s {\n    self = [super init];\n    if (self) {\n" % _selector("initWith", features, _objc_param)
        for f in features:
            designated += "        _%s = %s;\n" % (f.swift_name, f.swift_name)
            if f.kind == "image":
                designated += "        CVPixelBufferRetain(_%s);\n" % f.swift_name
        designated += "    }\n    return self;\n}\n"
        if kind == "Input" and reduced is not None:
            # The initializers without the optional inputs pass nil for them.
            designated = ("- (instancetype)%s {\n    return [self %s];\n}\n" % (
                _selector("initWith", reduced, _objc_param),
                _selector("initWith", inputs, lambda f: "nil" if f.optional else f.swift_name))
                + "".join(_image_init(source, reduced, reduced) + "\n" for source in IMAGE_SOURCES if images) + designated)
        M.append(designated)
        pictures = [f for f in features if f.kind == "image"]
        if pictures:
            methods = ["- (void)dealloc {\n%s}\n" % "".join("    CVPixelBufferRelease(_%s);\n" % f.swift_name for f in pictures)]
            if kind == "Input":
                methods += [_image_init(source, inputs, inputs) for source in IMAGE_SOURCES]
                methods += [_image_setter(source, f) for source in IMAGE_SOURCES for f in pictures]
            M.append("\n".join(methods))
        M.append("- (NSSet<NSString *> *)featureNames {\n    return [NSSet setWithArray:@[%s]];\n}\n" % ", ".join('@"%s"' % f.name for f in features))
        M.append("- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {")
        for f in features:
            value = _objc(f)[2] % ((f.swift_name, f.swift_name) if f.optional else f.swift_name)
            M.append("    if ([featureName isEqualToString:@\"%s\"]) {\n        return %s;\n    }" % (f.name, value if f.optional else "[MLFeatureValue %s]" % value))
        M.append("    return nil;\n}\n\n@end\n")
    if states:
        M.append(OBJC_STATE.replace("{N}", name).replace("{cases}", "".join(
            "        case %sStateName%s:\n            stateNameString = @\"%s\";\n            break;\n" % (name, swift[:1].upper() + swift[1:], n) for n, swift in states)))
    M.append(M_MODEL.replace("{N}", name).rstrip("\n") + "\n")
    state_arg = " usingState:state" if states else ""
    if states:
        M.append("\n/**\n    Make a new state.\n*/\n- ({N}State *)newState {\n    MLState *state = [self.model newState];\n    return [[{N}State alloc] initWithState:state];\n}\n".replace("{N}", name))
    M.append(("- (nullable {N}Output *)predictionFromFeatures:({N}Input *)input%s error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n"
              "    return [self predictionFromFeatures:input%s options:[[MLPredictionOptions alloc] init] error:error];\n}\n").replace("{N}", name) % (using, state_arg))
    M.append(("- (nullable {N}Output *)predictionFromFeatures:({N}Input *)input%s options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n"
              "    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input%s options:options error:error];\n"
              "    if (!outFeatures) { return nil; }\n    return %s;\n}\n").replace("{N}", name) % (using, state_arg + ".state" if states else "", make_output("outFeatures")))
    for opts in ("", " options:(MLPredictionOptions *)options"):
        call = " options:options" if opts else ""
        if states:
            # With a state, both pass options.
            call = " usingState:state.state options:" + ("options" if opts else "[[MLPredictionOptions alloc] init]")
        M.append(("- (void)predictionFromFeatures:({N}Input *)input%s%s completionHandler:(void (^)({N}Output * _Nullable output, NSError * _Nullable error))completionHandler {\n"
                  "    [self.model predictionFromFeatures:input%s completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {\n"
                  "        if (prediction != nil) {\n            {N}Output *output = %s;\n            completionHandler(output, predictionError);\n"
                  "        } else {\n            completionHandler(nil, predictionError);\n        }\n    }];\n}\n").replace("{N}", name) % (using, opts, call, make_output("prediction")))
    M.append("- (nullable %sOutput *)%s%s error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n    %sInput *input_ = [[%sInput alloc] %s];\n    return [self predictionFromFeatures:input_%s error:error];\n}\n" % (
        name, _selector("predictionFrom", inputs, _objc_param), using, name, name, _selector("initWith", inputs, lambda f: f.swift_name), state_arg))
    if states:
        M.append("@end\n")
        return header, "\n".join(M)
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


def _image_init(source, features, target):
    """An initializer taking `features`' images from a CGImage or URL, which
    calls the one for `target`."""
    make = "[MLFeatureValue featureValueWith%s:%s pixelsWide:%d pixelsHigh:%d pixelFormatType:%s options:nil error:&localError]"
    out = ("- (nullable instancetype)%s error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n"
           "    if (self) {\n        NSError *localError;\n        BOOL result = YES;\n        id retVal = nil;\n        @autoreleasepool {\n            do {\n") % (
        _selector("initWith", features, lambda f: _objc_param(f, source), source))
    for f in features:
        if f.kind == "image":
            n = f.swift_name
            out += ("                MLFeatureValue * __%s = %s;\n                if (__%s == nil) {\n                    result = NO;\n                    break;\n                }\n") % (
                n, make % ("CGImage" if source[0] == "cgImage" else "ImageAtURL", n if source[0] == "cgImage" else n + "URL", f.width, f.height, f.cg_format), n)
    out += ("                retVal = [self %s];\n            }\n            while(0);\n        }\n"
            "        if (error != NULL) {\n            *error = localError;\n        }\n        return result ? retVal : nil;\n    }\n    return self;\n}\n") % (
        _selector("initWith", target, lambda f: "(CVPixelBufferRef)__%s.imageBufferValue" % f.swift_name if f.kind == "image" else f.swift_name))
    return out


def _image_setter(source, f):
    cap = f.swift_name[:1].upper() + f.swift_name[1:]
    n = f.swift_name
    make = "[MLFeatureValue featureValueWith%s:%s pixelsWide:%d pixelsHigh:%d pixelFormatType:%s options:nil error:&localError]" % (
        "CGImage" if source[0] == "cgImage" else "ImageAtURL", n if source[0] == "cgImage" else n + "URL", f.width, f.height, f.cg_format)
    return ("-(BOOL)set%sWith%s:%s error:(NSError * _Nullable __autoreleasing * _Nullable)error {\n    NSError *localError;\n    BOOL result = NO;\n    @autoreleasepool {\n"
            "        MLFeatureValue * __%s = %s;\n        if (__%s != nil) {\n            CVPixelBufferRelease(self.%s);\n"
            "            self.%s =  (CVPixelBufferRef)__%s.imageBufferValue;\n            CVPixelBufferRetain(self.%s);\n            result = YES;\n        }\n    }\n"
            "    if (error != NULL) {\n        *error = localError;\n    }\n    return result;\n}\n") % (cap, source[2], _objc_param(f, source), n, make, n, n, n, n, n)


OBJC_STATE = '@implementation {N}State\n\n- (instancetype)initWithState:(MLState *)state {\n    self = [super init];\n    if (self) {\n        _state = state;\n    }\n    return self;\n}\n\n- (void)getMultiArrayForState:({N}StateName)stateName\n                      handler:(void (NS_NOESCAPE ^)(MLMultiArray *buffer))handler {\n    NSString *stateNameString = nil;\n    switch (stateName) {\n{cases}    }\n\n    [self.state getMultiArrayForStateNamed:stateNameString handler:handler];\n}\n\n@end\n'

# (text, its replacement, occurrences) turning H_MODEL's predictions into
# those of a model with states.
OBJC_STATE_PREDICTIONS = [
    ('/**\n    Make a prediction using the standard interface\n    @param input an instance of {N}Input to predict from\n    @param error',
     '/**\n    Make a new state.\n*/\n- ({N}State *)newState;\n\n/**\n    Make a prediction using the standard interface\n    @param input an instance of {N}Input to predict from\n    @param error', 1),
    ('    @param input an instance of {N}Input to predict from\n', '    @param input an instance of {N}Input to predict from\n    @param state prediction state\n', 4),
    ('predictionFromFeatures:({N}Input *)input error:', 'predictionFromFeatures:({N}Input *)input usingState:({N}State *)state error:', 1),
    ('predictionFromFeatures:({N}Input *)input options:', 'predictionFromFeatures:({N}Input *)input usingState:({N}State *)state options:', 2),
    ('predictionFromFeatures:({N}Input *)input completionHandler:', 'predictionFromFeatures:({N}Input *)input usingState:({N}State *)state completionHandler:', 1),
]


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
        if os.path.isdir(source):
            model = read_package(source)
            if not model.has(502):
                unestablished("a model that isn't an ML Program")
        else:
            with open(source, "rb") as f:
                model = Message(f.read())
            if not any(model.has(n) for n in SPEC_ARCHIVED) and tree_kind(model) is None and pipeline_parts(model) is None:
                refuse_model_file(model)
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
        if os.path.isdir(source):
            compile_package(source, outdir)
        else:
            with open(source, "rb") as f:
                model_bytes = f.read()
            model = Message(model_bytes)
            if any(model.has(n) for n in SPEC_ARCHIVED):
                compile_spec_model(model_bytes, generated_class_name(os.path.splitext(os.path.basename(source))[0]), outdir)
            elif tree_kind(model) is not None:
                compile_tree(model_bytes, generated_class_name(os.path.splitext(os.path.basename(source))[0]), outdir)
            elif pipeline_parts(model) is not None:
                compile_pipeline(model_bytes, generated_class_name(os.path.splitext(os.path.basename(source))[0]), outdir)
            else:
                refuse_model_file(model)
    except Unestablished as e:
        print("error: coremlc: %s is not compiled as Xcode compiles it" % e, file=sys.stderr)
        return 1
    if plist:
        with open(plist, "w") as f:
            f.write(PARTIAL_INFO_PLIST)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
