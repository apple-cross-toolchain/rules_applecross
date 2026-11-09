import sys,math,random,collections; sys.path.insert(0,'/tmp/cgw')
from thinsweep import alphas
def model(a,b,W,H,k0mode='floor',c=0.375):
    (x0,y0),(x1,y1)=sorted([a,b])
    X0,Y0,X1,Y1=[math.floor(v*4096+0.5) for v in (x0,y0,x1,y1)]
    dx,dy=X1-X0,Y1-Y0; s=int((dy*4096+dx//2)/dx); m=dy/dx*4096
    T=math.floor(256*math.hypot(1,dy/dx))
    k0=math.floor(X0/4096) if k0mode=='floor' else 0
    desc = Y1<Y0
    if desc:
        sr=int((-dy*4096+dx//2)/dx)   # slope going leftwards (positive)
        k1=math.floor((X1-1)/4096)
    out={}
    for k in range(W):
        if not (x0<=k and k+1<=x1): continue
        if desc and k0mode!='nodesc':
            Y=Y1+(-m)*(X1/4096-(k1+1-c))+sr*(k1-k)
        else:
            Y=Y0+m*(k0+c-X0/4096)+s*(k-k0)
        bot=math.floor((Y-2032)/16); top=bot+T
        for r in range(H):
            v=max(0,min(top,(r+1)*256)-max(bot,r*256))
            if v>0: out[(k,r)]=min(255,v)
    return out
if __name__=='__main__':
    random.seed(61); data=[]
    for _ in range(40):
        m=random.uniform(-0.2,0.2); y0=random.uniform(4,8); x0=random.uniform(0.05,2.95)
        a=(x0,y0); b=(18.5,y0+(18.5-x0)*m)
        data.append((a,b,alphas('m %r %r\nl %r %r'%(a+b),1.0,W=20,H=24)))
    for k0m in ('floor','nodesc'):
        st=collections.Counter()
        for a,b,A in data:
            M=model(a,b,20,24,k0m)
            for k in range(4,18):
                for r in range(24):
                    v=M.get((k,r),0)
                    if v or A[r][k]: st[A[r][k]==v]+=1
        print(k0m,dict(st))
