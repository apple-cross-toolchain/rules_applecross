import sys,math,random,collections; sys.path.insert(0,'/tmp/cgw'); sys.path.insert(0,'/tmp/cg')
from thinsweep import alphas
def band_area(yb0,slope,T,k,r):
    # band bottom y(x)=yb0+slope*(x-(k+0.5)) within column k (x in [k,k+1]); top = bottom + T/256 ; area in row r (units 1/256 px^2 *256)
    N=64; s=0.0
    for i in range(N):
        x=k+(i+0.5)/N; b=yb0+slope*(x-(k+0.5)); t=b+T/256
        s+=max(0.0,min(t,r+1)-max(b,r))
    return s/N*256
def col_values(yb,slope,T,k,rule):
    rows=[r for r in range(int(math.floor(yb-1)),int(math.floor(yb+T/256+2)))]
    ar={r:band_area(yb,slope,T,k,r) for r in rows}
    ar={r:v for r,v in ar.items() if v>1e-9}
    if rule=='outerfloor':
        big=max(ar,key=ar.get); out={r:math.floor(v) for r,v in ar.items() if r!=big}; out[big]=T-sum(out.values())
    elif rule=='floor': out={r:math.floor(v) for r,v in ar.items()}
    elif rule=='lowestfloor':
        rs=sorted(ar); out={r:math.floor(ar[r]) for r in rs[:-1]}; out[rs[-1]]=T-sum(out.values())
    elif rule=='topfloor':
        rs=sorted(ar); out={r:math.floor(ar[r]) for r in rs[1:]}; out[rs[0]]=T-sum(out.values())
    return {r:min(255,v) for r,v in out.items() if v>0}
random.seed(51)
for _ in range(6):
    m=random.uniform(-0.8,0.8); y0=random.uniform(4,7); x0=random.uniform(0.1,0.9)
    a=(x0,y0); b=(14.5,y0+(14.5-x0)*m)
    A=alphas('m %r %r\nl %r %r'%(a+b),1.0,W=16,H=24)
    X0,Y0,X1,Y1=[math.floor(v*4096+0.5) for v in (a[0],a[1],b[0],b[1])]
    dx,dy=X1-X0,Y1-Y0; s=int((dy*4096+dx//2)/dx); T=math.floor(256*math.hypot(1,dy/dx))
    slope=s/4096
    yref=(y0+m*(0.5-x0))-T/512
    res={}
    for rule in ('outerfloor','lowestfloor','topfloor'):
        best=(-1,0)
        for off in range(-40,41):
            yb0=yref+off/4096
            ok=0
            for k in range(1,13):
                cv=col_values(yb0+slope*k,slope,T,k,rule)
                cg={r:A[r][k] for r in range(24) if A[r][k]}
                ok+=cv==cg
            if ok>best[0]: best=(ok,off)
        res[rule]=best
    print('m %.3f T %d'%(m,T),res)
