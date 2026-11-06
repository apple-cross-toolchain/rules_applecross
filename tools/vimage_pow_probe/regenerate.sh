#!/bin/bash
# regenerate.sh: vImage's exp2 and log2 exceptions, then the checks (on a Mac, in this directory).
set -euo pipefail
cd "$(dirname "$0")"
cc() { clang -O2 -ffp-contract=off -framework Accelerate "$@"; }
echo "static const uint32_t EXC_N = 0; static const uint32_t EXC_X[] = {0}; static const uint32_t EXC_Y[] = {0};" > eexc.h
cc exp2_exceptions.c -o exp2_exceptions
python3 - <<'P' > parts.txt
lo, hi, n = 0x00000001, 0x3f800000, 16
step = (hi - lo) // n
for i in range(n):
    print(hex(lo + i * step), hex(hi if i == n - 1 else lo + (i + 1) * step), i)
P
while read -r a b i; do ./exp2_exceptions "$a" "$b" "exp2_$i.bin" & done < parts.txt; wait
cat exp2_*.bin > exp2.bin
: > empty.bin
python3 pack.py exp2.bin empty.bin empty.bin vimage_pow.bin && python3 headers.py vimage_pow.bin
cc log2_recover.c -o log2_recover
./log2_recover 0.70710677 1 lneg2.bin & ./log2_recover 1 1.41421354 lpos2.bin & wait
python3 - <<'P'
import struct
above = [0x3fb8aa3b, 0xbf38aa06, 0x3ef62766, 0xbeb7a5bf, 0x3e8c16d0, 0xbe383a67, 0x3d8c36eb]
below = [0x3fb8aa3b, 0xbf38a9f3, 0x3ef6559a, 0xbeb66ff0, 0x3ea80bf8, 0xbd7a8021, 0x3f126d6c]
open("lfin_pos.bin", "wb").write(struct.pack("<7I", *above))
open("lfin_neg.bin", "wb").write(struct.pack("<7I", *below))
P
cc log2_exceptions.c -o log2_exceptions && ./log2_exceptions
python3 pack.py exp2.bin lexc_pos.bin lexc_neg.bin vimage_pow.bin && python3 headers.py vimage_pow.bin
cc validate_pow.c -o validate_pow && ./validate_pow
cc validate_curves.c -o validate_curves && ./validate_curves 1 0x3d000000 0x3d100000
cmp vimage_pow.bin ../../tools/apple_tools/crates/ibtool/src/vimage_pow.bin && echo "matches the crate's vimage_pow.bin"
