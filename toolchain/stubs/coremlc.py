#!/usr/bin/env python3
"""coremlc off a Mac: `coremlc compile` of a Core ML model, as rules_apple
runs it; see docs/clean-room/coremlc.md.

Work in progress: this module prints an ML Program's model.mil from the
model's specification (Apple's published Core ML protobuf schema).
"""

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
        for op in block.msgs(3):
            self.operation(op, indent)
        return ", ".join(self.ref(o.decode()) for o in block.all(2))

    def operation(self, op, indent):
        pad = " " * indent
        outputs = op.msgs(3)
        if len(outputs) != 1:
            unestablished("an operation with %d outputs" % len(outputs))
        out = outputs[0]
        inputs = []
        for name, argument in sorted(op.map(2), key=lambda kv: kv[0]):
            bindings = argument.msgs(1)
            if len(bindings) != 1 or not bindings[0].has(1):
                unestablished("an operation input other than one name")
            inputs.append("%s = %s" % (name, self.ref(bindings[0].str(1))))
        attributes = ["%s = %s" % (name, self.value(v)) for name, v in sorted(op.map(5), key=lambda kv: kv[0])]
        head = "%s%s %s = %s(%s)[%s]" % (pad, self.value_type(out.msg(2)), "%s", op.str(1), ", ".join(inputs), ", ".join(attributes))
        blocks = op.msgs(4)
        if not blocks:
            self.lines.append(head % self.declare(out.str(1)) + ";")
            return
        at = len(self.lines)
        self.lines.append(None)
        inner = " " * (indent + 4)
        for k, b in enumerate(blocks):
            self.scope.append({})
            params = ", ".join("%s %s" % (self.value_type(i.msg(2)), self.declare(i.str(1))) for i in b.msgs(1))
            self.lines.append(inner + ("(%s) {" % params if b.msgs(1) else "{"))
            outputs = self.block(b, indent + 8)
            self.scope.pop()
            self.lines.append(inner + "} -> (%s)%s" % (outputs, ";" if k == len(blocks) - 1 else ""))
        # The operation's output is declared after its blocks' names.
        self.lines[at] = head % self.declare(out.str(1))


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


def model_mil(model_bytes, extra_build_info):
    model = Message(model_bytes)
    return print_program(model.msg(502) or unestablished("a model that isn't an ML Program"), model.int(1), extra_build_info)


if __name__ == "__main__":
    sys.stdout.write(model_mil(open(sys.argv[1], "rb").read(), [("coremlc-component-MIL", "3600.16.1"), ("coremlc-version", "3600.25.1")]))
