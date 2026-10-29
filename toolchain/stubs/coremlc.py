#!/usr/bin/env python3
"""coremlc off a Mac: `coremlc compile` of a Core ML model, as rules_apple
runs it (`compile MODEL OUTDIR --output-partial-info-plist PLIST`); see
docs/clean-room/coremlc.md.

ML Programs (.mlpackage) and the other model types (.mlmodel, or in a
.mlpackage with the overlays Xcode's model editor writes) compile to the
files Xcode 27.0's coremlc writes, read from the model's specification
(Apple's published Core ML protobuf schema). Neural networks, an older
format, are refused; so is what hasn't
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


def _minimal_varint(data, i):
    """(value, next index), or None for a varint longer than it needs to be."""
    start, value, shift = i, 0, 0
    while True:
        b = data[i]
        i += 1
        value |= (b & 0x7F) << shift
        shift += 7
        if b < 0x80:
            return None if i - start > 1 and b == 0 else (value, i)


def _wire_message(data):
    """[(number, wire, payload or None)] if `data` parses as a whole message
    with minimal varints, otherwise None."""
    out, i = [], 0
    try:
        while i < len(data):
            key = _minimal_varint(data, i)
            if key is None or key[0] >> 3 == 0:
                return None
            (number, wire), i = divmod(key[0], 8), key[1]
            if wire in (0, 2):
                value = _minimal_varint(data, i)
                if value is None:
                    return None
                value, i = value
                out.append((number, wire, data[i:i + value] if wire == 2 else None))
                i += value if wire == 2 else 0
            elif wire in (1, 5):
                i += 8 if wire == 1 else 4
                out.append((number, wire, None))
            else:
                return None
        return out if i == len(data) else None
    except IndexError:
        return None


def _in_order(fields):
    """Fields in number order, a scalar field at most once (repeated scalars
    packed), as protobuf's serializers write them."""
    last = 0
    for number, wire, _ in fields:
        if number < last or (number == last and wire != 2):
            return False
        last = number
    return True


def check_encoding(model_bytes):
    """Xcode re-encodes the specification it copies; this tool copies it as
    given. So the levels known to be messages must already be encoded as
    protobuf writes them: the Model, its description, the description's
    features and metadata, and the model type's own message."""
    model = _wire_message(model_bytes)
    ok = model is not None and _in_order(model)
    for number, _, payload in model or []:
        if payload is None:
            continue
        fields = _wire_message(payload)
        ok = ok and fields is not None and _in_order(fields)
        if number == 2:
            for n, _, p in fields or []:
                if n in (1, 10, 13, 50, 100) and p is not None:
                    inner = _wire_message(p)
                    ok = ok and inner is not None and _in_order(inner)
    if not ok:
        unestablished("a model not encoded as protobuf's serializers encode it (Xcode re-encodes it)")


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
            "license", "metadataOutputVersion", "customModels", "trainingInputSchema", "updateParameters", "outputSchema", "defaultFunctionName", "stateSchema", "isUpdatable",
            "classLabels", "specificationVersion", "availability", "generatedClassName", "userDefinedMetadata",
            "storagePrecision", "functions", "mlProgramOperationTypeHistogram", "computePrecision"]
SCHEMA_KEYS = ["sizeFlexibility", "enumeratedSizes", "isColor", "colorspace", "hasSizeFlexibility", "shape", "keyType", "type",
            "sizeRange", "dataType", "width", "height", "name", "isOptional", "hasShapeFlexibility", "shortDescription",
            "formattedType", "valueType"]
# A flexible multi-array's keys go in in another order.
FLEXIBLE_ARRAY_KEYS = ["shapeRange", "enumeratedShapes", "shapeFlexibility", "shape", "type", "name", "dataType",
                       "hasShapeFlexibility", "isOptional", "shortDescription", "formattedType"]
# A multi-function model's functions.
# A multi-function model's keys go in in another order.
MULTI_FUNCTION_TOP_KEYS = ["author", "inputSchema", "modelParameters", "shortDescription", "version", "method", "modelType",
                           "license", "metadataOutputVersion", "outputSchema", "stateSchema", "defaultFunctionName",
                           "specificationVersion", "availability", "isUpdatable", "generatedClassName", "userDefinedMetadata",
                           "storagePrecision", "functions", "mlProgramOperationTypeHistogram", "computePrecision"]
FUNCTION_KEYS = ["name", "inputSchema", "outputSchema", "stateSchema", "storagePrecision", "mlProgramOperationTypeHistogram",
                 "computePrecision"]

