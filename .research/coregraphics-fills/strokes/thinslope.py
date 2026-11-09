import sys,math,collections; sys.path.insert(0,'/tmp/cg'); sys.path.insert(0,'/tmp/cgw')
from model import clip; import poly
from thinsweep import alphas
def rect(a,b,w):
    dx,dy=b[0]-a[0],b[1]-a[1]; L=math.hypot(dx,dy); n=(-dy/L*w/2,dx/L*w/2)
    return [(a[0]+n[0],a[1]+n[1]),(b[0]+n[0],b[1]+n[1]),(b[0]-n[0],b[1]-n[1]),(a[0]-n[0],a[1]-n[1])]
import random
random.seed(1); stats=collections.Counter(); ex=[]
for _ in range(40):
    a=(random.uniform(1,3),random.uniform(1,7)); b=(random.uniform(5,7),random.uniform(1,7)); w=random.choice([1.0,0.5,0.75])
    A=alphas('m %r %r\nl %r %r'%(a+b),w)
    R=rect(a,b,w)
    for y in range(8):
        for x in range(8):
            ar=abs(poly.sarea(clip(R,x,y,x+1,y+1)))
            if ar<=0 and A[y][x]==0: continue
            c=A[y][x]
            for name,v in (('floor256',min(255,math.floor(256*ar))),('ceil256-1',max(0,min(255,math.ceil(256*ar)-1))),('round255',round(255*ar))):
                stats[(name,v==c)]+=1
            if len(ex)<12 and c!=min(255,math.floor(256*ar)): ex.append((a,b,w,x,y,c,round(256*ar,2)))
print(sorted(stats.items()))
for e in ex: print(e)
