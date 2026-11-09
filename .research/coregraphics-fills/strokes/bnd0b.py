import sys,math,pickle; sys.path.insert(0,'/tmp/cgw'); from offs import outline,bez,off; from offmodel import piece
from bnd2 import nearest
B=[(10,0.5,0.196421),(10,0.5,0.949811),(10,0.4,0.134668),(10,0.4,0.941872),(40,0.5,0.391940),(40,0.5,0.803066),(40,0.6,0.451418),(40,0.6,0.823126),(20,0.5,0.315766),(20,0.5,0.874452),(80,0.6,0.507236),(80,0.6,0.779118),(20,0.3,0.166615),(20,0.3,0.475332)]
def fam(s,u,v): return [(0.0,0.0),(s*u,0.0),(float(s),s-s*v),(float(s),float(s))]
for N in (2,3,4,5,6,8,10,12,16):
    for kind in ('a2o','o2a','pw'):
        mx=[]
        for s,u,v in B:
            q=fam(s,u,v); c=piece(q,0.0)
            if kind=='a2o': e=[nearest(lambda t:bez(q,t),bez(c,k/N)) for k in range(1,N)]
            elif kind=='o2a': e=[nearest(lambda t:bez(c,t),bez(q,k/N)) for k in range(1,N)]
            else: e=[math.dist(bez(c,k/N),bez(q,k/N)) for k in range(1,N)]
            mx.append(max(e))
        print('N',N,kind,'boundary maxima min %.5f max %.5f'%(min(mx),max(mx)))
