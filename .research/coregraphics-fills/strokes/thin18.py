import sys,math,random; sys.path.insert(0,'/tmp/cgw')
from thinsweep import alphas
random.seed(31)
rows=[]
for _ in range(16):
    m=random.uniform(-0.15,0.15); y0=random.uniform(4,8); x0=random.uniform(0.05,0.95)
    a=(x0,y0); b=(18.5,y0+(18.5-x0)*m)
    A=alphas('m %r %r\nl %r %r'%(a+b),1.0,W=20,H=24)
    X0,Y0,X1,Y1=[math.floor(v*4096+0.5) for v in (a[0],a[1],b[0],b[1])]
    dx,dy=X1-X0,Y1-Y0; s=int((dy*4096+dx//2)/dx); T=math.floor(256*math.hypot(1,dy/dx))
    bots={}
    for k in range(1,18):
        col=[(r,A[r][k]) for r in range(24) if A[r][k]>0]
        if len(col)==2 and col[0][1]+col[1][1]==T: bots[k]=col[0][0]*256+256-col[0][1]
    # robust: choose A maximizing matches
    best=None
    for Aa in range(16*min(bots.values())-16*30,16*max(bots.values())+16*30):
        c=sum(((Aa+s*k)//16)==v for k,v in bots.items())
        if best is None or c>best[0]: best=(c,Aa,Aa)
        elif c==best[0]: best=(c,best[1],Aa)
    c,lo,hi=best
    refs={'Y0-8T+s(.5-x0)':Y0-8*T+s*(0.5-x0),'Y0-8T':Y0-8*T,'Y0-8T+s(.5-frac)':Y0-8*T+s*(0.5-(x0%1)),'Y0-8T-s*x0':Y0-8*T-s*x0}
    print('s %4d T %d match %d/%d A[%d,%d]'%(s,T,c,len(bots),lo,hi),' '.join('%s:%.2f'%(k,lo-v) for k,v in refs.items()),'x0 %.3f'%x0)
