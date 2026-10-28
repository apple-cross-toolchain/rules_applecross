from probe import *
from b5 import ch
import itertools
# grid: 1 tree with L leaves (chain), npd, n total values
def leaves_for(L, npd, n):
    # distribute n values across L leaves, at least 1 each, max npd each; ensure not all exactly 1
    cnt = [1]*L; extra = n - L; i = 0
    while extra > 0:
        if cnt[i] < npd: cnt[i] += 1; extra -= 1
        i = (i+1) % L
    return [[(d, 1.0 + j + d*0.25) for d in range(c)] for j, c in enumerate(cnt)]
names = []
for npd in [2,3,4,5,6,8]:
    for L in [2,3,4]:
        for n in range(L+1, L*npd):
            nm = f'G_{npd}_{L}_{n}'
            reg(nm, ch(0, L-1, leaves_for(L, npd, n)), npd=npd)
# multi-tree grid npd=3, 2 trees of 2 leaves
for n in range(5, 12):
    lv = leaves_for(4, 3, n)
    reg(f'GT_3_{n}', ch(0,1,lv[:2]) + ch(1,1,lv[2:], start=5.0), npd=3)
reg('K4_max3', ch(0,1,[[(0,1.0)],[(3,2.0)]]), npd=4)
reg('K4_max3b', ch(0,1,[[(0,1.0)],[(3,2.0)]]), npd=7)
compile_all()
