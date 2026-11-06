#!/usr/bin/env python3
"""headers.py BIN: C headers (eexc.h, lexc.h) from a packed vimage_pow.bin, for the probes in this directory."""
import struct
import sys

data = open(sys.argv[1], "rb").read()
at = 0
tables = []
for _ in range(3):
    (n,) = struct.unpack_from("<I", data, at)
    at += 4
    tables.append(struct.unpack_from("<%dI" % (2 * n), data, at))
    at += 8 * n


def write(f, name, t):
    xs, ys = t[0::2], t[1::2]
    f.write("static const uint32_t %s_N = %d;\nstatic const uint32_t %s_X[] = {%s};\nstatic const uint32_t %s_Y[] = {%s};\n" % (
        name, len(xs), name, ",".join("0x%08x" % x for x in xs) or "0", name, ",".join("0x%08x" % y for y in ys) or "0"))


with open("eexc.h", "w") as f:
    write(f, "EXC", tables[0])
with open("lexc.h", "w") as f:
    write(f, "LPOS", tables[1])
    write(f, "LNEG", tables[2])
