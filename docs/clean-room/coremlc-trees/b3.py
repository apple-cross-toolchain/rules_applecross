from probe import *
from b2 import chain
def s(t=0, thr=3.0, a=10.0, b=20.0, fi=0, **kw): return [B(t,0,fi,thr,1,2,**kw), L(t,1,a), L(t,2,b)]
# roots
for sizes in [(1,1),(1,2),(2,1),(2,2),(3,3),(4,1),(1,1,1),(1,1,2),(2,1,1),(1,2,1),(8,1),(16,1),(7,1),(1,1,1,1),(3,1,1)]:
    nodes = []
    for t, k in enumerate(sizes): nodes += chain(t, k, start=t*10+1.0, leafbase=100.0*t)
    reg('R_' + '_'.join(map(str, sizes)), nodes)
# missing-all
reg('MA_lte', s(mode='LTE', miss=True))
reg('MA_eq', s(mode='EQ', miss=True))
reg('MA_lt2', s(miss=True) + s(1, 5.0, miss=True))
reg('MA_lt_mix', s(miss=True) + s(1, 5.0, miss=False))
reg('MA_ltlte', s(miss=True) + s(1, 5.0, mode='LTE', miss=True))
reg('MA_gte', s(mode='GTE', miss=True))
# npd and ev
reg('N2_leaf', [L(0,0,[1.0, 2.0])], npd=2, base=[0.5, 0.25])
reg('N2_s', [B(0,0,0,3.0,1,2), L(0,1,[1.0,2.0]), L(0,2,[3.0,4.0])], npd=2)
reg('N2_s_b', [B(0,0,0,3.0,1,2), L(0,1,[1.0,2.0]), L(0,2,[3.0,4.0])], npd=2, base=[0.5, 0.25])
reg('N3_s', [B(0,0,0,3.0,1,2), L(0,1,[1.0,2.0,5.0]), L(0,2,[3.0,4.0,6.0])], npd=3)
reg('N3_sparse', [B(0,0,0,3.0,1,2), L(0,1,[(0,1.0)]), L(0,2,[(2,4.0)])], npd=3)
reg('N3_sparse2', [B(0,0,0,3.0,1,2), L(0,1,[(1,1.0)]), L(0,2,[(1,4.0),(2,5.0)])], npd=3)
reg('N2_empty', [B(0,0,0,3.0,1,2), L(0,1,[]), L(0,2,[(1,4.0)])], npd=2)
reg('N1_empty', [B(0,0,0,3.0,1,2), L(0,1,[]), L(0,2,20.0)])
reg('N1_dup', [B(0,0,0,3.0,1,2), L(0,1,[(0,1.0),(0,2.0)]), L(0,2,20.0)])
reg('N3_order', [B(0,0,0,3.0,1,2), L(0,1,[(2,1.0),(0,2.0)]), L(0,2,[(1,4.0)])], npd=3)
reg('N2_2t', [B(0,0,0,3.0,1,2), L(0,1,[1.0,2.0]), L(0,2,[3.0,4.0]), B(1,0,0,5.0,1,2), L(1,1,[5.0,6.0]), L(1,2,[7.0,8.0])], npd=2)
# hit rates
reg('H_hit', [B(0,0,0,3.0,1,2,hit=0.7), L(0,1,10.0,hit=0.2), L(0,2,20.0,hit=0.5)])
# transforms
for tf in range(4): reg('TF%d' % tf, s(), transform=tf)
reg('TF1_n2', [B(0,0,0,3.0,1,2), L(0,1,[1.0,2.0]), L(0,2,[3.0,4.0])], npd=2, transform=1)
# spec versions
for sv in [1, 3, 4, 5, 9]: reg('SV%d' % sv, s(), sv=sv)
reg('L1_sv4', [L(0,0,1.0)], sv=4)
# float/double edge: leaf sums
reg('Fold2', [L(0,0,0.1), L(1,0,0.2)])
reg('Fold_f', [L(0,0,0.5)] + s(1), base=[0.25])
# shared node ids across trees, tree id gaps, node ids sparse
reg('Gap', s(0) + s(7, 5.0))
reg('Sparse', [B(3,100,0,3.0,50,20), L(3,50,10.0), L(3,20,20.0)])
compile_all()
