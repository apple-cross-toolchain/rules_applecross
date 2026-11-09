import sys,math,pickle; sys.path.insert(0,'/tmp/cgw'); from offs import outline,bez,off; from offmodel import piece
from bnd2 import nearest
def accepted(q,w):
    n,els=outline([q],w)[0]; return n==1 and sum(1 for e in els if e[0]=='c')==2
def fam(s,u,v): return [(0.0,0.0),(s*u,0.0),(float(s),s-s*v),(float(s),float(s))]
w=0.001
for s,u in ((10,0.5),(10,0.4),(40,0.5),(40,0.6),(20,0.5),(80,0.6),(20,0.3)):
    vs=[0.05+0.01*i for i in range(100)]
    a=[accepted(fam(s,u,v),w) for v in vs]
    for i in range(len(vs)-1):
        if a[i]!=a[i+1]:
            lo,hi=vs[i],vs[i+1]; flo=a[i]
            for _ in range(45):
                m=(lo+hi)/2
                if accepted(fam(s,u,m),w)==flo: lo=m
                else: hi=m
            v=(lo+hi)/2; q=fam(s,u,v); c=piece(q,0.0)
            e_a2o=max(nearest(lambda t:bez(q,t),bez(c,k/64)) for k in range(1,64))
            e_o2a=max(nearest(lambda t:bez(c,t),bez(q,k/64)) for k in range(1,64))
            e4=max(nearest(lambda t:bez(q,t),bez(c,k/4)) for k in range(1,4))
            e2=nearest(lambda t:bez(q,t),bez(c,.5))
            print('s',s,'u',u,'v %.6f'%v,'a2o %.5f o2a %.5f a2o@q %.5f mid %.5f'%(e_a2o,e_o2a,e4,e2))
