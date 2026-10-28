from probe import *
def s1(): return [B(0,0,0,3.0,1,2), L(0,1,10.0), L(0,2,20.0)]
reg('FoldF', [L(0,0,1.0), L(1,0,2.0**-30)] + [B(2,0,0,3.0,1,2), L(2,1,10.0), L(2,2,20.0)])
reg('FoldF2', [L(0,0,1.0), L(1,0,2.0**-30)])
reg('BaseD_leaf', [L(0,0,1.0)], base=[0.1])
reg('ThrF64', [B(0,0,0,3.0,1,2), L(0,1,10.0), L(0,2,20.0)], base=[1e300])
reg('NoBase', s1(), base=[])
reg('Det1', s1()); reg('Det2', s1())
compile_all()
