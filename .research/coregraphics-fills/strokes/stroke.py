import math
KAPPA=0.5522847498307935
def unit(a,b):
    dx,dy=b[0]-a[0],b[1]-a[1]; l=math.hypot(dx,dy); return (dx/l,dy/l)
def join(out,v,u1,u2,d,jt,ml=10.0,l1=None,l2=None,rev=False):
    # emits points after the end offset of the incoming segment, up to (excluding) the start offset of the next, on the side with offset d (left normal * d)
    n1=(-u1[1]*d,u1[0]*d); n2=(-u2[1]*d,u2[0]*d)
    p1=(v[0]+n1[0],v[1]+n1[1]); p2=(v[0]+n2[0],v[1]+n2[1])
    cross=u1[0]*u2[1]-u1[1]*u2[0]
    dot=u1[0]*u2[0]+u1[1]*u2[1]
    outer=(cross<0) if d>0 else (cross>0)
    if jt==1 and dot<0 and abs(cross)<1e-3: outer=rev
    gap2=(p1[0]-p2[0])**2+(p1[1]-p2[1])**2
    if gap2<0.01: return []
    if not outer or cross==0 and dot>0:
        if not outer and l1 is not None and cross!=0 and min(l1,l2) < abs(d): return [('l',v),('l',p2)]
        return [('l',p2)]
    if jt==0:
        # miter: ratio 1/sin(phi/2), phi = angle between segments = pi - turn
        cos_t=dot
        if 2/(1+cos_t) <= ml*ml and gap2>=0.05:
            # intersection of offset lines
            s=(n1[0]+n2[0],n1[1]+n2[1]); f=2/(1+cos_t)/2
            m=(v[0]+s[0]*f*1,v[1]+s[1]*f*1)
            return [('l',m),('l',p2)]
        return [('l',p2)]
    if jt==1 and gap2>=0.05:
        th=math.pi if dot<0 and abs(cross)<1e-3 else math.acos(max(-1,min(1,dot)))
        k=4/3*math.tan(th/4)*abs(d)
        c1=(p1[0]+u1[0]*k,p1[1]+u1[1]*k); c2=(p2[0]-u2[0]*k,p2[1]-u2[1]*k)
        return [('c',c1,c2,p2)]
    return [('l',p2)]
def cap(v,u,d,ct):
    # at end point v with direction u (pointing outward), from left offset (+d) to right offset (-d)
    n=(-u[1]*d,u[0]*d)
    pl=(v[0]+n[0],v[1]+n[1]); pr=(v[0]-n[0],v[1]-n[1])
    if ct==0: return [('l',pr)]
    if ct==2:
        e=(u[0]*d,u[1]*d)
        return [('l',(pl[0]+e[0],pl[1]+e[1])),('l',(pr[0]+e[0],pr[1]+e[1])),('l',pr)]
    k=KAPPA*d; tip=(v[0]+u[0]*d,v[1]+u[1]*d)
    return [('c',(pl[0]+u[0]*k,pl[1]+u[1]*k),(tip[0]+n[0]*KAPPA,tip[1]+n[1]*KAPPA),tip),
            ('c',(tip[0]-n[0]*KAPPA,tip[1]-n[1]*KAPPA),(pr[0]+u[0]*k,pr[1]+u[1]*k),pr)]
def side(P,d,jt,closed,rev=False):
    segs=[(P[i],P[i+1]) for i in range(len(P)-1)]
    if closed: segs.append((P[-1],P[0]))
    U=[unit(a,b) for a,b in segs]
    out=[]
    for i,(a,b) in enumerate(segs):
        u=U[i]; n=(-u[1]*d,u[0]*d)
        if i==0: start=('m',(a[0]+n[0],a[1]+n[1]))
        out.append(('l',(b[0]+n[0],b[1]+n[1])))
        if i+1<len(segs): out+=join(out,b,u,U[i+1],d,jt,l1=math.dist(a,b),l2=math.dist(*segs[i+1]),rev=rev)
    return start,out,U,segs
def stroke(P,w,jt,ct,closed):
    d=w/2
    # drop zero-length segments
    Q=[P[0]]
    for p in P[1:]:
        if p!=Q[-1]: Q.append(p)
    if closed and Q[-1]==Q[0] and len(Q)>1: Q.pop()
    P=Q
    res=[]
    if not closed:
        s,o,U,segs=side(P,d,jt,False)
        res.append(s); res+=o
        res+=cap(P[-1],U[-1],d,ct)
        R=P[::-1]
        s2,o2,_,_=side(R,d,jt,False,rev=True)
        res+=o2
        res+=cap(R[-1],unit(R[-2],R[-1]),d,ct)
        res.append(('h',))
        return res
    s,o,U,segs=side(P,d,jt,True)
    res.append(s); res+=o; res+=join(None,P[0],U[-1],U[0],d,jt,l1=math.dist(*segs[-1]),l2=math.dist(*segs[0])); res.append(('h',))
    # right contour: start at seg1 start right offset, join at P0 going backward, then reversed segments
    R=[P[0]]+P[:0:-1]
    s2,o2,U2,_=side(R,d,jt,True,rev=True)
    # outer contour in CG starts at seg1 start (right) = end of last reversed seg
    u_first=U[0]; n=(-u_first[1]*-d,u_first[0]*-d)
    j0=join(None,P[0],(-U[0][0],-U[0][1]),U2[0],d,jt,l1=math.dist(*segs[0]),l2=math.dist(*segs[-1]),rev=True)
    if j0:
        res.append(('m',(P[0][0]+n[0],P[0][1]+n[1])))
        res+=j0
    else:
        res.append(('m',s2[1]))
    res+=o2
    res.append(('h',))
    return res
