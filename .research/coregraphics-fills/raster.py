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
def edge_pieces(p0,p1,vy=None,start_x=None):
    """pieces (pa, pb, row) along the edge in path order"""
    (x0,y0),(x1,y1)=p0,p1
    if y0==y1: return []
    if x0==x1 and VERT:
        if VERT=='snapdown' or VERT=='snapup':
            # rect rule: the edge that starts a span snaps to its 1/256 step: floor((x*8192+1)/32)/256
            starts=(y1<y0) if VERT=='snapdown' else (y1>y0)
            if starts: x0=x1=math.floor((x0*8192+1)/32)/256
        elif VERT=='none': pass
        else: x0=x1=(math.ceil(x0*S)-1)/S
    up=y1>y0
    (bx,by),(tx,ty)=((x0,y0),(x1,y1)) if up else ((x1,y1),(x0,y0))
    if start_x is not None: bx=start_x
    rows=[(y,x) for y,x in stepper(by,bx,ty,tx)]            # x at integer y, bottom to top
    cols=[(x,y) for x,y in stepper(bx,by,tx,ty)]            # y at integer x, from the bottom vertex
    # the edge's end: the row stepper's x at the top (END='R'), else the vertex
    if RESTART and BREAKS:
        # re-anchor the row stepper at each band boundary inside the edge
        mr=q((tx-bx)/(ty-by),'trunc'); cur=(by,bx); newrows=[]; bands=[yv for yv in BREAKS if by<yv<ty]
        ks=[y for y,x in rows]; bi=0; i=0; anchor=(by,bx); first=True
        for k in ks:
            while bi<len(bands) and bands[bi]<k:
                # band point stepped from the last crossing (or anchor)
                ly,lx=(newrows[-1] if newrows else anchor)
                anchor=(bands[bi],lx+q((bands[bi]-ly)*mr,'floor')); newrows_anchor=True; bi+=1; first=True
            ay,ax=anchor
            if newrows and not first: x=newrows[-1][1]+mr
            else: x=ax+q((k-ay)*mr,'floor')
            newrows.append((k,x)); first=False
        rows=newrows
    if END=='R':
        mr=q((tx-bx)/(ty-by),'trunc')
        if rows: ky,kx=rows[-1]; tx_end=kx+q((ty-ky)*mr,ENDQ)
        else: tx_end=bx+q((ty-by)*mr,ENDQ)
    else: tx_end=tx
    ty_end=ty
    if END=='RC' or END=='RCf':
        mr=q((tx-bx)/(ty-by),'trunc')
        if abs(tx-bx)<=abs(ty-by):
            if rows: ky,kx=rows[-1]; tx_end=kx+q((ty-ky)*mr,ENDQ)
            else: tx_end=bx+q((ty-by)*mr,ENDQ)
        else:
            mc=q((ty-by)/abs(tx-bx),'trunc'); tx_end=tx
            if cols: kx,ky=cols[-1]; ty_end=ky+q(abs(tx-kx)*mc,ENDQ)
            else: ty_end=by+q(abs(tx-bx)*mc,ENDQ)
    global LAST_END
    LAST_END=tx_end
    colmap=dict(cols)
    ends=[(bx,by)]+[(x,y) for y,x in rows]+[(tx_end,ty_end)]
    if BANDS is not None and BREAKS:
        mr=q((tx-bx)/(ty-by),'trunc'); extra=[]
        for yv in BREAKS:
            if by<yv<ty:
                if BANDS=='exact': xv=q(bx+(tx-bx)*(yv-by)/(ty-by))
                elif BANDS=='exactfloor': xv=q(bx+(tx-bx)*(yv-by)/(ty-by),'floor')
                elif BANDS in ('interp','interpfloor','interpround'):
                    seq=[(by,bx)]+[(y,x) for y,x in rows]+[(ty,tx_end)]
                    for (ya,xa),(yb,xb) in zip(seq,seq[1:]):
                        if ya<=yv<=yb: break
                    xv=xa+(xb-xa)*(yv-ya)/(yb-ya) if yb!=ya else xa
                    if BANDS=='interpfloor': xv=q(xv,'floor')
                    if BANDS=='interpround': xv=q(xv,'round')
                elif BANDS=='fromvertex':
                    xv=bx+q((yv-by)*mr,BANDQ)
                elif BANDS=='prevpoint':
                    cand=[(y,x) for y,x in rows if y<=yv]+[(e[1],e[0]) for e in extra]
                    cand=[c for c in cand if c[0]<=yv]
                    ky,kx=max(cand) if cand else (by,bx)
                    xv=kx+q((yv-ky)*mr,BANDQ)
                else:
                    prev=[(y,x) for y,x in rows if y<=yv]
                    ky,kx=prev[-1] if prev else (by,bx)
                    bq=BANDQ
                    if BANDROLE is not None and EDGEROLE is not None:
                        bq=BANDROLE[0] if EDGEROLE>0 else BANDROLE[1]
                    xv=kx+q((yv-ky)*mr,bq)
                extra.append((xv,yv))
        ends=sorted(ends+extra,key=lambda p:p[1])
    pieces=[]
    for i in range(len(ends)-1):
        A,B=ends[i],ends[i+1]; row=math.floor(A[1])
        lo,hi=sorted((A[0],B[0]))
        ks=[k for k in range(math.floor(lo)+1,math.ceil(hi)) if k in colmap]
        ks.sort(reverse=B[0]<A[0])
        pts=[A]+[(k,colmap[k]) for k in ks]+[B]
        for j in range(len(pts)-1): pieces.append((pts[j],pts[j+1],row))
    if not up: pieces=[(b,a,r) for a,b,r in reversed(pieces)]
    return pieces