# Deployment availability by specification version.
AVAILABILITY = {
    1: {"macOS": "10.13", "tvOS": "11.0", "visionOS": "1.0", "watchOS": "4.0", "iOS": "11.0", "macCatalyst": "11.0"},
    2: {"macOS": "10.13.2", "tvOS": "11.2", "visionOS": "1.0", "watchOS": "4.2", "iOS": "11.2", "macCatalyst": "11.2"},
    3: {"macOS": "10.14", "tvOS": "12.0", "visionOS": "1.0", "watchOS": "5.0", "iOS": "12.0", "macCatalyst": "12.0"},
    4: {"macOS": "10.15", "tvOS": "13.0", "visionOS": "1.0", "watchOS": "6.0", "iOS": "13.0", "macCatalyst": "13.0"},
    5: {"macOS": "11.0", "tvOS": "14.0", "visionOS": "1.0", "watchOS": "7.0", "iOS": "14.0", "macCatalyst": "14.0"},
    6: {"macOS": "12.0", "tvOS": "15.0", "visionOS": "1.0", "watchOS": "8.0", "iOS": "15.0", "macCatalyst": "15.0"},
    7: {"macOS": "13.0", "tvOS": "16.0", "visionOS": "1.0", "watchOS": "9.0", "iOS": "16.0", "macCatalyst": "16.0"},
    8: {"macOS": "14.0", "tvOS": "17.0", "visionOS": "1.0", "watchOS": "10.0", "iOS": "17.0", "macCatalyst": "17.0"},
    9: {"macOS": "15.0", "tvOS": "18.0", "visionOS": "2.0", "watchOS": "11.0", "iOS": "18.0", "macCatalyst": "18.0"},
    10: {"macOS": "16.0", "tvOS": "19.0", "visionOS": "3.0", "watchOS": "12.0", "iOS": "19.0", "macCatalyst": "19.0"},
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
        # Without a size range, 0 to 0; an upper bound of -1 is unbounded.
        size = q.msg(101) or Message(b"")
        low, high = size.int(1), _signed(size.int(2))
        if low >= 1 << 63 or high < -1 or 0 <= high < low:
            unestablished("a sequence's size range out of order")
        bounds = str(low) if low == high else "%d..." % low if high == -1 else "%d...%d" % (low, high)
        pairs += [("type", "Sequence"), ("valueType", value), ("sizeRange", "[%d, %d]" % (low, high)),
                  ("formattedType", "Sequence (%s %s)" % (value, bounds))]
    else:
        unestablished("a feature type other than a multi-array, image, state, Int64, String or dictionary")
    if t.int(1000) and not t.has(5):
        # An optional feature's type ends in "?".
        pairs = [(k, v + "?" if k == "formattedType" else v) for k, v in pairs]
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
    if description.has(50) and not model.has(404):
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
        model_type, labels = pipeline_structure(model)
    elif tree_kind(model) is not None:
        kind = tree_kind(model)
        model_type = "MLModelType_" + TREE_TYPES[kind]
        labels = model_class_labels(model.msg(kind)) if kind == 402 else None
    elif model.has(404):
        model_type = "MLModelType_kNearestNeighborsClassifier"
        labels = knn_labels(model)
    elif model.has(501):
        model_type, labels = "MLModelType_itemSimilarityRecommender", None
    else:
        kind = SPEC_ARCHIVED.get(next((n for n, _, _ in model.fields if n in SPEC_ARCHIVED), None))
        if kind is None:
            unestablished("this model type's metadata")
        model_type = kind
        labels = spec_labels(model, SPEC_TYPE_FIELD[kind])
    updatable = bool(model.int(10))
    if updatable and not (model.has(404) and spec_version >= 4):
        unestablished("an updatable model other than a k-nearest neighbors classifier")
    vision = vision_model(model)
    pairs = [("metadataOutputVersion", "3.0")]
    for key, number in (("shortDescription", 1), ("author", 3), ("license", 4), ("version", 2)):
        if meta.has(number) and meta.str(number):
            pairs.append((key, meta.str(number)))
    pairs += [
        ("outputSchema", [feature(f) for f in outputs]),
        ("modelParameters", model_parameters(model)),
        ("specificationVersion", spec_version),
    ]
    if program is not None:
        pairs += [("mlProgramOperationTypeHistogram", CFDict(histogram)), ("computePrecision", precision)]
    pairs += [
        ("isUpdatable", "1" if updatable else "0"),
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
    if updatable:
        # Its training inputs, and no update parameters.
        pairs += [("updateParameters", []), ("trainingInputSchema", [feature(f) for f in description.msgs(50)])]
    if model.has(555):
        pairs.append(("customModels", [CFDict([("className", model.msg(555).str(10)), ("description", model.msg(555).str(40))])]))
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
    return "[\n  " + ns_json(ordered(pairs, MULTI_FUNCTION_TOP_KEYS if program is not None and description.has(20) else TOP_KEYS), 1) + "\n]"


def model_parameters(model, scope=""):
    """metadata.json's model parameters: a k-nearest neighbors classifier's
    number of neighbors, a linked model's file name and search path; a
    pipeline's parts', each scoped by its name (model<N> unless named),
    joined with a "." to the names of the pipelines it's in."""
    if model.has(404):
        out = [int64_parameter(model.msg(404).msg(3), "numberOfNeighbors", "Number of neighbors to use for prediction")[0]]
    elif model.has(556):
        out = linked_parameters(model.msg(556))
    elif pipeline_parts(model) is not None:
        kind, parts = pipeline_parts(model)
        names = [n.decode("utf-8") for n in (model.msg(kind) if kind == 202 else model.msg(kind).msg(1)).all(2)]
        out = []
        for i, (_, part) in enumerate(parts):
            out += model_parameters(Message(part), (scope + "." if scope else "") + (names[i] if names else "model%d" % i))
        return out
    else:
        return []
    for p in out:
        p.pairs = [(k, scope if k == "scope" else v) for k, v in p.pairs]
    return out


def linked_parameters(linked):
    if [n for n, _, _ in linked.fields if n != 1]:
        unestablished("a linked model of another kind")
    f = linked.msg(1) or unestablished("a linked model without its file")
    if [n for n, _, _ in f.fields if n not in (1, 2)]:
        unestablished("a linked model file field it doesn't know")
    out = []
    for number, name, description in ((1, "linkedModelFileName", "File name of linked model"),
                                      (2, "linkedModelSearchPath", "Search path used to resolve model file name")):
        # One not given is empty.
        p = f.msg(number) or Message(b"")
        if [n for n, _, _ in p.fields if n != 1]:
            unestablished("a string parameter of another form")
        out.append(ordered([("name", name), ("shortDescription", description), ("dataType", "String"),
                            ("defaultValue", p.str(1)), ("scope", "")], MODEL_PARAMETER_KEYS))
    return out


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


def analytics_sections(sections):
    """analytics/coremldata.bin's sections: [(name, [(key, value)])]."""
    return b"".join(_s(name) + _u64(len(entries)) + b"".join(_s(k) + _s(v) for k, v in entries) for name, entries in sections)


def program_files(model):
    """An ML Program's own files: model.mil, coremldata.bin and its analytics."""
    return {
        "model.mil": model_mil(model).encode("utf-8"),
        "coremldata.bin": coremldata(model),
        "analytics/coremldata.bin": analytics_sections([("NeuralNetworkModelDetails", [("containsCustomLayer", "0"), ("modelDimension", "0")])]),
    }


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
    556: "linked model", 560: "class confidence thresholding", 600: "one-hot encoder", 601: "imputer",
    602: "feature vectorizer", 603: "dictionary vectorizer", 604: "scaler", 606: "categorical mapping", 607: "normalizer",
    609: "array feature extractor", 610: "non-maximum suppression", 900: "identity", 3000: "serialized model",
    2000: "text classifier", 2001: "word tagger",
    2002: "vision feature print", 2003: "sound analysis preprocessing", 2004: "gazetteer", 2005: "word embedding",
    2006: "audio feature print",
}


# Models coremlc compiles to their own specification behind a header: their
# Model field and the name metadata.json gives their type.
SPEC_ARCHIVED = {2000: "MLModelType_textClassifier", 2001: "MLModelType_wordTagger",
                 301: "MLModelType_supportVectorRegressor", 401: "MLModelType_supportVectorClassifier",
                 300: "MLModelType_glmRegressor", 400: "MLModelType_glmClassifier", 304: "MLModelType_bayesianProbitRegressor",
                 555: "MLModelType_customModel", 556: "MLModelType_linkedModel", 560: "MLModelType_classConfidenceThresholding",
                 600: "MLModelType_oneHotEncoder", 601: "MLModelType_imputer", 602: "MLModelType_featureVectorizer",
                 603: "MLModelType_dictVectorizer", 604: "MLModelType_scaler", 606: "MLModelType_categoricalMapping",
                 607: "MLModelType_normalizer", 609: "MLModelType_arrayFeatureExtractor", 610: "MLModelType_nonMaximumSuppression",
                 900: "MLModelType_identity", 2003: "MLModelType_soundAnalysisPreprocessing", 2004: "MLModelType_gazetteer",
                 2005: "MLModelType_wordEmbedding", 2006: "MLModelType_audioFeaturePrint", 2002: "MLModelType_visionFeaturePrint"}
SPEC_TYPE_FIELD = {v: k for k, v in SPEC_ARCHIVED.items()}
# Their specification versions established, where they are limited.
SPEC_VERSIONS = {2000: (3,), 2001: (3,)}


def spec_labels(model, kind):
    """The class labels metadata.json gives a model compiled to its own
    specification, or None."""
    m = model.msg(kind)
    if kind in (2000, 2001, 2004):
        return [v.decode() for v in m.msg(200).all(1)] if m.has(200) else unestablished("a text model without labels")
    if kind == 402:
        return model_class_labels(m)
    if kind in (400, 401, 610):
        # Each label once, in the order first given.
        labels = model_class_labels(m)
        return None if labels is None else [x for k, x in enumerate(labels) if x not in labels[:k]]
    return None


def spec_bytes(model_bytes):
    """A specification as coremlc copies it for a model on its own: a
    description without metadata gets an empty one (a pipeline's parts are
    copied as they are)."""
    out = b""
    for number, wire, value, raw in _wire_fields(model_bytes):
        if number == 2 and not Message(value).has(100):
            value += b"\xa2\x06\x00"
            raw = b"\x12" + _enc_varint(len(value)) + value
        out += raw
    return out


def spec_files(model_bytes):
    """A model compiled to its specification: coremldata.bin is a header and
    the model's own bytes."""
    model = Message(model_bytes)
    kind = next(n for n, _, _ in model.fields if n in SPEC_ARCHIVED)
    spec_version = model.int(1)
    if spec_version not in SPEC_VERSIONS.get(kind, AVAILABILITY):
        unestablished("specification version %d of a %s" % (spec_version, MODEL_TYPES[kind]))
    header = _u32(kind) + _u32(1) + b"\0" * 20 + _u64(4) + b"spec" + _u32(spec_version) + b"\0" * 28
    return {"coremldata.bin": header + spec_bytes(model_bytes)}


PIPELINE_TYPES = {200: "pipelineClassifier", 201: "pipelineRegressor", 202: "pipeline"}
# Tree ensembles, compiled to their own packed form.
TREE_TYPES = {302: "treeEnsembleRegressor", 402: "treeEnsembleClassifier"}
# Pipeline parts, by Model field, and the names metadata.json gives them:
# those compiled to a header and their own specification, and trees.
PART_TYPES = dict([(k, v[len("MLModelType_"):]) for k, v in SPEC_ARCHIVED.items()] + list(TREE_TYPES.items())
                  + [(404, "kNearestNeighborsClassifier"), (501, "itemSimilarityRecommender")])


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
    # A pipeline classifier or regressor holds a pipeline.
    pipeline = model.msg(kind) if kind == 202 else model.msg(kind).msg(1) or Message(b"")
    for part in pipeline.all(1):
        fields = [n for n, _, _ in Message(part).fields if n not in (1, 2, 10)]
        if len(fields) != 1 or fields[0] not in PART_TYPES and fields[0] not in PIPELINE_TYPES:
            unestablished("a pipeline with a %s" % (MODEL_TYPES.get(fields[0], "type %d" % fields[0]) if fields else "typeless model"))
        parts.append((fields[0], part))
    if not parts:
        unestablished("a pipeline without models")
    return kind, parts


def pipeline_structure(model):
    """(metadata.json's model type, the class labels) of a pipeline: each
    part named, a pipeline with its own parts. The labels are those of the
    classifier giving the pipeline's predicted feature, or else of its only
    classifier, nested ones included; with several, none."""
    model_type, labelled = _pipeline_structure(model)
    predicted = (model.msg(2) or Message(b"")).str(11)
    matching = [labels for name, labels in labelled if predicted and name == predicted]
    if matching:
        return model_type, matching[-1]
    return model_type, labelled[0][1] if len(labelled) == 1 else None


def _pipeline_structure(model):
    """(model type, [(predicted feature, labels)] of its classifiers)."""
    kind, parts = pipeline_parts(model)
    structure, labelled = [], []
    for pk, part in parts:
        part = Message(part)
        if pk in PIPELINE_TYPES:
            name, inner = _pipeline_structure(part)
            labelled += inner
        else:
            name = CFDict([("name", "MLModelType_" + PART_TYPES[pk])])
            labels = knn_labels(part) if pk == 404 else spec_labels(part, pk)
            if labels is not None:
                labelled.append(((part.msg(2) or Message(b"")).str(11), labels))
        structure.append(name)
    name = "MLModelType_" + ("imageClassifier" if image_classifier(kind, parts) else PIPELINE_TYPES[kind])
    return CFDict([("name", name), ("structure", structure)]), labelled


def vision_model(model):
    """Whether the model uses Vision's feature prints, which watchOS lacks:
    is one, or a pipeline with one."""
    if model.has(2002):
        return True
    parts = pipeline_parts(model)
    return parts is not None and any(vision_model(Message(part)) for _, part in parts[1])


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

def _canon_description(desc, part=False):
    """Re-serialize ModelDescription as Xcode does (MD1, MD2, MD3): fields in
    field-number order, empty proto3 strings dropped, metadata (100) always
    present but in a pipeline's part. Nested messages are copied; they must
    already be canonical."""
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
    if not part and not any(f == 100 for f, _ in fl):
        fl.append((100, b'\xa2\x06\x00'))
    fl.sort(key=lambda x: x[0])            # stable
    return b''.join(r for _, r in fl)


def _encode_coremldata(m, npd, part=False):
    desc = _canon_description(m['desc'], part)
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


def encode_tree_part(part_model_bytes: bytes, part=False) -> dict:
    m = _parse_model(part_model_bytes)
    nodes, npd, base = _parse_params(m['params'] or b'')
    return {
        'coremldata.bin': _encode_coremldata(m, npd, part),
        '_B0000.DAT': _encode_dat(nodes, npd, base, m['tf']),
    }


def pipeline_archive(model_bytes, nested=False):
    """(files, tag) of a pipeline: its archive, each part compiled to
    model<N>/ (a pipeline as a pipeline, with analytics of its own), and the
    names given to them; the tag names each part's archive. A pipeline in a
    pipeline keeps its description as it is."""
    model = Message(model_bytes)
    kind, parts = pipeline_parts(model)
    spec_version = model.int(1)
    if spec_version not in AVAILABILITY:
        unestablished("a pipeline of specification %d" % spec_version)
    files, tags = {}, []
    for i, (pk, part) in enumerate(parts):
        if pk in PIPELINE_TYPES:
            inner, inner_tag = pipeline_archive(part, True)
            for rel, data in inner.items():
                files["model%d/%s" % (i, rel)] = data
            tags.append("model%dv1.0.1%s" % (i, inner_tag))
        elif pk in TREE_TYPES:
            for rel, data in encode_tree_part(part, True).items():
                files["model%d/%s" % (i, rel)] = data
            tags.append("model%dv1.0.0generic" % i)
        elif pk in OWN_ARCHIVES:
            for rel, data in OWN_ARCHIVES[pk](Message(part), True).items():
                files["model%d/%s" % (i, rel)] = data
            tags.append("model%dv1.0.0%s" % (i, "notreesyet" if pk == 404 else "generic"))
        else:
            part_version = Message(part).int(1)
            files["model%d/coremldata.bin" % i] = _u32(pk) + _u32(1) + b"\0" * 20 + _u64(4) + b"spec" + _u32(part_version) + b"\0" * 28 + part
            tags.append("model%dv1.0.0spec" % i)
    tag = ("generic." + ".".join(tags)).encode()
    description = model.bytes(2) if nested else canonical_description(model)
    files["coremldata.bin"] = (_u32(kind) + _u32(1) + b"\0" * 12 + _u32(1) + _u32(0) + _u64(len(tag)) + tag + _u32(spec_version) + b"\0" * 28
                               + _u64(len(description)) + description + _u64(len(parts)) + _u64(kind))
    files["analytics/coremldata.bin"] = _s("PipelineModelDetails") + _u64(1) + _s("modelDimension") + _s(str(len(parts)))
    names = [n.decode("utf-8") for n in (model.msg(kind) if kind == 202 else model.msg(kind).msg(1) or Message(b"")).all(2)]
    if names:
        files["modelNames/coremldata.bin"] = _u64(len(names)) + b"".join(_s(n) for n in names)
    return files, tag.decode()


def _s(text):
    """A length-prefixed string."""
    raw = text.encode()
    return _u64(len(raw)) + raw


def pipeline_files(model_bytes):
    """A pipeline of models compiled each to model<N>/, behind an archive
    naming them."""
    return pipeline_archive(model_bytes)[0]


def tree_files(model_bytes):
    """A tree ensemble on its own: its packed trees, behind the archive a
    pipeline's tree part has."""
    if Message(model_bytes).int(1) not in AVAILABILITY:
        unestablished("a tree ensemble of specification %d" % Message(model_bytes).int(1))
    return dict(encode_tree_part(model_bytes))


# --- Item similarity recommenders ---------------------------------------------------
#
# An item similarity recommender (Model field 501) compiles to the `generic`
# archive and a `_B0000.DAT` holding its similarities as a sparse matrix:
# rows by item, columns by similar item, in the packed arrays tree
# ensembles use.


def item_similarities(model):
    """(item count, {row: {column: score}}, {item: score adjustment}, integer
    ids, string ids, the feature names) of an item similarity recommender,
    checked as Xcode checks it."""
    r = model.msg(501)
    if any(n not in (1, 2, 3, 10, 11, 12, 13, 20, 21) for n, _, _ in r.fields):
        unestablished("an item similarity recommender field it doesn't know")
    rows, adjustments, count = {}, {}, 0
    for item in r.msgs(1):
        if any(n not in (1, 2, 3) for n, _, _ in item.fields):
            unestablished("an item similarity field it doesn't know")
        i = item.int(1)
        if i in rows:
            unestablished("an item listed twice")
        rows[i] = {}
        count = max(count, i + 1)
        for c in item.msgs(2):
            if any(n not in (1, 2) for n, _, _ in c.fields):
                unestablished("a connected item field it doesn't know")
            j = c.int(1)
            score = struct.unpack("<d", c.bytes(2, b"\0" * 8))[0]
            if j in rows[i]:
                unestablished("an item's similar item listed twice")
            if score != score or score in (float("inf"), float("-inf")):
                unestablished("a similarity score that isn't finite")
            rows[i][j] = score
            count = max(count, j + 1)
        if item.has(3):
            adjustments[i] = struct.unpack("<d", item.bytes(3))[0]
    if not rows:
        # Xcode writes uninitialized bytes for a recommender without items.
        unestablished("an item similarity recommender without items")
    if r.has(2) and r.has(3):
        unestablished("both integer and string item ids")
    strings = [v.decode("utf-8") for v in r.msg(2).all(1)] if r.has(2) else []
    ints = [_signed(v) for v in _ints(r.msg(3).all(1))] if r.has(3) else []
    if len(set(strings)) != len(strings) or len(set(ints)) != len(ints):
        unestablished("repeated item ids")
    # Ids must cover every item: integer ids each one, string ids all but
    # the last (as Xcode checks them).
    if ints and len(ints) < count or strings and len(strings) < count - 1:
        unestablished("fewer item ids than items")
    names = [r.str(n) for n in (10, 11, 12, 13, 20, 21)]
    return count, rows, adjustments, ints, strings, names


def item_similarity_archive(model, part=False):
    """coremldata.bin and _B0000.DAT of an item similarity recommender (a
    pipeline's part keeps its description as it is)."""
    count, rows, adjustments, ints, strings, names = item_similarities(model)
    # Row offsets, as Xcode writes them: an item not listed before a listed
    # one repeats the offset reached, one short of a row each.
    offsets, columns, values = [0], [], []
    for i in sorted(rows):
        while len(offsets) < i:
            offsets.append(len(values))
        previous = None
        # Scores of zero are left out; each row's columns are sorted and
        # written as the gap from the one before (the first as it is).
        for j in sorted(rows[i]):
            if rows[i][j] == 0:
                continue
            columns.append(j if previous is None else j - previous - 1)
            values.append(rows[i][j])
            previous = j
        offsets.append(len(values))
    dat = struct.pack("<II", 0, count) + _u64(0)
    dat += _packed_array(offsets, _bl1(max(offsets)))
    dat += _packed_array(columns, _bl1(max(columns)) if columns else 1)
    dat += _u64(len(values)) + b"".join(struct.pack("<d", v) for v in values)
    if adjustments:
        dat += _u64(count) + b"".join(struct.pack("<d", adjustments.get(i, 0.0)) for i in range(count))
    else:
        dat += _u64(0)
    description = _canon_description(model.bytes(2), part)
    head = (_u32(501) + _u32(1) + b"\0" * 20 + _s("generic") + _u32(model.int(1)) + b"\0" * 28 + _u64(len(description)) + description
            + _u64(0) + _s("_B0000.DAT") + _u64(len(ints) or len(strings) or count)
            + _u64(len(ints)) + b"".join(struct.pack("<q", x) for x in ints)
            + _u64(len(strings)) + b"".join(_s(x) for x in strings) + b"".join(_s(n) for n in names))
    return {"coremldata.bin": head, "_B0000.DAT": dat}


# --- k-nearest neighbors ----------------------------------------------------------
#
# A k-nearest neighbors classifier (Model field 404) compiles to its own
# archive (tag `notreesyet`) holding the samples and labels, and its number
# of neighbors as a model parameter under modelParameters/.

# A model parameter's keys in the order coremlc inserts them.
MODEL_PARAMETER_KEYS = ["name", "shortDescription", "dataType", "defaultValue", "allowedRange", "allowedSet", "scope"]


def _floats(values):
    """A repeated float field's values, packed or not, as float32 bytes."""
    out = b""
    for r in values:
        out += r
    if len(out) % 4:
        unestablished("a malformed float vector")
    return out


def int64_parameter(p, name, description):
    """(metadata.json's description, the parameter's bytes) of an
    Int64Parameter: its default must be positive and allowed, as Xcode
    requires."""
    if p is None:
        unestablished("an Int64 parameter without a default")
    default = _signed(p.int(1))
    pairs = [("name", name), ("shortDescription", description), ("dataType", "Int64"), ("defaultValue", str(default)), ("scope", "")]
    if default <= 0:
        unestablished("a parameter whose default isn't positive")
    if p.has(10) and p.has(11) or [n for n, _, _ in p.fields if n not in (1, 10, 11)]:
        unestablished("an Int64 parameter of another form")
    if p.has(10):
        r = p.msg(10)
        low, high = _signed(r.int(1)), _signed(r.int(2))
        if not low <= default <= high:
            unestablished("a parameter's default out of its range")
        pairs.append(("allowedRange", "[%d, %d]" % (low, high)))
    if p.has(11):
        values = [_signed(v) for v in _ints(p.msg(11).all(1))]
        if default not in values:
            unestablished("a parameter's default outside its set")
        pairs.append(("allowedSet", "[%s]" % ", ".join(map(str, values))))
    return ordered(pairs, MODEL_PARAMETER_KEYS), p.data


def knn_parts(model):
    """The facts of a k-nearest neighbors classifier, checked as Xcode checks
    them: (dimensions, samples' float32 bytes, sample count, labels as
    given, default label, weighting, index, leaf size, neighbors parameter)."""
    k = model.msg(404)
    known = {1, 3, 100, 101, 110, 111, 200, 210}
    if any(n not in known for n, _, _ in k.fields):
        unestablished("a k-nearest neighbors field it doesn't know")
    index = k.msg(1) or unestablished("a k-nearest neighbors classifier without an index")
    if any(n not in (1, 2, 100, 110, 200) for n, _, _ in index.fields):
        unestablished("a nearest neighbors index field it doesn't know")
    dims = index.int(1)
    samples = [_floats(v.all(1)) for v in index.msgs(2)]
    if any(len(v) != 4 * dims for v in samples):
        unestablished("a sample of another length than the dimensions")
    if index.has(100) == index.has(110):
        unestablished("a nearest neighbors index of no or several types")
    if not index.has(200):
        unestablished("a nearest neighbors index without its distance")
    leaf = None
    if index.has(110):
        leaf = index.msg(110).int(1)
        if leaf <= 0 or leaf >= 1 << 31:
            unestablished("a k-d tree's leaf size that isn't positive")
    if k.has(200) == k.has(210):
        unestablished("a k-nearest neighbors classifier with no or several weightings")
    if k.has(100) and k.has(101) or k.has(110) and k.has(111):
        unestablished("several kinds of labels")
    labels = model_class_labels(k)
    default = k.str(110) if k.has(110) else _signed(k.int(111)) if k.has(111) else None
    if not labels:
        if default is None:
            unestablished("a k-nearest neighbors classifier without labels or a default")
        if samples:
            unestablished("samples without labels")
    elif len(labels) != len(samples):
        unestablished("a k-nearest neighbors classifier with as many labels as samples")
    if labels is not None and default is not None and k.has(110) != k.has(100):
        unestablished("a default label of another type than the labels")
    return dims, b"".join(samples), labels, k.has(101) or labels is None and k.has(111), default, 1 if k.has(210) else 0, leaf, k.msg(3)


def knn_archive(model, part=False):
    """coremldata.bin and modelParameters/ of a k-nearest neighbors
    classifier (a pipeline's part keeps its description as it is)."""
    dims, samples, labels, ints, default, weighting, leaf, neighbors = knn_parts(model)
    sv = model.int(1)
    description = _canon_description(model.bytes(2), part)
    out = (_u32(404) + _u32(1) + b"\0" * 20 + _s("notreesyet") + _u32(sv) + b"\0" * 28 + _u64(len(description)) + description
           + _u32(dims) + _u64(len(samples) // 4) + samples)
    # Labels not given at all are written as none; given empty, as empty.
    if labels is None:
        out += _u64(0)
    elif ints:
        out += _u64(101) + _u64(len(labels)) + b"".join(struct.pack("<q", x) for x in labels)
    else:
        out += _u64(100) + _u64(len(labels)) + b"".join(_s(x) for x in labels)
    if default is None:
        out += _u64(0)
    elif isinstance(default, str):
        out += _u64(110) + _s(default)
    else:
        out += _u64(111) + struct.pack("<q", default)
    out += _u32(weighting) + _u32(0 if leaf is None else 1) + (b"" if leaf is None else _u32(leaf))
    _, parameter = int64_parameter(neighbors, "numberOfNeighbors", "Number of neighbors to use for prediction")
    return {"coremldata.bin": out, "modelParameters/coremldata.bin": _u64(0),
            "modelParameters/numberOfNeighbors/coremldata.bin": parameter}


def knn_labels(model):
    """metadata.json's labels of a k-nearest neighbors classifier: each label
    once, in the order first given."""
    labels = knn_parts(model)[2]
    if labels is None:
        return None
    out = []
    for x in labels:
        if x not in out:
            out.append(x)
    return out


# --- Checks of .mlmodel files -----------------------------------------------------
#
# Xcode checks a model as it reads it, for compiling and generating alike,
# and refuses one that fails. Each type's checks below are those seen to
# refuse a model, and a model of a form not seen accepted is refused too.

def feature_kind(f):
    """A feature's type: "double", "int64", "string", "array", "image",
    "dict", "seq" or "state"."""
    t = f.msg(3)
    kinds = {1: "int64", 2: "double", 3: "string", 4: "image", 5: "array", 6: "dict", 7: "seq", 8: "state"}
    present = [kinds[n] for n, _, _ in (t.fields if t else []) if n in kinds]
    if t is None or len(present) != 1 or any(n not in kinds and n != 1000 for n, _, _ in t.fields):
        unestablished("a feature without a single known type")
    return present[0]


def key_kind(f):
    """A dictionary's key type or a sequence's element type."""
    t = f.msg(3)
    inner = t.msg(6) if t.has(6) else t.msg(7)
    return "int64" if inner.has(1) else "string" if inner.has(2 if t.has(6) else 3) else unestablished("a dictionary or sequence of another type")


def array_type(f):
    return ARRAY_TYPES.get(f.msg(3).msg(5).int(2)) or unestablished("array data type %d" % f.msg(3).msg(5).int(2))


def check_description(model, optional_inputs=False):
    """The checks of every model's features."""
    d = model.msg(2) or Message(b"")
    sv = model.int(1)
    inputs, outputs = d.msgs(1), d.msgs(10)
    if not inputs or not outputs:
        unestablished("a model without inputs or outputs")
    if any(n not in (1, 10, 11, 12, 100) + ((50,) if model.has(404) else ()) for n, _, _ in d.fields):
        unestablished("a model description field it doesn't know")
    # Training inputs, as coremltools makes them: features the model has.
    if any(t not in [f.data for f in inputs + outputs] for t in d.all(50)):
        unestablished("a training input that isn't one of the model's features")
    for f in inputs + outputs:
        if not f.str(1):
            unestablished("a feature without a name")
        kind = feature_kind(f)
        if kind == "state":
            unestablished("a state outside an ML Program")
        if kind == "array":
            a = f.msg(3).msg(5)
            if array_type(f) == "Float16" and sv < 7:
                unestablished("a Float16 array before specification 7")
            if f in inputs and not _ints(a.all(1)):
                unestablished("an input array without a shape")
            if a.has(41) or a.has(51) or a.has(61):
                unestablished("a default value outside an ML Program")
            shape, ranges, shapes = flexibility(f)
            if ranges and any(not lo <= x <= hi for x, (lo, hi) in zip(shape, ranges)):
                unestablished("a default shape out of its range")
            if shapes is not None and shape not in shapes:
                unestablished("a default shape not among its enumerated shapes")
        if kind == "image":
            im = f.msg(3).msg(4)
            if im.int(3) == 40 and sv < 7:
                unestablished("a Grayscale16Half image before specification 7")
            shape, ranges, shapes = flexibility(f)
            if ranges and any(not lo <= x <= hi for x, (lo, hi) in zip(shape, ranges)):
                unestablished("a default size out of its range")
            if shapes is not None and shape not in shapes:
                unestablished("a default size not among its enumerated sizes")
        if kind == "seq" and sv < 3:
            unestablished("a sequence before specification 3")
        if kind in ("dict", "seq"):
            key_kind(f)
    if any(f.msg(3).int(1000) for f in outputs):
        unestablished("an optional output")
    if not optional_inputs and any(f.msg(3).int(1000) for f in inputs):
        unestablished("an optional input to a model that can't have them")
    return inputs, outputs


def _named(features, name):
    out = [f for f in features if f.str(1) == name]
    if not name or not out:
        unestablished("a feature named %r not in the description" % name)
    return out[0]


def _one(inputs, outputs):
    if len(inputs) != 1 or len(outputs) != 1:
        unestablished("more than one input or output to a model that takes one")
    return inputs[0], outputs[0]


def _kinds(features, allowed, what):
    for f in features:
        if feature_kind(f) not in allowed:
            unestablished("%s of type %s" % (what, feature_kind(f)))


def _fields(m, known, what):
    if any(n not in known for n, _, _ in m.fields):
        unestablished("a %s field it doesn't know" % what)


def _doubles(m, number):
    """A repeated double field's values, packed or not."""
    out = []
    for n, wire, v in m.fields:
        if n == number:
            out += [struct.unpack("<d", v[i:i + 8])[0] for i in range(0, len(v), 8)] if wire == 2 else [struct.unpack("<d", v)[0]]
    return out


def _predicted(model, outputs, classifier):
    d = model.msg(2)
    _named(outputs, d.str(11))
    if d.has(12) and d.str(12):
        _named(outputs, d.str(12))
        if not classifier:
            unestablished("probabilities from a model that isn't a classifier")


def _labels(model, m, outputs):
    """A classifier's labels, required, and its predicted feature of their
    type."""
    labels = model_class_labels(m)
    if not labels:
        unestablished("a classifier without labels")
    if m.has(100) and m.has(101):
        unestablished("both string and integer labels")
    predicted = _named(outputs, model.msg(2).str(11))
    if feature_kind(predicted) != ("string" if m.has(100) else "int64"):
        unestablished("a predicted label of another type than the labels")
    return labels


def check_glm_regressor(model, m, inputs, outputs):
    _fields(m, (1, 2, 3), "GLM regressor")
    _predicted(model, outputs, False)
    _kinds(inputs, ("double", "int64", "array"), "a GLM regressor input")
    _kinds(outputs, ("double",), "a GLM regressor output")
    if not m.msgs(1) or len(m.msgs(1)) != len(_doubles(m, 2)) or m.int(3) > 2:
        unestablished("a GLM regressor's weights, offsets or transform")


def check_glm_classifier(model, m, inputs, outputs):
    _fields(m, (1, 2, 3, 4, 100, 101), "GLM classifier")
    _predicted(model, outputs, True)
    _kinds(inputs, ("double", "int64", "array"), "a GLM classifier input")
    labels = _labels(model, m, outputs)
    weights = len(m.msgs(1))
    if weights != len(_doubles(m, 2)) or m.int(3) > 1 or m.int(4) > 1:
        unestablished("a GLM classifier's weights, offsets, transform or encoding")
    if m.int(4) == 0 and weights != len(labels) - 1:
        unestablished("reference-class weights other than one fewer than the classes")
    if m.int(4) == 1 and weights != (1 if len(labels) == 2 else len(labels)):
        unestablished("one-vs-rest weights other than one per class (one for two classes)")
    if m.int(3) == 1 and len(labels) != 2:
        unestablished("a probit GLM classifier of more than two classes")


def _support_vectors(m, number_dense, number_sparse):
    """The support vectors' count, dense or sparse; one kind required."""
    if m.has(number_dense) == m.has(number_sparse):
        unestablished("support vectors neither or both dense and sparse")
    if m.has(number_dense):
        return len(m.msg(number_dense).msgs(1))
    for v in m.msg(number_sparse).msgs(1):
        for node in v.msgs(1):
            _fields(node, (1, 2), "sparse node")
    return len(m.msg(number_sparse).msgs(1))


def _kernel(k):
    if k is None or len([n for n, _, _ in k.fields]) != 1 or k.fields[0][0] not in (1, 2, 3, 4):
        unestablished("a support vector machine without a single known kernel")
    _fields(k.msg(k.fields[0][0]), (1, 2, 3), "kernel")


def check_svr(model, m, inputs, outputs):
    _fields(m, (1, 2, 3, 4, 5), "support vector regressor")
    _predicted(model, outputs, False)
    _kinds(inputs, ("double", "int64", "array"), "a support vector regressor input")
    _kinds(outputs, ("double", "array"), "a support vector regressor output")
    _kernel(m.msg(1))
    count = _support_vectors(m, 3, 2)
    if len(_doubles(m.msg(4) or Message(b""), 1)) != count:
        unestablished("as many coefficients as support vectors")


def check_svc(model, m, inputs, outputs):
    _fields(m, (1, 2, 3, 4, 5, 6, 7, 8, 100, 101), "support vector classifier")
    _predicted(model, outputs, True)
    _kinds(inputs, ("double", "int64", "array"), "a support vector classifier input")
    labels = _labels(model, m, outputs)
    _kernel(m.msg(1))
    count = _support_vectors(m, 4, 3)
    per_class = _ints(m.all(2))
    if len(per_class) != len(labels) or sum(per_class) != count:
        unestablished("support vectors per class that don't add up")
    coefficients = [_doubles(c, 1) for c in m.msgs(5)]
    if len(coefficients) != len(labels) - 1 or any(len(c) != count for c in coefficients):
        unestablished("coefficients other than one fewer than the classes, each one per support vector")
    if len(_doubles(m, 7)) != len(_doubles(m, 8)):
        unestablished("probA and probB of different sizes")


def check_probit(model, m, inputs, outputs):
    _fields(m, (1, 2, 3, 10, 11, 12, 13, 20, 21, 22, 23), "Bayesian probit regressor")
    for f in inputs:
        if feature_kind(f) != "array" or array_type(f) != "Int32":
            unestablished("a Bayesian probit regressor input other than an Int32 array")
    _kinds(outputs, ("double",), "a Bayesian probit regressor output")


def check_nms(model, m, inputs, outputs):
    _fields(m, (1, 100, 101, 110, 111, 200, 201, 202, 203, 210, 211), "non-maximum suppression")
    for number in (200, 201):
        f = _named(inputs, m.str(number))
        if feature_kind(f) != "array" or array_type(f) != "Double":
            unestablished("non-maximum suppression of other than Double arrays")
    for number in (202, 203):
        if m.str(number) and feature_kind(_named(inputs, m.str(number))) != "double":
            unestablished("a threshold input other than a double")
    names = [m.str(210), m.str(211)]
    if sorted(f.str(1) for f in outputs) != sorted(names):
        unestablished("non-maximum suppression outputs other than those it names")
    for f in outputs:
        if feature_kind(f) != "array" or array_type(f) != "Double" or len(_ints(f.msg(3).msg(5).all(1))) != 2:
            unestablished("a non-maximum suppression output other than a Double matrix")
    for number in (110, 111):
        value = struct.unpack("<d", m.bytes(number, b"\0" * 8))[0]
        if not 0 <= value <= 1:
            unestablished("a threshold outside 0 to 1")
    labels = model_class_labels(m)
    if labels is not None and len(labels) != _ints(_named(outputs, m.str(210)).msg(3).msg(5).all(1))[1]:
        unestablished("as many labels as the confidence output's classes")
    if m.has(1):
        _fields(m.msg(1), (1,), "pick top")


def check_cct(model, m, inputs, outputs):
    _fields(m, (100,), "class confidence thresholding")
    curves = m.msgs(100)
    if not curves:
        unestablished("class confidence thresholding without curves")
    for c in curves:
        _fields(c, (1, 2, 3, 4), "precision-recall curve")
        lengths = {len(_floats((c.msg(n) or Message(b"")).all(1))) // 4 for n in (1, 2, 3, 4)}
        if len(lengths) != 1 or 0 in lengths:
            unestablished("precision-recall curves of empty or mismatched components")


def check_custom(model, m, inputs, outputs):
    _fields(m, (10, 30, 40), "custom model")
    if not m.str(10):
        unestablished("a custom model without a class name")


def check_linked(model, m, inputs, outputs):
    linked_parameters(m)
    if not m.msg(1).msg(1).str(1):
        unestablished("a linked model without a file name")


def check_one_hot(model, m, inputs, outputs):
    _fields(m, (1, 2, 10, 11), "one-hot encoder")
    i, o = _one(inputs, outputs)
    _kinds([i], ("string", "int64"), "a one-hot encoder input")
    _kinds([o], ("dict",) if m.int(10) else ("array",), "a one-hot encoder output")
    if m.has(1) == m.has(2) or m.int(11) > 1:
        unestablished("a one-hot encoder without its categories")


def check_imputer(model, m, inputs, outputs):
    _fields(m, (1, 2, 3, 4, 11, 12, 13), "imputer")
    i, o = _one(inputs, outputs)
    kind = feature_kind(i)
    imputed = [n for n in (1, 2, 3, 4) if m.has(n)]
    replace = [n for n in (11, 12, 13) if m.has(n)]
    # The imputed value of each input type, and the replaced value.
    expected = {"double": (1, 11), "int64": (2, 12), "string": (3, 13), "array": (4, 11)}.get(kind)
    if expected is None or imputed != [expected[0]] or replace not in ([], [expected[1]]) or feature_kind(o) != kind:
        unestablished("an imputer of these types")
    if kind == "array":
        shape = _ints(i.msg(3).msg(5).all(1))
        if len(shape) != 1 or len(_doubles(m.msg(4), 1)) != shape[0]:
            unestablished("an imputed array of another shape than its input")


def check_feature_vectorizer(model, m, inputs, outputs):
    _fields(m, (1,), "feature vectorizer")
    for c in m.msgs(1):
        _fields(c, (1, 2), "feature vectorizer column")
    _kinds(inputs, ("double", "int64", "array", "dict"), "a feature vectorizer input")
    if len(outputs) != 1:
        unestablished("a feature vectorizer of several outputs")
    _kinds(outputs, ("array",), "a feature vectorizer output")


def check_dict_vectorizer(model, m, inputs, outputs):
    _fields(m, (1, 2), "dictionary vectorizer")
    i, o = _one(inputs, outputs)
    # Xcode accepts only a dictionary output.
    _kinds([i], ("dict",), "a dictionary vectorizer input")
    _kinds([o], ("dict",), "a dictionary vectorizer output")
    if m.has(1) == m.has(2):
        unestablished("a dictionary vectorizer without its index")


def check_scaler(model, m, inputs, outputs):
    _fields(m, (1, 2), "scaler")
    i, o = _one(inputs, outputs)
    kind, out = feature_kind(i), feature_kind(o)
    shift, scale = len(_doubles(m, 1)), len(_doubles(m, 2))
    if kind == "array":
        shape = _ints(i.msg(3).msg(5).all(1))
        if out != "array" or len(shape) != 1 or _ints(o.msg(3).msg(5).all(1)) != shape or shift not in (0, 1, shape[0]) or scale not in (0, 1, shape[0]):
            unestablished("a scaler of these arrays")
    elif (kind, out) not in (("double", "double"), ("int64", "int64"), ("int64", "double")) or shift > 1 or scale > 1:
        unestablished("a scaler of these types")


def check_categorical_mapping(model, m, inputs, outputs):
    _fields(m, (1, 2, 101, 102), "categorical mapping")
    i, o = _one(inputs, outputs)
    if m.has(1) == m.has(2):
        unestablished("a categorical mapping without its map")
    source, target = ("string", "int64") if m.has(1) else ("int64", "string")
    if m.has(101 if m.has(1) else 102):
        unestablished("a default of another type than the mapping's")
    if (feature_kind(i), feature_kind(o)) != (source, target) and not (
            feature_kind(i) == feature_kind(o) == "seq" and key_kind(i) == source and key_kind(o) == target):
        unestablished("a categorical mapping of these types")


def check_normalizer(model, m, inputs, outputs):
    _fields(m, (1,), "normalizer")
    i, o = _one(inputs, outputs)
    _kinds([i, o], ("array",), "a normalizer feature")
    if m.int(1) > 2:
        unestablished("a normalizer of another norm")


def check_array_feature_extractor(model, m, inputs, outputs):
    _fields(m, (1,), "array feature extractor")
    i, o = _one(inputs, outputs)
    _kinds([i], ("array",), "an array feature extractor input")
    indices = _ints(m.all(1))
    if feature_kind(o) in ("double", "int64") and len(indices) != 1 or feature_kind(o) == "array" and not indices:
        unestablished("an array feature extractor of these indices")
    _kinds([o], ("double", "int64", "array"), "an array feature extractor output")


def check_text_model(model, m, inputs, outputs, output_kinds):
    i = _one(inputs, outputs)[0] if model.has(2004) or model.has(2005) else inputs[0]
    _kinds([i], ("string",), "a text model's input")
    if not m.bytes(100):
        unestablished("a text model without its data")
    if model.has(2004) or model.has(2005):
        _fields(m, (1, 10, 100, 200), "text model")
        if m.int(1) < 2:
            unestablished("a text model of revision 1")
        _kinds(outputs, output_kinds, "a text model's output")
    if model.has(2004) and not spec_labels(model, 2004):
        unestablished("a gazetteer without labels")


def check_item_similarity(model, m, inputs, outputs):
    item_similarities(model)
    _kinds([_named(inputs, m.str(10))], ("dict",), "a recommender's items")
    _kinds([_named(outputs, m.str(20))], ("seq",), "a recommender's recommendations")
    for number, kinds in ((11, ("int64",)), (12, ("seq",)), (13, ("seq",))):
        if m.str(number):
            _kinds([_named(inputs, m.str(number))], kinds, "a recommender input")
    if m.str(21):
        _kinds([_named(outputs, m.str(21))], ("dict",), "a recommender's scores")


def check_knn(model, m, inputs, outputs):
    _predicted(model, outputs, True)
    _kinds(inputs, ("array",), "a k-nearest neighbors input")
    _, _, labels, ints, default, _, _, _ = knn_parts(model)
    if feature_kind(_named(outputs, model.msg(2).str(11))) != ("int64" if ints else "string"):
        unestablished("a predicted label of another type than the labels")


def check_vision_feature_print(model, m, inputs, outputs):
    _fields(m, (20, 21), "vision feature print")
    _kinds(inputs, ("image",), "a vision feature print input")
    _kinds(outputs, ("array",), "a vision feature print output")
    if len(inputs) != 1 or m.has(20) == m.has(21):
        unestablished("a vision feature print of no or several kinds or inputs")
    if m.has(20):
        _fields(m.msg(20), (1,), "scene print")
        if m.msg(20).int(1) not in (1, 2) or len(outputs) != 1:
            unestablished("a scene print of another version or output count")
    else:
        _fields(m.msg(21), (1, 100), "object print")
        names = [n.decode("utf-8") for n in m.msg(21).all(100)]
        if m.msg(21).int(1) != 1 or len(names) != 2 or sorted(names) != sorted(f.str(1) for f in outputs):
            unestablished("an object print of another version or outputs")


def check_audio(model, m, inputs, outputs):
    """An audio feature print (sound, version 1) or sound analysis
    preprocessing (VGGish)."""
    _fields(m, (20,), "audio model")
    i, o = _one(inputs, outputs)
    _kinds([i, o], ("array",), "an audio model's feature")
    if not m.has(20) or model.has(2006) and m.msg(20).int(1) != 1 or [n for n, _, _ in m.msg(20).fields if n != 1]:
        unestablished("an audio model of another kind or version")


def _type_bytes(f):
    """A feature's type as compared along a pipeline: whether it's optional
    aside."""
    return b"".join(raw for n, _, _, raw in _wire_fields(f.bytes(3)) if n != 1000)


def check_pipeline(model):
    """Each part's inputs come from the pipeline's or an earlier part's
    outputs, of the same type, and so do the pipeline's outputs; names, if
    given, name each part."""
    kind, parts = pipeline_parts(model)
    inputs, outputs = check_description(model)
    pipeline = model.msg(kind) if kind == 202 else model.msg(kind).msg(1)
    if pipeline.has(2) and len(pipeline.all(2)) != len(parts):
        unestablished("pipeline names not one per model")
    available = {f.str(1): _type_bytes(f) for f in inputs}
    for _, part in parts:
        d = Message(part).msg(2) or Message(b"")
        for f in d.msgs(1):
            if available.get(f.str(1)) != _type_bytes(f):
                unestablished("a pipeline model's input not given before it, or of another type")
        for f in d.msgs(10):
            available[f.str(1)] = _type_bytes(f)
    for f in outputs:
        if available.get(f.str(1)) != _type_bytes(f):
            unestablished("a pipeline output no model gives, or of another type")


# The checks of each type, and whether its inputs may be optional.
CHECKS = {
    300: (check_glm_regressor, False), 400: (check_glm_classifier, False), 301: (check_svr, False), 401: (check_svc, False),
    304: (check_probit, False), 610: (check_nms, False), 560: (check_cct, False), 555: (check_custom, False),
    556: (check_linked, False), 600: (check_one_hot, False), 601: (check_imputer, True), 602: (check_feature_vectorizer, False),
    603: (check_dict_vectorizer, False), 604: (check_scaler, False), 606: (check_categorical_mapping, False),
    607: (check_normalizer, False), 609: (check_array_feature_extractor, False), 900: (lambda *a: None, True),
    2004: (lambda model, m, i, o: check_text_model(model, m, i, o, ("string",)), False),
    2005: (lambda model, m, i, o: check_text_model(model, m, i, o, ("array",)), False),
    2000: (lambda model, m, i, o: check_text_model(model, m, i, o, None), False),
    2001: (lambda model, m, i, o: check_text_model(model, m, i, o, None), False),
    501: (check_item_similarity, True), 404: (check_knn, False), 2002: (check_vision_feature_print, False),
    2003: (check_audio, False), 2006: (check_audio, False),
}


def validate(model):
    """The checks Xcode makes of a .mlmodel as it reads it, before compiling
    or generating: a pipeline's and each of its parts'."""
    kinds = [n for n, _, _ in model.fields if n not in (1, 2, 10)]
    if len(kinds) != 1:
        unestablished("a model of no or several types")
    kind = kinds[0]
    if model.int(10) and kind != 404:
        unestablished("an updatable model other than a k-nearest neighbors classifier")
    if kind in PIPELINE_TYPES:
        check_pipeline(model)
        for _, part in pipeline_parts(model)[1]:
            validate(Message(part))
        return
    if kind in TREE_TYPES:
        # The trees must be well formed (each leaf with its values), and a
        # classifier of two or more classes predicts a value per class, or
        # one for two classes.
        encode_tree_part(model.data)
        if kind == 402:
            labels = model_class_labels(model.msg(402)) or []
            dimensions = (model.msg(402).msg(1) or Message(b"")).int(2)
            if len(labels) < 2 or (dimensions != len(labels) and not (len(labels) == 2 and dimensions == 1)):
                unestablished("a tree classifier predicting %d values for %d classes" % (dimensions, len(labels)))
    if kind in CHECKS:
        check, optional = CHECKS[kind]
        inputs, outputs = check_description(model, optional)
        check(model, model.msg(kind), inputs, outputs)
    model_parameters(model)


# Models compiled to archives of their own, by Model field.
OWN_ARCHIVES = {404: knn_archive, 501: item_similarity_archive}


def own_archive_files(model_bytes):
    """A k-nearest neighbors classifier or item similarity recommender."""
    model = Message(model_bytes)
    kind = next(n for n, _, _ in model.fields if n in OWN_ARCHIVES)
    if model.int(1) not in AVAILABILITY:
        unestablished("a %s of specification %d" % (MODEL_TYPES[kind], model.int(1)))
    files = OWN_ARCHIVES[kind](model)
    if model.int(10):
        # An updatable model's update parameters: none.
        files["updateParameters/coremldata.bin"] = _u64(0)
    return files


def write_bundle(outdir, name, files):
    bundle = os.path.join(outdir, name + ".mlmodelc")
    for rel, data in files.items():
        os.makedirs(os.path.dirname(os.path.join(bundle, rel)), exist_ok=True)
        with open(os.path.join(bundle, rel), "wb") as f:
            f.write(data)


def compiled_type(model):
    """Whether a .mlmodel's type is compiled here."""
    return (any(model.has(n) for n in SPEC_ARCHIVED) or tree_kind(model) is not None or pipeline_parts(model) is not None
            or any(model.has(n) for n in OWN_ARCHIVES))


def refuse_model_file(model):
    """A .mlmodel file's refusal: ML Programs come as .mlpackage, and the
    other model types aren't reproduced yet."""
    kinds = [MODEL_TYPES.get(n, "type %d" % n) for n, _, _ in model.fields if n not in (1, 2, 10)]
    kind = kinds[0] if kinds else "an unknown"
    if model.has(500) or model.has(403) or model.has(303):
        unestablished("a neural network (an older Core ML format; convert the model to an ML Program)")
    unestablished("a %s model (.mlmodel)" % kind)


# --- Packages ---------------------------------------------------------------------
#
# A .mlpackage holds any model type, not only ML Programs: Manifest.json lists
# its items by identifier, and rootModelIdentifier names the specification.
# Xcode compiles and generates a package's model as it does the same
# specification in a .mlmodel named after the package, except that
# analytics/coremldata.bin names the specification's own file, and that
# compiling (not generating) applies the package's overlays: the items Xcode
# writes when a model's metadata is edited in its model editor.

PACKAGE_FORMAT = "1.0.0"
ITEM_KEYS = ("author", "description", "name", "path")
# Overlays are found by item name and author (not path or description).
OVERLAY_AUTHOR = "com.apple.CoreML"
METADATA_OVERLAY, FEATURES_OVERLAY = "Metadata.json", "FeatureDescriptions.json"
# The Metadata overlay's keys (MLModelMetadataKey) and their Metadata fields.
METADATA_KEYS = (("MLModelDescriptionKey", 1), ("MLModelVersionStringKey", 2), ("MLModelAuthorKey", 3), ("MLModelLicenseKey", 4))
CREATOR_DEFINED_KEY = "MLModelCreatorDefinedKey"
# The FeatureDescriptions overlay's sections and their ModelDescription fields.
FEATURE_SECTIONS = (("Inputs", 1), ("Outputs", 10), ("TrainingInputs", 50))
FEATURE_DESCRIPTION_KEY = "MLFeatureShortDescription"


def _json_object(raw, what, encodings):
    """A JSON document's top-level object; duplicate keys, non-finite numbers
    and encodings other than those given are refused."""
    for bom, encoding in ((b"\xef\xbb\xbf", "utf-8"), (b"\xff\xfe", "utf-16-le"), (b"\xfe\xff", "utf-16-be"), (b"", "utf-8")):
        if raw.startswith(bom) and (bom or not raw[:4].count(b"\0")):
            if encoding not in encodings or (bom and encoding == "utf-8" and "bom" not in encodings):
                unestablished("%s in another encoding" % what)
            break
    else:
        unestablished("%s in another encoding" % what)

    def pairs(items):
        keys = [k for k, _ in items]
        if len(set(keys)) != len(keys):
            unestablished("%s with a key given twice" % what)
        return dict(items)

    def constant(name):
        unestablished("%s with %s" % (what, name))
    try:
        text = raw[len(bom):].decode(encoding)
        value = json.loads(text, object_pairs_hook=pairs, parse_constant=constant)
        # Lone surrogates can't be written out.
        json.dumps(value, ensure_ascii=False).encode("utf-8")
    except (ValueError, UnicodeError):
        unestablished("%s that isn't JSON" % what)
    if not isinstance(value, dict):
        unestablished("%s that isn't a JSON object" % what)
    return value


def _is_program(model_bytes):
    try:
        return Message(model_bytes).has(502)
    except (ValueError, IndexError):
        return False


class Package:
    """A package's specification (its bytes, directory and file name) and
    overlays (the Metadata and FeatureDescriptions JSON objects, or None)."""

    def __init__(self, path, overlays):
        manifest_path = os.path.join(path, "Manifest.json")
        if not os.path.isfile(manifest_path):
            unestablished("a package without a manifest")
        with open(manifest_path, "rb") as f:
            manifest = _json_object(f.read(), "a manifest", ("utf-8",))
        if manifest.get("fileFormatVersion") != PACKAGE_FORMAT:
            unestablished("a package of format %r" % (manifest.get("fileFormatVersion"),))
        entries = manifest.get("itemInfoEntries")
        if not isinstance(entries, dict):
            unestablished("a manifest without items")
        data = os.path.join(path, "Data")
        items, missing = {}, []
        for identifier, item in entries.items():
            if not isinstance(item, dict) or not all(isinstance(item.get(k), str) for k in ITEM_KEYS):
                unestablished("a manifest item without an author, description, name and path")
            rel = item["path"]
            if not rel or os.path.isabs(rel) or os.path.normpath(rel).split(os.sep)[0] == "..":
                unestablished("an item path outside the package's Data")
            if not os.path.exists(os.path.join(data, rel)):
                missing.append(os.path.normpath(os.path.join(data, rel)))
            items[identifier] = item
        root = manifest.get("rootModelIdentifier")
        # Looked up exactly (not, say, ignoring a UUID's case).
        if not isinstance(root, str) or root not in items:
            unestablished("a package whose root model isn't one of its items")
        root_path = os.path.join(data, items[root]["path"])
        if not os.path.isfile(root_path):
            unestablished("a root model that isn't a file")
        with open(root_path, "rb") as f:
            self.model_bytes = f.read()
        self.model_dir = os.path.dirname(root_path)
        # Xcode refuses a package missing any item it lists. But an empty
        # directory isn't kept by git or Bazel, and coremltools lists an ML
        # Program's weights directory even when it holds nothing: that one,
        # missing, is taken as empty.
        weights = os.path.normpath(os.path.join(self.model_dir, "weights"))
        for item_path in missing:
            if item_path != weights or not _is_program(self.model_bytes):
                unestablished("a package missing an item it lists")
        # analytics names the specification's file, less its extension.
        self.model_name = os.path.splitext(os.path.basename(os.path.normpath(root_path)))[0]
        self.metadata = self.features = None
        if not overlays:
            return
        for name in (METADATA_OVERLAY, FEATURES_OVERLAY):
            found = [i for k, i in items.items() if i["name"] == name and i["author"] == OVERLAY_AUTHOR]
            if not found:
                continue
            if len(found) > 1:
                unestablished("a package with two %s overlays" % name)
            if found[0] is items[root]:
                unestablished("a root model that is an overlay")
            overlay_path = os.path.join(data, found[0]["path"])
            if not os.path.isfile(overlay_path):
                unestablished("an overlay that isn't a file")
            with open(overlay_path, "rb") as f:
                value = _json_object(f.read(), "a %s overlay" % name, ("utf-8", "bom", "utf-16-le", "utf-16-be"))
            if name == METADATA_OVERLAY:
                self.metadata = check_metadata_overlay(value)
            else:
                self.features = check_features_overlay(value)


def check_metadata_overlay(value):
    """Its strings and creator-defined map; any other key is ignored."""
    for key, _ in METADATA_KEYS:
        if key in value and not isinstance(value[key], str):
            unestablished("a Metadata overlay's %s that isn't a string" % key)
    creator = value.get(CREATOR_DEFINED_KEY, {})
    if not isinstance(creator, dict) or not all(isinstance(v, str) for v in creator.values()):
        unestablished("a Metadata overlay's %s that isn't a map of strings" % CREATOR_DEFINED_KEY)
    return value


def check_features_overlay(value):
    """Each section's features' descriptions; any other key is ignored."""
    for section, _ in FEATURE_SECTIONS:
        features = value.get(section, {})
        if not isinstance(features, dict):
            unestablished("a FeatureDescriptions overlay's %s that isn't an object" % section)
        for f in features.values():
            if not isinstance(f, dict) or not isinstance(f.get(FEATURE_DESCRIPTION_KEY), str):
                unestablished("a FeatureDescriptions overlay's feature without a %s string" % FEATURE_DESCRIPTION_KEY)
    return value


def _ld(number, payload):
    """A length-delimited field."""
    return _enc_varint(number << 3 | 2) + _enc_varint(len(payload)) + payload


def _with_strings(message, values):
    """`message` (encoded as protobuf encodes it) with string fields set
    ({number: text}; an empty one left out, as proto3 leaves it)."""
    fields = [(n, raw) for n, _, _, raw in _wire_fields(message) if n not in values]
    fields += [(n, _ld(n, text.encode("utf-8"))) for n, text in values.items() if text]
    return b"".join(raw for _, raw in sorted(fields, key=lambda x: x[0]))


def _metadata_entries(message, entries):
    """A Metadata message with user-defined entries set: one already there
    keeps its place, a new one is added after them. Each entry is written
    with its key and value, as protobuf writes a map's entries."""
    out, done = [], set()
    for n, _, v, raw in _wire_fields(message):
        if n == 100:
            key = Message(v).str(1)
            if key in done:
                unestablished("user-defined metadata with a key given twice")
            if key in entries:
                raw = _ld(100, _ld(1, key.encode("utf-8")) + _ld(2, entries[key].encode("utf-8")))
            done.add(key)
        out.append(raw)
    out += [_ld(100, _ld(1, k.encode("utf-8")) + _ld(2, v.encode("utf-8"))) for k, v in entries.items() if k not in done]
    return b"".join(out)


def _described_features(features, overlay, section):
    """Features (FeatureDescription messages) with the overlay's
    descriptions; every feature the overlay names must be there."""
    given = overlay.get(section, {})
    names = {Message(f).str(1) for f in features}
    missing = [n for n in given if n not in names]
    if missing:
        unestablished("a FeatureDescriptions overlay naming %s, which isn't among the model's %s" % (missing[0], section))
    return [_with_strings(f, {2: given[Message(f).str(1)][FEATURE_DESCRIPTION_KEY]}) if Message(f).str(1) in given else f
            for f in features]


def _overlaid_description(description, metadata, features, described):
    """A ModelDescription with overlays applied, re-encoded as protobuf
    encodes it: as compiled, or as metadata.json describes it (`described`)."""
    fields = list(_wire_fields(description))
    default = Message(description).str(21)
    # Replacements by field number and occurrence.
    replaced = {}
    if features is not None:
        functions = [v for n, _, v, _ in fields if n == 20]
        sections = [(section, [v for n, _, v, _ in fields if n == number]) for section, number in FEATURE_SECTIONS]
        if functions:
            # A multi-function model's features are its default function's;
            # only metadata.json describes them with the overlay.
            function = next((v for v in functions if Message(v).str(1) == default), None)
            if function is None:
                unestablished("functions without the default function")
            inner = list(_wire_fields(function))
            ins = _described_features([v for n, _, v, _ in inner if n == 2], features, "Inputs")
            outs = _described_features([v for n, _, v, _ in inner if n == 3], features, "Outputs")
            _described_features(sections[2][1], features, "TrainingInputs")
            if described:
                new = iter(ins + outs)
                replaced[(20, functions.index(function))] = b"".join(_ld(n, next(new)) if n in (2, 3) else raw for n, _, _, raw in inner)
        else:
            for (section, old), (_, number) in zip(sections, FEATURE_SECTIONS):
                replaced.update(((number, i), f) for i, f in enumerate(_described_features(old, features, section)))
    out, seen = [], {}
    for n, _, v, raw in fields:
        i = seen[n] = seen.get(n, -1) + 1
        if n != 100:
            out.append((n, _ld(n, replaced[(n, i)]) if (n, i) in replaced else raw))
    metas = [v for n, _, v, _ in fields if n == 100]
    if len(metas) > 1:
        unestablished("a description with its metadata given twice")
    meta = metas[0] if metas else None
    if metadata is not None:
        strings = {number: metadata.get(key, "") for key, number in METADATA_KEYS}
        creator = metadata.get(CREATOR_DEFINED_KEY, {})
        if described:
            # metadata.json's metadata is the overlay's alone.
            meta = _metadata_entries(_with_strings(b"", strings), creator)
        else:
            # Compiled, a string given (and not empty) replaces the model's;
            # creator-defined entries are added to the model's.
            meta = _metadata_entries(_with_strings(meta or b"", {n: t for n, t in strings.items() if t}), creator)
    if meta is not None:
        out.append((100, _ld(100, meta)))
    return b"".join(raw for _, raw in sorted(out, key=lambda x: x[0]))


def _with_description(model_bytes, description):
    """A Model with its description replaced (the rest as it is)."""
    out = []
    for n, _, _, raw in _wire_fields(model_bytes):
        out.append((n, _ld(2, description) if n == 2 else raw))
    if not any(n == 2 for n, _ in out):
        out.append((2, _ld(2, description)))
    return b"".join(raw for _, raw in sorted(out, key=lambda x: x[0]))


def overlaid(model_bytes, package):
    """(the specification compiled, the one metadata.json describes) with a
    package's overlays applied. Compiled, the Metadata overlay's strings that
    aren't empty replace the model's and its creator-defined entries are
    added to the model's; metadata.json takes the model's metadata from the
    overlay alone. Both take the FeatureDescriptions overlay's descriptions
    (a multi-function model's default function's, in metadata.json only)."""
    if package is None or (package.metadata is None and package.features is None):
        return model_bytes, model_bytes
    description = Message(model_bytes).bytes(2)
    compiled = _overlaid_description(description, package.metadata, package.features, False)
    described = _overlaid_description(description, package.metadata, package.features, True)
    return _with_description(model_bytes, compiled), _with_description(model_bytes, described)


# --- Compiling ----------------------------------------------------------------------

def blob_files(mil):
    """The weight files model.mil reads, by name in weights/."""
    out = []
    paths = re.findall(r'BLOBFILE\(path = [^(]*\("([^"\\]*)"\)', mil)
    if len(paths) != mil.count("BLOBFILE("):
        unestablished("a weight file path needing escapes")
    for path in paths:
        rest = path[len("@model_path/weights/"):] if path.startswith("@model_path/weights/") else ""
        if not rest or "/" in rest or rest in (".", ".."):
            unestablished("weights at %s" % path)
        if rest not in out:
            out.append(rest)
    return out


def compile_model(source, outdir):
    """Compiles a .mlmodel file or .mlpackage directory."""
    package = Package(source, True) if os.path.isdir(source) else None
    if package is None:
        with open(source, "rb") as f:
            file_bytes = f.read()
    else:
        file_bytes = package.model_bytes
    name = generated_class_name(os.path.splitext(os.path.basename(source.rstrip("/")))[0])
    try:
        model = Message(file_bytes)
    except (ValueError, IndexError):
        unestablished("a specification that isn't a Core ML model")
    if package is not None and model.has(502):
        check_encoding(file_bytes)
        compiled, described = overlaid(file_bytes, package)
        files = program_files(Message(compiled))
        weights = [(os.path.join(package.model_dir, "weights", f), "weights/" + f) for f in blob_files(files["model.mil"].decode("utf-8"))]
    else:
        if not compiled_type(model):
            refuse_model_file(model)
        validate(model)
        if tree_kind(model) is None:
            # A tree ensemble's description is re-encoded here as Xcode does.
            check_encoding(file_bytes)
        compiled, described = overlaid(file_bytes, package)
        if any(model.has(n) for n in SPEC_ARCHIVED):
            files = spec_files(compiled)
        elif tree_kind(model) is not None:
            files = tree_files(compiled)
        elif pipeline_parts(model) is not None:
            files = pipeline_files(compiled)
        else:
            files = own_archive_files(compiled)
        weights = []
    files["metadata.json"] = metadata_json(Message(described), name).encode("utf-8")
    files["analytics/coremldata.bin"] = files.get("analytics/coremldata.bin", b"") + analytics_sections([("SpecificationDetails", [
        ("modelHash", base64.b64encode(hashlib.sha256(file_bytes).digest()).decode()),
        ("modelName", package.model_name if package else name)])])
    for path, _ in weights:
        if not os.path.isfile(path):
            unestablished("a package without the weights its program reads")
    write_bundle(outdir, name, files)
    for path, rel in weights:
        os.makedirs(os.path.dirname(os.path.join(outdir, name + ".mlmodelc", rel)), exist_ok=True)
        shutil.copyfile(path, os.path.join(outdir, name + ".mlmodelc", rel))


# --- coremlc generate (Swift) ----------------------------------------------------

AVAILABILITY_LINE = {
    1: "macOS 10.13, iOS 11.0, tvOS 11.0, watchOS 4.0, visionOS 1.0",
    2: "macOS 10.13.2, iOS 11.2, tvOS 11.2, watchOS 4.2, visionOS 1.0",
    3: "macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0",
    4: "macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0",
    5: "macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0",
    6: "macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0",
    7: "macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0",
    8: "macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0",
    9: "macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0",
    10: "macOS 16.0, iOS 19.0, tvOS 19.0, watchOS 12.0, visionOS 3.0",
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
        if self.optional and t.has(4):
            unestablished("an optional image's generated code")
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

    def value(self, expr, input=False):
        if self.kind == "dictionary" and input:
            # An input's dictionary is converted so.
            return "try! MLFeatureValue(dictionary: %s as [NSObject : NSNumber])" % expr
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


def update_inputs(model):
    """An updatable model's training inputs, for its update input class."""
    if not model.int(10):
        return []
    return [Feature(f) for f in (model.msg(2) or Message(b"")).msgs(50)]


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
    training = update_inputs(model)
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
    def input_class(title, cls, inputs):
        """The input class (or the update input class) and its initializers."""
        # Initializers take optional inputs last.
        params = [f for f in inputs if not f.optional] + [f for f in inputs if f.optional]
        w("/// %s\n%s\npublic class %s%s : MLFeatureProvider {\n" % (title, available, name, cls))
        for f in inputs:
            w("    /// %s\n    public var %s: %s%s\n" % (f.doc, f.swift_name, f.swift_type, "? = nil" if f.optional else ""))
        w("    public var featureNames: Set<String> { [%s] }\n" % ", ".join('"%s"' % f.name for f in inputs))
        w("    public func featureValue(for featureName: String) -> MLFeatureValue? {")
        for f in inputs:
            value = ("%s == nil ? nil : %s" % (f.swift_name, f.value(f.swift_name + "!", True))) if f.optional else f.value(f.swift_name, True)
            if f.optional and f.kind == "dictionary":
                # Its try! comes first.
                value = "try! " + value.replace("try! ", "")
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

    # Input.
    input_class("Model Prediction Input Type", "Input", inputs)
    arrays = [f for f in inputs if f.kind == "array"]
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
    if training:
        input_class("Model Update Input Type", "TrainingInput", training)
    if states:
        L[-1] = L[-1].rstrip("\n") + "\n\n" + (SWIFT_STATE.replace("{available}", available).replace("{N}", name).replace(
            "{cases}", "\n".join('        case %s = "%s"' % (swift, n) for n, swift in states)))
    template = MODEL_CLASS_SPEC3 if spec_version < 6 else MODEL_CLASS
    model_class = template.replace("{available}", available).replace("{N}", name).rstrip("\n") + "\n"
    if spec_version == 5:
        # Specification 5 drops the deprecated initializer, defaults the
        # configuration and needs nothing more for loading with a handler.
        for old, new, count in SWIFT_SPEC5:
            old = old.replace("{N}", name)
            if model_class.count(old) != count:
                unestablished("the model class of a specification 5 model")
            model_class = model_class.replace(old, new)
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


# (text, its replacement, occurrences) turning MODEL_CLASS_SPEC3 into
# specification 5's model class.
SWIFT_SPEC5 = [
    ('    /**\n        Construct {N} instance by automatically loading the model from the app\'s bundle.\n    */\n'
     '    @available(*, deprecated, message: "Use init(configuration:) instead and handle errors appropriately.")\n'
     '    public convenience init() {\n        try! self.init(contentsOf: type(of:self).urlOfModelInThisBundle)\n    }\n\n', '', 1),
    ('    public convenience init(configuration: MLModelConfiguration) throws {',
     '    public convenience init(configuration: MLModelConfiguration = MLModelConfiguration()) throws {', 1),
    ('    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)\n', '', 2),
]

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
    2: "macos(10.13.2), ios(11.2), watchos(4.2), tvos(11.2)",
    3: "macos(10.14), ios(12.0), watchos(5.0), tvos(12.0)",
    4: "macos(10.15), ios(13.0), watchos(6.0), tvos(13.0)",
    5: "macos(11.0), ios(14.0), watchos(7.0), tvos(14.0)",
    6: "macos(12.0), ios(15.0), watchos(8.0), tvos(15.0)",
    7: "macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)",
    8: "macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)",
    9: "macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)",
    10: "macos(16.0), ios(19.0), watchos(12.0), tvos(19.0)",
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
        # An optional feature is nullable, a number boxed.
        value = {"array": "featureValueWithMultiArray:(MLMultiArray *)self.%s", "string": "featureValueWithString:(NSString *)self.%s",
                 "int64": "featureValueWithInt64:self.%s.longLongValue", "double": "featureValueWithDouble:self.%s.doubleValue",
                 "dictionary": "featureValueWithDictionary:(NSDictionary *)self.%s error:nil"}.get(f.kind)
        if f.kind == "sequence":
            value = "featureValueWithSequence:[MLSequence %s:(%s)self.%%s]" % (
                "sequenceWithStringArray" if f.sequence == "string" else "sequenceWithInt64Array", out[1])
        if value is None:
            unestablished("an optional %s's generated code" % f.kind)
        if f.kind in ("int64", "double"):
            out = ("(readwrite, nonatomic, strong) NSNumber * ", "NSNumber *") + out[2:]
        out = (out[0].replace(") ", ", nullable) ", 1), "nullable " + out[1], "self.%s == nil ? nil : [MLFeatureValue " + value + "]", out[3])
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
    # An updatable model's update input class follows the output's.
    training = update_inputs(model)
    if any(f.kind == "image" or f.optional for f in training):
        unestablished("an image or optional training input's generated code")
    classes = (("Input", inputs), ("Output", outputs)) + ((("TrainingInput", training),) if training else ())
    for f in inputs + outputs:
        if not (f.name[:1].isalpha() or f.name[:1] == "_"):
            unestablished("generated code for a feature named %r" % f.name)
    images = [f for f in inputs if f.kind == "image"]
    # Initializers without the optional inputs come first.
    required = [f for f in inputs if not f.optional]
    # (Not when every input is optional.)
    reduced = required if required and len(required) < len(inputs) else None
    ios13 = " API_AVAILABLE(macos(10.15), ios(13.0), watchos(6.0), tvos(13.0)) __attribute__((visibility(\"hidden\")))" if spec_version < 4 else ""
    error = " error:(NSError * _Nullable __autoreleasing * _Nullable)error"
    H = []
    H.append("//\n// %s.h\n//\n// This file was automatically generated and should not be edited.\n//\n\n#import <Foundation/Foundation.h>\n#import <CoreML/CoreML.h>\n#include <stdint.h>\n#include <os/log.h>\n\nNS_ASSUME_NONNULL_BEGIN\n" % name)
    for kind, features in classes:
        title = "Model Update Input Type" if kind == "TrainingInput" else "Model Prediction %s Type" % kind
        H.append("/// %s\n%s\n@interface %s%s : NSObject<MLFeatureProvider>\n" % (title, available, name, kind))
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
    if spec_version < 5:
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
    for kind, features in classes:
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
        # A package's overlays aren't read: generated code describes the
        # model as its specification does.
        package = Package(source, False) if os.path.isdir(source) else None
        if package is None:
            with open(source, "rb") as f:
                model_bytes = f.read()
        else:
            model_bytes = package.model_bytes
        try:
            model = Message(model_bytes)
        except (ValueError, IndexError):
            unestablished("a specification that isn't a Core ML model")
        if package is None or not model.has(502):
            if not compiled_type(model):
                refuse_model_file(model)
            # Xcode checks the model as it reads it, for either.
            validate(model)
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
        compile_model(source, outdir)
    except Unestablished as e:
        print("error: coremlc: %s is not compiled as Xcode compiles it" % e, file=sys.stderr)
        return 1
    if plist:
        with open(plist, "w") as f:
            f.write(PARTIAL_INFO_PLIST)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
