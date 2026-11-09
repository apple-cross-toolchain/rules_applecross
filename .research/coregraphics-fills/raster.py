"""raster.py: model of CoreGraphics' general-path fill (alpha only), black-box.
Each edge: vertices converted to float32, then rounded half up to 1/4096; from the edge's lower endpoint two DDAs with slopes truncated to 1/4096:
x at each integer y (rows) and y at each integer x (columns); first crossing = vertex + floor4096(frac * slope).
The merged crossings make a polyline; pieces between row crossings belong to that row; per pixel the signed
area is accumulated as the integral of clamp(x - X, 0, 1) dy (Green), alpha = floor(256 |a|)."""
import math, struct
S=4096
def f32(v): return struct.unpack('f',struct.pack('f',v))[0]
def vq(v): return math.floor(f32(v)*S+0.5)/S
def q(v,mode='round'):
    return {'round':math.floor(v*S+0.5),'trunc':math.trunc(v*S),'floor':math.floor(v*S),'ceil':math.ceil(v*S)}[mode]/S
def stepper(pa,pb,qa,qb):
    # values of b at integers strictly between pa and qa, stepping from (pa,pb)
    if pa==qa: return []
    dirn=1 if qa>pa else -1
    m=q((qb-pb)/abs(qa-pa),'trunc')
    k0=math.floor(pa)+1 if dirn>0 else math.ceil(pa)-1
    B0=pb+q(abs(k0-pa)*m,'floor')
    out=[]; k=k0; i=0
    while (k<qa if dirn>0 else k>qa): out.append((k,B0+i*m)); k+=dirn; i+=1
    return out
def edge_pieces(p0,p1,vy=None):
    """pieces (pa, pb, row) along the edge in path order"""
    (x0,y0),(x1,y1)=p0,p1
    if y0==y1: return []
    if x0==x1 and VERT:
        x0=x1=(math.ceil(x0*S)-1)/S
    up=y1>y0
    (bx,by),(tx,ty)=((x0,y0),(x1,y1)) if up else ((x1,y1),(x0,y0))
    rows=[(y,x) for y,x in stepper(by,bx,ty,tx)]            # x at integer y, bottom to top
    cols=[(x,y) for x,y in stepper(bx,by,tx,ty)]            # y at integer x, from the bottom vertex
    # order along the edge from the bottom: parameter by true position
    L=[(0.0,(bx,by),None)]
    for y,x in rows: L.append(((y-by)/(ty-by),(x,y),'row'))
    for x,y in cols: L.append(((x-bx)/(tx-bx),(x,y),'col'))
    if END=='R' and rows:
        mr=q((tx-bx)/(ty-by),'trunc'); ky,kx=rows[-1]; tx2=kx+q((ty-ky)*mr,'floor'); L.append((1.0,(tx2,ty),None))
    elif END=='R' and not rows:
        mr=q((tx-bx)/(ty-by),'trunc'); L.append((1.0,(bx+q((ty-by)*mr,'floor'),ty),None))
    elif END=='C' and cols:
        mc=q((ty-by)/abs(tx-bx),'trunc'); kx,ky=cols[-1]; L.append((1.0,(tx,ky+q(abs(tx-kx)*mc,'floor')),None))
    else: L.append((1.0,(tx,ty),None))
    L.sort(key=lambda z:(z[0], 0 if z[2]=='row' else 1))
    pts=[p for _,p,_ in L]; kinds=[k for _,_,k in L]
    pieces=[]; row=math.floor(by)
    for i in range(len(pts)-1):
        if kinds[i]=='row': row=int(pts[i][1])
        pieces.append((pts[i],pts[i+1],row))
    if not up: pieces=[(b,a,r) for a,b,r in reversed(pieces)]
    return pieces
