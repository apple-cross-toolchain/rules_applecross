#!/bin/zsh
# check.sh NAME PDF: Xcode's drawing of PDF against CoreGraphics' drawing compiled as PNGs, per scale
cd /tmp/cg; n=$1
./compile.sh v_$n $2 > /dev/null
for s in 1 2 3; do ./render $2 $s ${n}_$s.png; done
./bitmap.sh b_$n ${n}_1.png ${n}_2.png ${n}_3.png
python3 -I -c '
import sys; sys.path.insert(0, "/tmp/cg"); from payloads import payloads
v = payloads(sys.argv[1]); b = payloads(sys.argv[2])
print(sys.argv[3], " ".join("%dx%d:%s" % (k[0], k[1], "SAME" if v.get(k) == b.get(k) else "DIFF") for k in sorted(v)))' v_$n/x/Assets.car b_$n/x/Assets.car $n
