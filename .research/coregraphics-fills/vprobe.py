import sys,subprocess,math; sys.path.insert(0,'/tmp/cg'); import raster
S=4096; N=32
def obs_many(polys):
    inp=''.join('m %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\nf\nn\n' for P in polys)
    out=subprocess.run(['./cov',str(N),str(N)],input=inp,capture_output=True,text=True).stdout.split('--\n')
    return [[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in out[:len(polys)]]
def probe(V,A,B,axis,steps=96):
    polys=[]
    for k in range(steps):
        v=list(V); v[axis]=round(V[axis]*S)/S+k/S
        polys.append([tuple(v),A,B])
    O=obs_many(polys)
    px,py=math.floor(V[0]),math.floor(V[1])
    pix=[(px+dx,py) for dx in (-1,0,1)]
    best=[]
    for shift in range(-4,5):
        for sx in range(-4,5):
            bad=0
            for k,(P,o) in enumerate(zip(polys,O)):
                v=(P[0][0]+(sx/S),P[0][1]+(shift/S))
                m=raster.fill_path([[v,A,B]],N,N)
                bad+=sum(1 for (x,y) in pix if m[y][x]!=o[y][x])
            best.append((bad,sx,shift))
    best.sort(); return best[:4], sum(1 for b in best if b[1]==0 and b[2]==0 for _ in [0]) and [b for b in best if b[1]==0 and b[2]==0][0][0]
if __name__=='__main__':
    for V,A,B in (((10.37,8.21),(4.1,28.3),(25.7,27.9)),((15.61,10.73),(2.2,13.9),(29.4,14.8)),((12.13,5.52),(11.0,29.1),(14.3,29.6))):
        for axis in (1,0):
            print(V,'axis',axis,probe(V,A,B,axis))
