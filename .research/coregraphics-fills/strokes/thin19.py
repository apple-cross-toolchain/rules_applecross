import sys,math,random,itertools; sys.path.insert(0,'/tmp/cgw')
from thinsweep import alphas
random.seed(41); data=[]
while len(data)<14:
    m=random.uniform(-0.05,0.05); y0=random.uniform(4,8); x0=random.uniform(0.05,0.95)
    a=(x0,y0); b=(18.5,y0+(18.5-x0)*m)
    A=alphas('m %r %r\nl %r %r'%(a+b),1.0,W=20,H=24)
    X0,Y0,X1,Y1=[math.floor(v*4096+0.5) for v in (a[0],a[1],b[0],b[1])]
    dx,dy=X1-X0,Y1-Y0; s=int((dy*4096+dx//2)/dx); T=math.floor(256*math.hypot(1,dy/dx))
    if T!=256: continue
    bots={}
    for k in range(1,18):
        col=[(r,A[r][k]) for r in range(24) if A[r][k]>0]
        if len(col)==2 and col[0][1]+col[1][1]==T: bots[k]=col[0][0]*256+256-col[0][1]
    sols=[Aa for Aa in range(16*min(bots.values())-16*30,16*max(bots.values())+16*30) if all(((Aa+s*k)//16)==v for k,v in bots.items())]
    if sols: data.append((X0,Y0,s,min(sols),max(sols)))
print(len(data))
def check(f):
    return sum(1 for X0,Y0,s,lo,hi in data if lo<=f(X0,Y0,s)<=hi)
fl=lambda v,g: g*math.floor(v/g); rd=lambda v,g: g*math.floor(v/g+0.5)
cands={}
for yq in ('none','f16','r16','f256','r256'):
    for xq in ('none','f16','r16','f256','r256','f4096'):
        for c in range(-16,33):
            def f(X0,Y0,s,yq=yq,xq=xq,c=c):
                Y=Y0 if yq=='none' else (fl(Y0,int(yq[1:])) if yq[0]=='f' else rd(Y0,int(yq[1:])))
                X=X0 if xq=='none' else (fl(X0,int(xq[1:])) if xq[0]=='f' else rd(X0,int(xq[1:])))
                return math.floor(Y-2048+s*(2048-X)/4096)+c
            cands[(yq,xq,c)]=check(f)
for k,v in sorted(cands.items(),key=lambda kv:-kv[1])[:8]: print(k,v,'/',len(data))
for d in data: print(d)
