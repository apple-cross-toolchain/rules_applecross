import sys,math,random; sys.path.insert(0,'/tmp/cgw')
from thinsweep import alphas
random.seed(9)
for _ in range(10):
    x0=random.randint(0,255)/256; y0=random.randint(3*256,6*256)/256
    dyu=random.randint(-60,60)
    a=(x0,y0); b=(24+x0,y0+dyu/256)
    m=(b[1]-a[1])/(b[0]-a[0]); sec=math.hypot(1,m); T=math.floor(256*sec)
    A=alphas('m %r %r\nl %r %r'%(a+b),1.0,W=26,H=10)
    es=[]
    for x in range(2,24):
        col=[(r,A[r][x]) for r in range(10) if A[r][x]>0]
        if len(col)!=2: continue
        (r0,v0),(r1,v1)=col
        bot=r1*256-v0; top=r1*256+v1
        yc=y0+m*(x+0.5-x0)
        es.append(round(bot-(yc*256-T/2),3))
    print('x0*256 %3d y0*256 %4d dy %3d T %d'%(x0*256,y0*256,dyu,T),'e range [%.3f, %.3f]'%(min(es),max(es)) if es else '', es[:8])
