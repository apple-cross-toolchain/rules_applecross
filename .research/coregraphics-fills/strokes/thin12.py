import sys,math,random; sys.path.insert(0,'/tmp/cgw')
from thinsweep import alphas
random.seed(9)
for _ in range(10):
    x0=random.randint(0,255)/256; y0=random.randint(3*256,6*256)/256
    dyu=random.randint(-60,60)
    a=(x0,y0); b=(24+x0,y0+dyu/256)
    m=(b[1]-a[1])/(b[0]-a[0]); sec=math.hypot(1,m); T=math.floor(256*sec)
    S=round(m*512)
    A=alphas('m %r %r\nl %r %r'%(a+b),1.0,W=26,H=10)
    lo,hi=-1e9,1e9
    for x in range(2,24):
        col=[(r,A[r][x]) for r in range(10) if A[r][x]>0]
        if len(col)!=2: continue
        (r0,v0),(r1,v1)=col
        bot=r1*256-v0
        # model bot = floor((Y0 + x*S)/2)
        lo=max(lo,2*bot-x*S); hi=min(hi,2*bot+2-x*S)
    ex=(y0+m*(0.5-x0))*512-T   # exact 2*bot at column 0 (1/512 units)
    print('x0 %3d y0 %4d dy %3d S %d'%(x0*256,y0*256,dyu,S),'Y0 in [%.3f, %.3f) width %.3f'%(lo,hi,hi-lo),'exact %.3f'%ex,'Y0-exact [%.3f,%.3f)'%(lo-ex,hi-ex))
