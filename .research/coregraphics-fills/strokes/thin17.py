import sys,math,random; sys.path.insert(0,'/tmp/cgw')
from thinsweep import alphas
random.seed(21)
for _ in range(14):
    m=random.uniform(-0.2,0.2); y0=random.uniform(4,8); x0=random.uniform(0.05,0.95)
    a=(x0,y0); b=(18.5,y0+(18.5-x0)*m)
    A=alphas('m %r %r\nl %r %r'%(a+b),1.0,W=20,H=24)
    X0,Y0,X1,Y1=[math.floor(v*4096+0.5) for v in (a[0],a[1],b[0],b[1])]
    dx,dy=X1-X0,Y1-Y0; s=int((dy*4096+dx//2)/dx)
    T=math.floor(256*math.hypot(1,dy/dx))
    bots={}
    for k in range(1,18):
        col=[(r,A[r][k]) for r in range(24) if A[r][k]>0]
        if len(col)==2 and col[0][1]+col[1][1]==T: bots[k]=col[0][0]*256+256-col[0][1]
    sols=[Aa for Aa in range(16*min(bots.values())-16*40,16*max(bots.values())+16*40) if all(((Aa+s*k)//16)==v for k,v in bots.items())]
    if not sols: print('no sol', s, T); continue
    Amin,Amax=min(sols),max(sols)
    ref=Y0-T*8+s*(0.5-x0)   # band bottom at col0 center in 1/4096 using quantized slope and T/2
    print('T %d s %4d  A in [%d,%d]  A-ref [%.2f,%.2f]  frac(x0)*s %.2f  Y0%%16 %d'%(T,s,Amin,Amax,Amin-ref,Amax-ref,x0*s,Y0%16))
