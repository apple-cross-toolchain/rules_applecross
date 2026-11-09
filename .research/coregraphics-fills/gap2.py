import sys,subprocess,random,math; sys.path.insert(0,'/tmp/cg'); import raster
S=4096; W,H=24,32
def endx(b,t):
    rows=raster.stepper(b[1],b[0],t[1],t[0]); mr=raster.q((t[0]-b[0])/(t[1]-b[1]),'trunc')
    if rows: ky,kx=rows[-1]; return kx+raster.q((t[1]-ky)*mr,'floor')
    return b[0]+raster.q((t[1]-b[1])*mr,'floor')
def gen(seed,count=300):
    random.seed(seed); cases=[]
    for _ in range(count):
        a=(random.uniform(3,8),random.uniform(1.5,3)); b=(a[0]+random.uniform(-3,5),random.uniform(27,29.5)); n=random.randint(5,40)
        left=[(a[0]+(b[0]-a[0])*i/n, a[1]+(b[1]-a[1])*i/n) for i in range(n+1)]
        cases.append(([a,(20.2,2.3),(19.6,28.9),b]+list(reversed(left[1:-1])),left))
    inp=''.join('m %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\nf\nn\n' for P,_ in cases)
    O=[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run(['./cov',str(W),str(H)],input=inp,capture_output=True,text=True).stdout.split('--\n')[:len(cases)]]
    out=[]
    for (P,left),o in zip(cases,O):
        acc=raster.fill_path_wind([P],W,H)
        L=[(raster.vq(x),raster.vq(y)) for x,y in left]
        for i in range(1,len(L)-1):
            V=L[i]; y=math.floor(V[1])
            if sum(1 for p in L if math.floor(p[1])==y)!=1: continue
            gap=(endx(L[i-1],V)-V[0])*S
            for x in range(0,14):
                if 0<o[y][x]<255 and x<=V[0]+2 and x>=V[0]-2:
                    raw=min(1,max(0,acc[y][x]))*256
                    out.append(dict(gap=gap,fy=V[1]-y,fx=V[0]-x,dx=(x-math.floor(V[0])),lo=o[y][x]-raw,hi=o[y][x]+1-raw,miss=math.floor(raw+1e-9)!=o[y][x],slope=(L[i+1][0]-L[i-1][0])/(L[i+1][1]-L[i-1][1])))
    return out
