from probe import *
def chain(tree, n_int, start=1.0, leafbase=10.0, fi=0, mode='LT'):
    # chain: node 2k: true->leaf 2k+1, false->node 2k+2 ; last node has two leaves
    nodes = []
    for k in range(n_int):
        i = 2*k
        if k < n_int-1:
            nodes += [B(tree, i, fi, start+k, i+2, i+1, mode=mode), L(tree, i+1, leafbase+k)]
        else:
            nodes += [B(tree, i, fi, start+k, i+1, i+2, mode=mode), L(tree, i+1, leafbase+k), L(tree, i+2, leafbase+k+0.5)]
    return nodes
def s(fi): return [B(0,0,fi,3.0,1,2), L(0,1,10.0), L(0,2,20.0)]
for fi in list(range(4, 20)) + [31,32,33,63,64,65,127,128,129,255,256,511,512,1000,1023,1024,65535,65536]:
    reg('FI%d' % fi, s(fi), nfeat=fi+1, arrin=True)
reg('FI8_n100', s(8), nfeat=100, arrin=True)
for n in range(1, 20): reg('Ch%d' % n, chain(0, n))
for n in [31, 32, 33, 63, 64, 65, 127, 128, 129, 300]: reg('Ch%d' % n, chain(0, n))
# multiple trees of different sizes
reg('T_1_3', chain(0,1) + chain(1,3, start=5.0, leafbase=50.0))
reg('T_3_1', chain(0,3) + chain(1,1, start=5.0, leafbase=50.0))
reg('T_2_2_2', chain(0,2) + chain(1,2, start=5.0, leafbase=50.0) + chain(2,2, start=7.0, leafbase=70.0))
reg('T_1x5', sum([chain(t,1,start=t+1.0, leafbase=10.0*t+5) for t in range(5)], []))
reg('T_1x9', sum([chain(t,1,start=t+1.0, leafbase=10.0*t+5) for t in range(9)], []))
reg('T_5_1', chain(0,5) + chain(1,1, start=9.0, leafbase=50.0))
reg('T_1_5', chain(0,1) + chain(1,5, start=9.0, leafbase=50.0))
reg('T_L_1', [L(0,0,3.0)] + chain(1,1))
reg('T_1_L', chain(0,1) + [L(1,0,3.0)])
# modes mixed
def two(m1, m2, miss=(False, False)):
    return [B(0,0,0,3.0,1,2,mode=m1,miss=miss[0]), L(0,1,10.0), L(0,2,20.0), B(1,0,0,5.0,1,2,mode=m2,miss=miss[1]), L(1,1,30.0), L(1,2,40.0)]
for a, b in [('LT','LTE'),('LT','EQ'),('LTE','EQ'),('LT','NE'),('GT','GTE'),('LTE','NE'),('GT','EQ')]:
    reg('MM_%s_%s' % (a, b), two(a, b))
reg('MM3', two('LT','LTE') + [B(2,0,0,7.0,1,2,mode='EQ'), L(2,1,1.0), L(2,2,2.0)])
reg('MM3n', two('GTE','GT') + [B(2,0,0,7.0,1,2,mode='NE'), L(2,1,1.0), L(2,2,2.0)])
reg('MMm', two('LT','LTE', miss=(True, False)))
reg('MMm2', two('LT','EQ', miss=(False, True)))
reg('Mm_tf', [B(0,0,0,3.0,1,2,miss=True), L(0,1,10.0), B(0,2,0,4.0,3,4), L(0,3,1.0), L(0,4,2.0)])
reg('Mm_ft', [B(0,0,0,3.0,2,1,miss=True), L(0,1,10.0), B(0,2,0,4.0,3,4), L(0,3,1.0), L(0,4,2.0)])
reg('M_eq_tf', [B(0,0,0,3.0,1,2,mode='EQ'), L(0,1,10.0), B(0,2,0,4.0,3,4,mode='EQ'), L(0,3,1.0), L(0,4,2.0)])
reg('M_gt_tf', [B(0,0,0,3.0,1,2,mode='GT'), L(0,1,10.0), B(0,2,0,4.0,3,4,mode='GT'), L(0,3,1.0), L(0,4,2.0)])
# double vs float
reg('D_thr', [B(0,0,0,0.1,1,2), L(0,1,10.0), L(0,2,20.0)])
reg('D_leaf', [B(0,0,0,3.0,1,2), L(0,1,0.1), L(0,2,20.0)])
reg('D_base', [B(0,0,0,3.0,1,2), L(0,1,10.0), L(0,2,20.0)], base=[0.1])
reg('D_thr_big', [B(0,0,0,1e300,1,2), L(0,1,10.0), L(0,2,20.0)])
reg('D_thr_sub', [B(0,0,0,1e-45,1,2), L(0,1,10.0), L(0,2,20.0)])
reg('D_thr_2', [B(0,0,0,3.0,1,2), L(0,1,10.0), B(0,2,0,0.1,3,4), L(0,3,1.0), L(0,4,2.0)])
reg('F_neg', [B(0,0,0,-3.0,1,2), L(0,1,-10.0), L(0,2,20.0)])
reg('F_nan', [B(0,0,0,float('nan'),1,2), L(0,1,10.0), L(0,2,20.0)])
reg('F_inf', [B(0,0,0,float('inf'),1,2), L(0,1,float('-inf')), L(0,2,20.0)])
reg('F_negz', [B(0,0,0,-0.0,1,2), L(0,1,-0.0), L(0,2,20.0)])
compile_all()
