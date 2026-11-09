import sys, math, random, subprocess, collections; sys.path.insert(0,'/tmp/cg'); import raster
N=24
def bez(p0,p1,p2,p3,t):
    u=1-t
    return (u**3*p0[0]+3*u*u*t*p1[0]+3*u*t*t*p2[0]+t**3*p3[0], u**3*p0[1]+3*u*u*t*p1[1]+3*u*t*t*p2[1]+t**3*p3[1])
def gen(seed,n=40,scale=1.0):
    random.seed(seed); C=[]
    for _ in range(n):
        c=(12,12); r=random.uniform(2,10)*scale
        # a closed shape: two cubics forming a blob
        a=random.uniform(0,2*math.pi)
        p0=(c[0]+r*math.cos(a),c[1]+r*math.sin(a)); p3=(c[0]-r*math.cos(a),c[1]-r*math.sin(a))
        rnd=lambda: (c[0]+random.uniform(-r,r),c[1]+random.uniform(-r,r))
        C.append((p0,rnd(),rnd(),p3,rnd(),rnd()))
    inp=''.join('m %r %r\nc %r %r %r %r %r %r\nc %r %r %r %r %r %r\nh\nf\nn\n'%(s[0]+s[1]+s[2]+s[3]+s[4]+s[5]+s[0]) for s in C)
    out=subprocess.run(['./cov',str(N),str(N)],input=inp,capture_output=True,text=True).stdout.split('--\n')
    return [(s,[list(map(int,r.split())) for r in o.strip().split('\n')][::-1]) for s,o in zip(C,out)]
def flatten(s,nseg):
    p0,a,b,p3,c,d=s
    pts=[bez(p0,a,b,p3,i/nseg[0]) for i in range(nseg[0])]+[bez(p3,c,d,p0,i/nseg[1]) for i in range(nseg[1])]
    return pts
def score(D,segf):
    bad=0;tot=0
    for s,rows in D:
        p0,a,b,p3,c,d=s
        P=flatten(s,(segf(p0,a,b,p3),segf(p3,c,d,p0)))
        m=raster.fill([P],N,N)
        for y in range(N):
            for x in range(N):
                if rows[y][x] in (0,255) and m[y][x]==rows[y][x]: continue
                tot+=1; bad+=m[y][x]!=rows[y][x]
    return bad,tot
