import sys,random,subprocess,math,pickle; sys.path.insert(0,'/tmp/cg'); import raster
N=16
def gen(seed,n=60):
    random.seed(seed); out=[]
    for _ in range(n):
        polys=[[(random.uniform(1,15),random.uniform(1,15)) for _ in range(3)] for _ in range(2)]
        out.append(polys)
    return out
def render(G,rule):
    inp=''
    for polys in G:
        for P in polys: inp+='m %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\n'
        inp+=('f' if rule=='nz' else 'e')+'\nn\n'
    out=subprocess.run(['./cov',str(N),str(N)],input=inp,capture_output=True,text=True).stdout.split('--\n')
    return [[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in out[:len(G)]]
def model(polys,rule):
    # accumulate all subpaths into one signed buffer
    acc=[[0.0]*N for _ in range(N)]
    save=raster.BREAKS
    allP=[(raster.vq(x),raster.vq(y)) for P in polys for x,y in P]
    tot=[[0.0]*N for _ in range(N)]
    import copy
    raws=[]
    # one buffer: fill_raw sums per polygon already into acc when given several
    raw=raster.fill_path_raw(polys,N,N)
    res=[]
    for r in raw:
        row=[]
        for v in r:
            if rule=='nz': c=min(1.0,abs(v))
            else: f=abs(v)%2; c=f if f<=1 else 2-f
            row.append(max(0,min(255,math.floor(c*256+1e-9))))
        res.append(row)
    return res
if __name__=='__main__':
    G=gen(1)
    for rule in ('nz','eo'):
        obs=render(G,rule); bad=0; tot=0
        for polys,o in zip(G,obs):
            m=model(polys,rule)
            bad+=sum(1 for y in range(N) for x in range(N) if m[y][x]!=o[y][x]); tot+=sum(1 for y in range(N) for x in range(N) if 0<o[y][x]<255)
        print(rule,bad,tot)