PQ=lambda v: v
SNAP=None
VERT="none"
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
END="R"
SWEEP="right"
BANDS="stepped"
RESTART=False
CHAIN=False
BANDQ2=None
BANDQ2_MULTI=True
CHAIN_SET=None
LAST_END=None
BANDROLE=None
EDGEROLE=None
BANDQ_AREA=None
ENDQ="floor"
BANDQ="floor"
BREAKS=[]
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
        global BREAKS
        BREAKS=sorted({y for x,y in P})
        bandacc={}
        for i in range(len(P)):
            for a,b,row in snap_pieces(P[i],P[(i+1)%len(P)]):
                if not 0<=row<H: continue
                lo=max(0,math.floor(min(a[0],b[0]))); hi=min(W-1,math.floor(max(a[0],b[0])))
                d=b[1]-a[1]
                if BANDQ_AREA is not None:
                    ym=(a[1]+b[1])/2
                    bk=(row,)+tuple(1 if ym>yv else 0 for yv in BREAKS if row<yv<row+1)
                    accb=bandacc.setdefault(bk,[0.0]*W)
                    for x in range(lo,hi+1): accb[x]+=seg_I(a,b,x+1)-seg_I(a,b,x)-d
                    for x in range(max(hi+1,0),W): accb[x]-=d
                    continue
                if SWEEP=='right':
                    # cover carried to the right, as a left-to-right sweep does
                    for x in range(lo,hi+1):
                        acc[row][x]+=PQ(seg_I(a,b,x+1)-seg_I(a,b,x))-d
                    for x in range(max(hi+1,0),W): acc[row][x]-=d
                else:
                    for x in range(lo,hi+1):
                        acc[row][x]+=PQ(seg_I(a,b,x+1)-seg_I(a,b,x))
                    for x in range(0,min(lo,W)): acc[row][x]+=d
        for bk,accb in bandacc.items():
            for x in range(W): acc[bk[0]][x]+=BANDQ_AREA(abs(accb[x]))
    return acc
def fill(polys,W,H):
    return [[max(0,min(255,math.floor(abs(v)*256+1e-9))) for v in r] for r in fill_raw(polys,W,H)]
def fill_path_raw(subpaths,W,H):
    """one path of several subpaths: band boundaries at every vertex of the path; one signed accumulator"""
    global BREAKS
    acc=[[0.0]*W for _ in range(H)]
    Ps=[]
    for P in subpaths:
        if CLIP is not None:
            P=sh_clip(P,*CLIP)
            if not P: continue
        Ps.append([(vq(x),vq(y)) for x,y in P])
    BREAKS=sorted({y for P in Ps for x,y in P})
    for P in Ps:
        for i in range(len(P)):
            for a,b,row in snap_pieces(P[i],P[(i+1)%len(P)]):
                if not 0<=row<H: continue
                lo=max(0,math.floor(min(a[0],b[0]))); hi=min(W-1,math.floor(max(a[0],b[0])))
                d=b[1]-a[1]
                for x in range(lo,hi+1): acc[row][x]+=seg_I(a,b,x+1)-seg_I(a,b,x)-d
                for x in range(max(hi+1,0),W): acc[row][x]-=d
    return acc
def _xat(p0,p1,y):
    return p0[0]+(p1[0]-p0[0])*(y-p0[1])/(p1[1]-p0[1])
