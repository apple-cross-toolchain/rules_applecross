import sys,subprocess,math,random; sys.path.insert(0,'/tmp/cg'); import raster, vprobe
S=4096; N=32
def run(polys):
    O=vprobe.obs_many(polys); bad=0; tot=0; ex=[]
    for P,o in zip(polys,O):
        m=raster.fill_path([P],N,N); acc=raster.fill_path_wind([P],N,N)
        for y in range(N):
            for x in range(N):
                if 0<o[y][x]<255 or m[y][x]!=o[y][x]:
                    tot+=1
                    if m[y][x]!=o[y][x]: bad+=1; ex.append((x,y,round(min(1,acc[y][x])*256,3),o[y][x]))
    return bad,tot,ex[:6]
random.seed(9)
# shape: far-bottom vertex, then two vertices in the same row (short edge), then far-top vertex ... a quadrilateral
for kind in ('chain-up','peak','side-short'):
    polys=[]
    for _ in range(40):
        r=random.randint(10,20); y1=r+random.uniform(0.05,0.95); y2=r+random.uniform(0.05,0.95)
        x1=random.uniform(12,18); x2=x1+random.uniform(-1.5,1.5)
        if kind=='chain-up':   # left side of a convex shape: bottom -> V1 -> V2 -> top, interior to the right
            lo,hi=min(y1,y2),max(y1,y2)
            P=[(x1+random.uniform(-3,3),2.3),(28.7,3.1),(27.9,29.4),(x2+random.uniform(-3,3),29.8),(x2,hi),(x1,lo)]
        elif kind=='peak':
            P=[(4.2,3.3),(27.6,2.9),(x2,y2),(x1,y1)]
        else:
            P=[(x1,y1),(x2,y2),(28.4,y2+random.uniform(5,9)),(29.1,y1-random.uniform(5,9))]
        polys.append(P)
    print(kind,run(polys))
