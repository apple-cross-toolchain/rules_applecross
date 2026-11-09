import subprocess, math, sys, random
S=4096; W=400; H=400
def img(P):
    inp='m %r %r\n'%P[0]+''.join('l %r %r\n'%q for q in P[1:])+'h\nf\n'
    o=subprocess.run(['./cov',str(W),str(H)],input=inp,capture_output=True,text=True).stdout.split('--\n')[0]
    return [list(map(int,r.split())) for r in o.strip().split('\n')][::-1]
def drift(x0,y0,x1,y1, up=True):
    P=[(x0,y0),(x1,y1),(-5.0,y1),(-5.0,y0)] if up else [(x1,y1),(x0,y0),(-5.0,y0),(-5.0,y1)][::-1]
    if not up: P=[(x1,y1),(x0,y0),(-5.0,y0),(-5.0,y1)]   # same region, reversed winding
    im=img(P); ys=[];ds=[]
    for y in range(math.ceil(y0)+1,math.floor(y1)-1):
        tot=sum(1 if v==255 else (v+0.5)/256 for v in im[y]); xm=lambda yy: x0+(x1-x0)*(yy-y0)/(y1-y0)
        ys.append(y); ds.append(tot-(xm(y)+xm(y+1))/2)
    n=len(ys); my=sum(ys)/n; md=sum(ds)/n
    b=sum((y-my)*(d-md) for y,d in zip(ys,ds))/sum((y-my)**2 for y in ys)
    return b, md
random.seed(int(sys.argv[1]) if len(sys.argv)>1 else 1)
for _ in range(10):
    y0=random.uniform(3,10); y1=random.uniform(H-10,H-3); m=random.uniform(-0.99,0.99)
    x0=200-m*(y1-y0)/2; x1=x0+m*(y1-y0); mt=(x1-x0)/(y1-y0)
    bu,_=drift(x0,y0,x1,y1,True); bd,_=drift(x0,y0,x1,y1,False)
    print('m %+.5f up %+.7f down %+.7f | trunc %+.7f floor %+.7f round %+.7f | m*4096 frac %.3f'%(mt,bu,bd,math.trunc(mt*S)/S-mt,math.floor(mt*S)/S-mt,round(mt*S)/S-mt,(mt*S)%1))