def fill_path_wind(subpaths,W,H,rule='nz',inter=True):
    """bands at every vertex y and every edge intersection y; within each band edges are ordered by x and
    the winding rule picks the span boundaries; only those contribute (opening: +area right, closing: -)."""
    global BREAKS
    Ps=[]
    for P in subpaths:
        if CLIP is not None:
            P=sh_clip(P,*CLIP)
            if not P: continue
        Ps.append([(vq(x),vq(y)) for x,y in P])
    BREAKS=sorted({y for P in Ps for x,y in P})
    edges=[(P[i],P[(i+1)%len(P)]) for P in Ps for i in range(len(P)) if P[i][1]!=P[(i+1)%len(P)][1]]
    def sgn_area(P): return 1 if sum(P[i-1][0]*P[i][1]-P[i][0]*P[i-1][1] for i in range(len(P)))>0 else -1
    orient=[sgn_area(P) for P in Ps for i in range(len(P)) if P[i][1]!=P[(i+1)%len(P)][1]]
    cuts=set(BREAKS)
    if inter:
        for i in range(len(edges)):
            for j in range(i+1,len(edges)):
                (a,b),(c,d)=edges[i],edges[j]
                den=(b[0]-a[0])*(d[1]-c[1])-(b[1]-a[1])*(d[0]-c[0])
                if den==0: continue
                t=((c[0]-a[0])*(d[1]-c[1])-(c[1]-a[1])*(d[0]-c[0]))/den
                u=((c[0]-a[0])*(b[1]-a[1])-(c[1]-a[1])*(b[0]-a[0]))/den
                if 1e-12<t<1-1e-12 and 1e-12<u<1-1e-12: cuts.add(a[1]+t*(b[1]-a[1]))
    cuts=sorted(cuts)
    acc=[[0.0]*W for _ in range(H)]
    # pieces per edge, upward-oriented, with direction
    per=[]
    global EDGEROLE
    ends_at={}   # (vertex) -> stepped top-end x of the edge whose top is that vertex
    order=sorted(range(len(edges)),key=lambda i:min(edges[i][0][1],edges[i][1][1]))
    tmp={}
    for ei in order:
        p0,p1=edges[ei]
        dirn=1 if p1[1]>p0[1] else -1
        EDGEROLE=dirn*-orient[ei]
        bottom=p0 if dirn>0 else p1; top=p1 if dirn>0 else p0
        sx=ends_at.get(bottom) if (CHAIN and (CHAIN_SET is None or bottom in CHAIN_SET)) else None
        tmp[ei]=edge_pieces(p0,p1,start_x=sx)
        ends_at[top]=LAST_END
    for ei,(p0,p1) in enumerate(edges):
        dirn=1 if p1[1]>p0[1] else -1
        pcs=tmp[ei]
        up=[(a,b,r) if b[1]>=a[1] or True else None for a,b,r in pcs]
        # orient each piece upward along the edge (bottom to top)
        if dirn<0: pcs=[(b,a,r) for a,b,r in reversed(pcs)]
        per.append((ei,dirn,p0 if dirn>0 else p1,p1 if dirn>0 else p0,pcs))
    icuts=sorted(c for c in cuts if c not in set(BREAKS))
    def clipy(a,b,lo,hi):
        # clip piece a->b to lo<=y<=hi (lo/hi may be None for unbounded)
        (x0,y0),(x1,y1)=a,b
        if y0==y1:
            return (a,b) if (lo is None or y0>=lo) and (hi is None or y0<=hi) else None
        def at(yy): return (x0+(x1-x0)*(yy-y0)/(y1-y0),yy)
        ya,yb=min(y0,y1),max(y0,y1)
        if (lo is not None and yb<=lo) or (hi is not None and ya>=hi): return None
        if lo is not None and y0<lo: a=at(lo)
        if hi is not None and y1>hi: b=at(hi)
        return a,b
    for y in range(H):
        vb=[y]+[c for c in BREAKS if y<c<y+1]+[y+1]
        for bi in range(len(vb)-1):
            v0,v1=vb[bi],vb[bi+1]
            ic=[c for c in icuts if v0<c<v1]
            sub=[v0]+ic+[v1]
            for si in range(len(sub)-1):
                s0,s1=sub[si],sub[si+1]; ym=(s0+s1)/2
                lo_c=s0 if si>0 else None; hi_c=s1 if si<len(sub)-2 else None
                act=[]
                for ei,dirn,lo_,hi_,pcs in per:
                    if not (lo_[1]<ym<hi_[1]): continue
                    act.append((_xat(lo_,hi_,ym),ei,dirn,pcs))
                act.sort()
                w=0
                if BANDQ2 is not None: bandbuf=[0.0]*W
                for x0,ei,dirn,pcs in act:
                    before=w; w+=dirn
                    if rule=='nz': role=1 if before==0 and w!=0 else -1 if before!=0 and w==0 else 0
                    else: role=1 if before%2==0 else -1
                    if not role: continue
                    for a_,b_,r in pcs:
                        if r!=y: continue
                        m=(a_[1]+b_[1])/2
                        if not (v0<=m<v1 or (bi==len(vb)-2 and m>=v1) or (bi==0 and m<v0)): continue
                        cpd=clipy(a_,b_,lo_c,hi_c)
                        if not cpd: continue
                        a,b=cpd; d=b[1]-a[1]
                        lo=max(0,math.floor(min(a[0],b[0]))); hi=min(W-1,math.floor(max(a[0],b[0])))
                        tgt=acc[y] if BANDQ2 is None else bandbuf
                        for x in range(lo,hi+1): tgt[x]+=role*(d-(seg_I(a,b,x+1)-seg_I(a,b,x)))
                        for x in range(max(hi+1,0),W): tgt[x]+=role*d
                if BANDQ2 is not None:
                    multi=len(vb)>2 or len(sub)>2
                    for x in range(W):
                        if bandbuf[x]: acc[y][x]+=BANDQ2(bandbuf[x]) if (multi or not BANDQ2_MULTI) else bandbuf[x]; bandbuf[x]=0.0
    return acc
def fill_path(subpaths,W,H,rule='nz'):
    return [[max(0,min(255,math.floor(min(1.0,max(0.0,v))*256+1e-9))) for v in r] for r in fill_path_wind(subpaths,W,H,rule)]
