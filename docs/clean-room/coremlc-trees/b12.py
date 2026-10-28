"""Holdout: random ensembles (not used to fit rules)."""
from probe import *
import random
R = random.Random(98765)
MODESL = ['LTE','LT','GTE','GT','EQ','NE']
def rtree(t, npd, depth, modes, pmiss, nfeat, dbl, sparse):
    nodes = []; nid = [0]
    ids = list(range(1000)); R.shuffle(ids)
    def mk(dp):
        my = ids[nid[0]]; nid[0] += 1
        if dp == 0 or R.random() < 0.3:
            if npd == 1: ev = [(0, val(dbl))]
            else:
                k = R.randint(1, npd) if sparse else npd
                dims = R.sample(range(npd), k)
                ev = [(dd, val(dbl)) for dd in dims]
            nodes.append(L(t, my, ev)); return my
        a = mk(dp-1); b = mk(dp-1)
        nodes.append(B(t, my, R.randrange(nfeat), val(dbl), a, b, mode=R.choice(modes), miss=R.random() < pmiss))
        return my
    mk(depth)
    R.shuffle(nodes)
    return nodes
def val(dbl):
    x = R.choice([R.randint(-100, 100) / 4.0, R.uniform(-50, 50)])
    return x if dbl else float(__import__('numpy').float32(x))
for i in range(100):
    npd = R.choice([1,1,2,3,4,7,12])
    modes = R.choice([['LT'], ['LTE'], ['LT','GTE'], ['LTE','GT'], MODESL, ['EQ']])
    nfeat = R.choice([1,3,17,300])
    ntrees = R.randint(1, 6)
    dbl = R.random() < 0.2
    sparse = R.random() < 0.5
    pmiss = R.choice([0, 0.5, 1.0])
    nodes = []
    tids = R.sample(range(50), ntrees)
    for t in tids: nodes += rtree(t, npd, R.randint(0, 8), modes, pmiss, nfeat, dbl, sparse)
    base = [R.choice([0.0, 0.5, 0.1]) for _ in range(npd)]
    if R.random() < 0.3 and npd > 1:
        labels = ['c%d' % j for j in range(npd)] if R.random() < 0.5 else list(range(10, 10+npd))
        cls('RNE%02d' % i, nodes, labels, nfeat=nfeat, npd=npd, base=base, transform=R.choice([0,1,3]))
    else:
        reg('RNE%02d' % i, nodes, nfeat=nfeat, npd=npd, base=base, transform=R.choice([0,1,2,3]), arrin=True)
compile_all()