PQ=lambda v: v
SNAP=None
VERT=True
CLIP=None
def sh_clip(P,x0,y0,x1,y1):
    def side(P,inside,inter):
        out=[]
        for i in range(len(P)):
            a,b=P[i-1],P[i]
            if inside(b):
                if not inside(a): out.append(inter(a,b))
                out.append(b)
            elif inside(a): out.append(inter(a,b))
        return out
    def ix(X): return lambda a,b:(X,a[1]+(b[1]-a[1])*(X-a[0])/(b[0]-a[0]))
    def iy(Y): return lambda a,b:(a[0]+(b[0]-a[0])*(Y-a[1])/(b[1]-a[1]),Y)
    for inside,inter in ((lambda p:p[0]>=x0,ix(x0)),(lambda p:p[0]<=x1,ix(x1)),(lambda p:p[1]>=y0,iy(y0)),(lambda p:p[1]<=y1,iy(y1))):
        P=side(P,inside,inter)
        if not P: return []
    return P
END=None
def snap_lo(y): return math.floor(y*256)/256
def snap_hi(y): return (math.ceil(y*256)-1)/256
def cut(a,b,ylo,yhi):
    (x0,y0),(x1,y1)=a,b
    if y0==y1: return None
    lo,hi=min(y0,y1),max(y0,y1)
    if hi<=ylo or lo>=yhi: return None
    def at(y): return (x0+(x1-x0)*(y-y0)/(y1-y0),y)
    if y0<ylo: a=at(ylo)
    if y0>yhi: a=at(yhi)
    if y1<ylo: b=at(ylo)
    if y1>yhi: b=at(yhi)
    return a,b
def snap_pieces(p0,p1):
    if SNAP is None: return edge_pieces(p0,p1)
    lo,hi=min(p0[1],p1[1]),max(p0[1],p1[1])
    if SNAP=='A':
        q0=(p0[0],snap_lo(p0[1]) if p0[1]==lo else snap_hi(p0[1])); q1=(p1[0],snap_lo(p1[1]) if p1[1]==lo else snap_hi(p1[1]))
        return edge_pieces(q0,q1)
    ylo,yhi=snap_lo(lo),snap_hi(hi)
    out=[]
    for a,b,row in edge_pieces(p0,p1):
        c=cut(a,b,ylo,yhi)
        if c: out.append((c[0],c[1],row))
    return out
def seg_I(p,qq,X):
    (x0,y0),(x1,y1)=p,qq
    if y0==y1: return 0.0
    if x0<=X and x1<=X: return (x0+x1)/2*(y1-y0)
    if x0>=X and x1>=X: return X*(y1-y0)
    t=(X-x0)/(x1-x0); ym=y0+t*(y1-y0)
    return seg_I((x0,y0),(X,ym),X)+seg_I((X,ym),(x1,y1),X)
def fill_raw(polys,W,H):
    acc=[[0.0]*W for _ in range(H)]
    for P in polys:
        if CLIP is not None:
            m=CLIP if isinstance(CLIP,(int,float)) else 0
            box={'y':(-1e9,0,1e9,H),'x':(0,-1e9,W,1e9),'xy':(0,0,W,H)}.get(CLIP if isinstance(CLIP,str) else '', (-m,-m,W+m,H+m)) if not isinstance(CLIP,tuple) else CLIP
            P=sh_clip(P,*box)
            if not P: continue
        P=[(vq(x),vq(y)) for x,y in P]
        for i in range(len(P)):
            for a,b,row in snap_pieces(P[i],P[(i+1)%len(P)]):
                if not 0<=row<H: continue
                lo=max(0,math.floor(min(a[0],b[0]))); hi=min(W-1,math.floor(max(a[0],b[0])))
                for x in range(lo,hi+1):
                    acc[row][x]+=PQ(seg_I(a,b,x+1)-seg_I(a,b,x))
                # pixels left of the piece get its full height
                d=b[1]-a[1]
                for x in range(0,min(lo,W)): acc[row][x]+=d
    return acc
def fill(polys,W,H):
    return [[max(0,min(255,math.floor(abs(v)*256+1e-9))) for v in r] for r in fill_raw(polys,W,H)]
