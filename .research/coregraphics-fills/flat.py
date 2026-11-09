import sys, math, pickle; sys.path.insert(0,'/tmp/cg'); import curves, raster
MID=None
def split(q):
    p0,p1,p2,p3=q
    if MID is None: m=lambda a,b:((a[0]+b[0])/2,(a[1]+b[1])/2)
    else: m=lambda a,b:(MID(a[0],b[0]),MID(a[1],b[1]))
    a=m(p0,p1); b=m(p1,p2); c=m(p2,p3); d=m(a,b); e=m(b,c); f=m(d,e)
    return (p0,a,d,f),(f,e,c,p3)
def meas(q,kind):
    p0,p1,p2,p3=q
    if kind=='dd':
        return max(math.hypot(p0[0]-2*p1[0]+p2[0],p0[1]-2*p1[1]+p2[1]),math.hypot(p1[0]-2*p2[0]+p3[0],p1[1]-2*p2[1]+p3[1]))
    if kind=='ddL1':
        return max(abs(p0[0]-2*p1[0]+p2[0])+abs(p0[1]-2*p1[1]+p2[1]),abs(p1[0]-2*p2[0]+p3[0])+abs(p1[1]-2*p2[1]+p3[1]))
    if kind=='ddinf':
        return max(abs(p0[0]-2*p1[0]+p2[0]),abs(p0[1]-2*p1[1]+p2[1]),abs(p1[0]-2*p2[0]+p3[0]),abs(p1[1]-2*p2[1]+p3[1]))
    if kind=='d2sum':
        return math.hypot(p0[0]-2*p1[0]+p2[0],p0[1]-2*p1[1]+p2[1])+math.hypot(p1[0]-2*p2[0]+p3[0],p1[1]-2*p2[1]+p3[1])
    if kind=='ctrl':   # max distance of control points from the chord's points at 1/3, 2/3
        return max(math.hypot(p1[0]-(2*p0[0]+p3[0])/3,p1[1]-(2*p0[1]+p3[1])/3),math.hypot(p2[0]-(p0[0]+2*p3[0])/3,p2[1]-(p0[1]+2*p3[1])/3))
    if kind=='ddsq':
        return max((p0[0]-2*p1[0]+p2[0])**2+(p0[1]-2*p1[1]+p2[1])**2,(p1[0]-2*p2[0]+p3[0])**2+(p1[1]-2*p2[1]+p3[1])**2)
    if kind=='dist':
        dx,dy=p3[0]-p0[0],p3[1]-p0[1]; L=math.hypot(dx,dy) or 1e-12
        return max(abs((p[0]-p0[0])*dy-(p[1]-p0[1])*dx)/L for p in (p1,p2))
MINDEPTH=0
def flat_rec(q,kind,tol,uniform,depth=0):
    if uniform:
        # find the uniform depth: halve until every piece is flat
        n=1; pieces=[q]
        while any(meas(x,kind)>tol for x in pieces) and n<1024:
            pieces=[h for x in pieces for h in split(x)]; n*=2
        return [x[0] for x in pieces]
    if (meas(q,kind)<=tol and depth>=MINDEPTH) or depth>12: return [q[0]]
    a,b=split(q); return flat_rec(a,kind,tol,False,depth+1)+flat_rec(b,kind,tol,False,depth+1)
def score(D,kind,tol,uniform):
    N=curves.N; bad=0;tot=0
    for s,rows in D:
        p0,a,b,p3,c,d=s
        P=flat_rec((p0,a,b,p3),kind,tol,uniform)+flat_rec((p3,c,d,p0),kind,tol,uniform)
        m=raster.fill([P],N,N)
        for y in range(N):
            for x in range(N):
                if rows[y][x] in (0,255) and m[y][x]==rows[y][x]: continue
                tot+=1; bad+=m[y][x]!=rows[y][x]
    return bad,tot
