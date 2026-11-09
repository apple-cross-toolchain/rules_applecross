import sys,math,random,collections; sys.path.insert(0,'/tmp/cgw')
from thinsweep import alphas
def model(a,b,W,H,var):
    (x0,y0),(x1,y1)=sorted([a,b])
    X0,Y0,X1,Y1=[math.floor(v*4096+0.5) for v in (x0,y0,x1,y1)]
    dx,dy=X1-X0,Y1-Y0
    s=int((dy*4096+dx//2)/dx) if True else 0     # C truncation
    m=dy/dx; sec=math.hypot(1,m); T=math.floor(256*sec)
    out={}
    for k in range(W):
        if not (x0<=k and k+1<=x1): continue
        xc=(k+0.5)*4096
        if var=='center': Y=Y0+s*(xc-X0)/4096
        elif var=='centerfloor': Y=Y0+math.floor(s*(xc-X0)/4096)
        elif var=='exact': Y=Y0+dy*(xc-X0)/dx
        half=T/2
        bot=math.floor(Y/16-half)+1
        top=bot+T
        for r in range(H):
            c=max(0,min(top,(r+1)*256)-max(bot,r*256))
            if c>0: out[(k,r)]=min(255,c)
    return out
random.seed(12); data=[]
for _ in range(40):
    m=random.uniform(-0.9,0.9); y0=random.uniform(4,8); x0=random.uniform(0.1,0.9)
    a=(x0,y0); b=(18.5,y0+(18.5-x0)*m)
    data.append((a,b,alphas('m %r %r\nl %r %r'%(a+b),1.0,W=20,H=24)))
for var in ('center','centerfloor','exact'):
    st=collections.Counter(); st2=collections.Counter()
    for a,b,A in data:
        M=model(a,b,20,24,var)
        rows=3 if abs((b[1]-a[1])/(b[0]-a[0]))>0.5 else 2
        for k in range(1,18):
            for r in range(24):
                v=M.get((k,r),0)
                if v or A[r][k]: st[A[r][k]==v]+=1
    print(var,dict(st))
