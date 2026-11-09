# general stroker: segments are ('l',a,b) or ('c',q0,q1,q2,q3)
import math, sys
sys.path.insert(0,'/tmp/cgw')
import stroke as S1
from offs import approx, bez, d1, norm
def seg_dirs(s):
    if s[0]=='l': u=S1.unit(s[1],s[2]); return u,u
    q=s[1:]
    a=d1(q,0); b=d1(q,1)
    # degenerate handles
    if math.hypot(*a)==0: a=(q[2][0]-q[0][0],q[2][1]-q[0][1])
    if math.hypot(*a)==0: a=(q[3][0]-q[0][0],q[3][1]-q[0][1])
    if math.hypot(*b)==0: b=(q[3][0]-q[1][0],q[3][1]-q[1][1])
    if math.hypot(*b)==0: b=(q[3][0]-q[0][0],q[3][1]-q[0][1])
    if math.hypot(*a)==0: a=(1.0,0.0)
    if math.hypot(*b)==0: b=a
    return norm(a),norm(b)
def seg_len(s):
    if s[0]=='l': return math.dist(s[1],s[2])
    q=s[1:]; return sum(math.dist(bez(q,i/16),bez(q,(i+1)/16)) for i in range(16))
def _sub(q,a,b):
    from offs6 import sub
    return sub(q,a,b)
def _accept(q,d):
    from offs import off
    from bnd2 import nearest
    c=_piece(q,d)
    if c is None: return False
    return max(nearest(lambda x:bez(c,x),off(q,t,d)) for t in (0.1,0.3,0.5,0.7,0.9))<=0.1
def _piece(q,d):
    r=seg_off_one(('c',)+tuple(q),d)
    if r is None: return None
    e=r[0]; u0,_=seg_dirs(('c',)+tuple(q))
    p0=(q[0][0]-u0[1]*d,q[0][1]+u0[0]*d)
    return [p0,e[1],e[2],e[3]]
SPLIT=True
ACCEPT_ALL=False
def _coord_infl(q):
    ti=[]
    for i in (0,1):
        a=q[0][i]-2*q[1][i]+q[2][i]; b=q[1][i]-2*q[2][i]+q[3][i]
        if (a>0 and b<0) or (a<0 and b>0): ti.append(a/(a-b))
    return sorted(set(t for t in ti if 0<t<1))
def seg_off(s,d):
    if s[0]=='l' or not SPLIT: return seg_off_one(s,d)
    q=list(s[1:])
    out=[]
    from offs5 import inflections
    try: has_infl=any(1e-9<t<1-1e-9 for t in inflections(q))
    except Exception: has_infl=False
    pre=_coord_infl(q) if has_infl else []
    def rec(a,b,depth=0):
        if b-a<1e-9: return
        sq=_sub(q,a,b)
        try: ok=depth>=8 or ACCEPT_ALL or _accept(sq,d)
        except Exception: ok=True
        if ok:
            mid_piece=len(pre)==2 and abs(a-pre[0])<1e-12 and abs(b-pre[1])<1e-12
            if (mid_piece and math.dist(sq[0],sq[3])<0.5) or math.dist(sq[0],sq[3])<TINY: out.extend(seg_off_fb(('c',)+tuple(sq),d))
            else: out.extend(seg_off_one(('c',)+tuple(sq),d) or [])
            return
        m=(a+b)/2; rec(a,m,depth+1); rec(m,b,depth+1)
    cuts=[0.0]+pre+[1.0]
    for x,y in zip(cuts,cuts[1:]): rec(x,y)
    return out
TINY=0.1
def seg_off_fb(s,d):
    q=list(s[1:]); u0,u3=seg_dirs(s)
    p3=(q[3][0]-u3[1]*d,q[3][1]+u3[0]*d)
    t=d1(q,.5)
    if math.hypot(*t)==0: t=(q[3][0]-q[0][0],q[3][1]-q[0][1])
    um=norm(t); m=((q[1][0]+q[2][0])/2,(q[1][1]+q[2][1])/2); fb=(m[0]-um[1]*d,m[1]+um[0]*d)
    return [('c',fb,fb,p3)]
