#!/usr/bin/env python3
"""pack.py EXP2 LOG2_ABOVE LOG2_BELOW OUT: the exception lists (pairs of input and output float bits)
as crates/ibtool/src/vimage_pow.bin: for each, a little-endian count and the sorted pairs."""
import struct
import sys


def pairs(path):
    data = open(path, "rb").read()
    words = struct.unpack("<%dI" % (len(data) // 4), data)
    return sorted(zip(words[0::2], words[1::2]))


out = b""
for path in sys.argv[1:4]:
    table = pairs(path)
    out += struct.pack("<I", len(table)) + b"".join(struct.pack("<II", x, y) for x, y in table)
open(sys.argv[4], "wb").write(out)
