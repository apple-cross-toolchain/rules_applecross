"""payloads.py CAR: each bitmap rendition's (width, height) -> its pixel payload (after the TLVs)."""
import struct, sys
def payloads(path):
    d = open(path, 'rb').read(); out = {}; k = 0
    while True:
        k = d.find(b'ISTC', k)
        if k < 0: break
        b = d[k:]
        w, h = struct.unpack_from('<II', b, 12)
        tlvs = struct.unpack_from('<I', b, 168)[0]; length = struct.unpack_from('<I', b, 180)[0]
        if w and h: out[(w, h)] = b[184 + tlvs:184 + tlvs + length - 0]
        k += 4
    return out
if __name__ == '__main__':
    for k, v in payloads(sys.argv[1]).items(): print(k, len(v), v[:16])