def seg_off_one(s,d):
    # elements for offset of s on the left side at distance d, excluding the start point
    if s[0]=='l':
        u=S1.unit(s[1],s[2]); n=(-u[1]*d,u[0]*d); return [('l',(s[2][0]+n[0],s[2][1]+n[1]))]
    q=list(s[1:]); u0,u3=seg_dirs(s)
    p0=(q[0][0]-u0[1]*d,q[0][1]+u0[0]*d); p3=(q[3][0]-u3[1]*d,q[3][1]+u3[0]*d)
    t=d1(q,.5)
    if math.hypot(*t)==0: t=(q[3][0]-q[0][0],q[3][1]-q[0][1])
    um=norm(t); M=bez(q,.5)
    ang=lambda a,b: math.atan2(a[0]*b[1]-a[1]*b[0],a[0]*b[0]+a[1]*b[1])
    t1=math.tan(ang(u0,um)/2); t2=math.tan(ang(um,u3)/2)
    if abs(t1)<1e-12 or abs(t2)<1e-12:
        m=((q[1][0]+q[2][0])/2,(q[1][1]+q[2][1])/2); n=(-um[1]*d,um[0]*d); c=(m[0]+n[0],m[1]+n[1])
        return [('c',c,c,p3)]
    from offfit import inter
    X1=inter(q[0],u0,M,um); X2=inter(q[3],u3,M,um)
    h1=(q[1][0]-q[0][0],q[1][1]-q[0][1]); h2=(q[2][0]-q[3][0],q[2][1]-q[3][1])
    L1=(2*(X1[0]-q[0][0])-h1[0]/2)*u0[0]+(2*(X1[1]-q[0][1])-h1[1]/2)*u0[1]
    L2=-((2*(X2[0]-q[3][0])-h2[0]/2)*u3[0]+(2*(X2[1]-q[3][1])-h2[1]/2)*u3[1])
    G=lambda a,b: (4/3)*a+(b*b-a*a)/(6*a*(1+b*b))
    neg1=L1<0; neg2=L2<0
    L1-=G(t1,t2)*d; L2-=G(t2,t1)*d
    c1=(p0[0]+u0[0]*L1,p0[1]+u0[1]*L1); c2=(p3[0]-u3[0]*L2,p3[1]-u3[1]*L2)
    if neg1 or neg2:
        m=((q[1][0]+q[2][0])/2,(q[1][1]+q[2][1])/2); n=(-um[1]*d,um[0]*d); fb=(m[0]+n[0],m[1]+n[1])
        if neg1: c1=fb
        if neg2: c2=fb
    return [('c',c1,c2,p3)]
def seg_start_off(s,d):
    u,_=seg_dirs(s); p=s[1]; return (p[0]-u[1]*d,p[1]+u[0]*d)
def rev(s):
    return ('l',s[2],s[1]) if s[0]=='l' else ('c',s[4],s[3],s[2],s[1])
def side(segs,d,jt,closed,rev_=False):
    out=[]
    for i,s in enumerate(segs):
        if i==0: start=('m',seg_start_off(s,d))
        out+=seg_off(s,d)
        if i+1<len(segs):
            _,u1=seg_dirs(s); u2,_=seg_dirs(segs[i+1])
            out+=S1.join(None,s[-1],u1,u2,d,jt,l1=seg_len(s),l2=seg_len(segs[i+1]),rev=rev_)
    return start,out
def stroke(segs,w,jt,ct,closed):
    d=w/2; res=[]
    if not closed:
        s,o=side(segs,d,jt,False); res.append(s); res+=o
        _,ue=seg_dirs(segs[-1]); res+=S1.cap(segs[-1][-1],ue,d,ct)
        R=[rev(s) for s in segs[::-1]]
        s2,o2=side(R,d,jt,False,True); res+=o2
        _,ue2=seg_dirs(R[-1]); res+=S1.cap(R[-1][-1],ue2,d,ct)
        res.append(('h',)); return res
    s,o=side(segs,d,jt,True); res.append(s); res+=o
    _,u1=seg_dirs(segs[-1]); u2,_=seg_dirs(segs[0])
    res+=S1.join(None,segs[0][1],u1,u2,d,jt,l1=seg_len(segs[-1]),l2=seg_len(segs[0])); res.append(('h',))
    R=[rev(s) for s in segs[::-1]]
    s2,o2=side(R,d,jt,True,True)
    uf,_=seg_dirs(segs[0]); n=(uf[1]*d,-uf[0]*d)
    _,ur=seg_dirs(R[-1]); ur2,_=seg_dirs(R[0])
    j0=S1.join(None,segs[0][1],ur,ur2,d,jt,l1=seg_len(segs[0]),l2=seg_len(segs[-1]),rev=True)
    if j0: res.append(('m',(segs[0][1][0]+n[0],segs[0][1][1]+n[1]))); res+=j0
    else: res.append(('m',s2[1]))
    res+=o2; res.append(('h',)); return res
def parse(path):
    # cov-format path text -> list of subpaths (segs, closed)
    subs=[]; cur=None; start=None; segs=[]
    for l in path.strip().split('\n'):
        t=l.split()
        if not t: continue
        v=list(map(float,t[1:]))
        if t[0]=='m':
            if segs: subs.append((segs,False))
            segs=[]; cur=start=(v[0],v[1])
        elif t[0]=='l':
            p=(v[0],v[1])
            if p!=cur: segs.append(('l',cur,p))
            cur=p
        elif t[0]=='c':
            p=(v[4],v[5]); segs.append(('c',cur,(v[0],v[1]),(v[2],v[3]),p)); cur=p
        elif t[0]=='h':
            if cur!=start: segs.append(('l',cur,start))
            if segs: subs.append((segs,True))
            segs=[]; cur=start
    if segs: subs.append((segs,False))
    return subs
def to_cov(els):
    out=[]
    for e in els:
        if e[0]=='h': out.append('h')
        else: out.append(e[0]+' '+' '.join('%r %r'%p for p in e[1:]))
    return '\n'.join(out)+'\n'
