import sys, math, random, subprocess, pickle; sys.path.insert(0,'/tmp/cg'); import raster, curves
N=40; K=0.5522847498
def arcs(cx,cy,rx,ry):
    P=[(cx+rx,cy),(cx,cy+ry),(cx-rx,cy),(cx,cy-ry)]
    T=[(0,ry),(-rx,0),(0,-ry),(rx,0)]
    out=[]
    for i in range(4):
        a=P[i]; b=P[(i+1)%4]; ta=T[i]; tb=T[(i+1)%4]
        out.append((a,(a[0]+K*ta[0],a[1]+K*ta[1]),(b[0]-K*tb[0],b[1]-K*tb[1]),b))
    return out
def gen(seed,n=40):
    random.seed(seed); E=[]
    for _ in range(n):
        rx=random.uniform(0.5,18); ry=random.uniform(0.5,18)
        E.append((random.uniform(19,21),random.uniform(19,21),rx,ry))
    inp=''
    for e in E:
        A=arcs(*e); inp+='m %r %r\n'%A[0][0]+''.join('c %r %r %r %r %r %r\n'%(a[1]+a[2]+a[3]) for a in A)+'h\nf\nn\n'
    out=subprocess.run(['./cov',str(N),str(N)],input=inp,capture_output=True,text=True).stdout.split('--\n')
    return [(e,[list(map(int,r.split())) for r in o.strip().split('\n')][::-1]) for e,o in zip(E,out)]
def score(e,rows,n):
    A=arcs(*e); P=[]
    for a in A: P+=[curves.bez(*a,i/n) for i in range(n)]
    m=raster.fill([P],N,N)
    return sum(1 for y in range(N) for x in range(N) if m[y][x]!=rows[y][x]), sum(1 for y in range(N) for x in range(N) if 0<rows[y][x]<255)
