import sys,pickle,math,collections; sys.path.insert(0,'/tmp/cg'); import raster, trieval
S=4096
def tagged_pieces(P):
    P=[(raster.vq(x),raster.vq(y)) for x,y in P]
    raster.BREAKS=sorted({y for x,y in P})
    out=[]
    for i in range(len(P)):
        for a,b,row in raster.edge_pieces(P[i],P[(i+1)%len(P)]):
            out.append((i,a,b,row))
    return P,out
def pixel_area(pieces,x,y,shift=None):
    acc=0.0
    for e,a,b,row in pieces:
        if row!=y: continue
        if shift:
            (se,pt,dx,dy)=shift
            if e==se and a==pt: a=(a[0]+dx,a[1]+dy)
            if e==se and b==pt: b=(b[0]+dx,b[1]+dy)
        d=b[1]-a[1]
        lo=math.floor(min(a[0],b[0])); hi=math.floor(max(a[0],b[0]))
        if lo<=x<=hi: acc+=raster.seg_I(a,b,x+1)-raster.seg_I(a,b,x)-d
        elif x>hi: acc-=d
    return abs(acc)*256
def kind(pt,P):
    if pt in P: return 'vertex'
    if pt[1]==int(pt[1]): return 'row'
    if pt[0]==int(pt[0]): return 'col'
    if pt[1] in {p[1] for p in P}: return 'band/end'
    return '?'
T=trieval.load(); fixes=collections.Counter(); n=0
for t,rows in T:
    m=raster.fill([t],8,8)
    for y in range(8):
        for x in range(8):
            o=rows[y][x]
            if m[y][x]==o: continue
            P,pcs=tagged_pieces(t); n+=1
            pts={(e,p) for e,a,b,r in pcs if r==y for p in (a,b)}
            sols=[]
            for e,p in pts:
                for dx,dy in ((1/S,0),(-1/S,0),(0,1/S),(0,-1/S)):
                    a=pixel_area(pcs,x,y,(e,p,dx,dy))
                    if max(0,min(255,math.floor(a+1e-9)))==o: sols.append((kind(p,P),'x' if dx else 'y','+' if dx+dy>0 else '-'))
            print((x,y),m[y][x],o,sorted(set(sols)))
            for s in set(sols): fixes[s]+=1
print(n,fixes.most_common(12))
