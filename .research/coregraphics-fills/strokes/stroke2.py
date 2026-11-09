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
    if math.hypot(*b)==0: b=(q[3][0]-q[1][0],q[3][1]-q[1][1])
    return norm(a),norm(b)
def seg_len(s):
    if s[0]=='l': return math.dist(s[1],s[2])
    q=s[1:]; return sum(math.dist(bez(q,i/16),bez(q,(i+1)/16)) for i in range(16))
def seg_off(s,d):
    # elements for offset of s on the left side at distance d, excluding the start point
    if s[0]=='l':
        u=S1.unit(s[1],s[2]); n=(-u[1]*d,u[0]*d); return [('l',(s[2][0]+n[0],s[2][1]+n[1]))]
    q=list(s[1:]); u0,u3=seg_dirs(s)
    p0=(q[0][0]-u0[1]*d,q[0][1]+u0[0]*d); p3=(q[3][0]-u3[1]*d,q[3][1]+u3[0]*d)
    m=bez(q,.5); t=d1(q,.5)
    if math.hypot(*t)==0: t=(q[3][0]-q[0][0],q[3][1]-q[0][1])
    tn=norm(t); m=(m[0]-tn[1]*d,m[1]+tn[0]*d)
    h1=(q[1][0]-q[0][0],q[1][1]-q[0][1]); h2=(q[2][0]-q[3][0],q[2][1]-q[3][1])
    bx=(m[0]*8-4*p0[0]-4*p3[0])/3; by=(m[1]*8-4*p0[1]-4*p3[1])/3
    det=h1[0]*h2[1]-h1[1]*h2[0]
    if abs(det)>1e-12:
        a1=(bx*h2[1]-by*h2[0])/det; a2=(h1[0]*by-h1[1]*bx)/det
    else:
        cx=h1[0]+h2[0]; cy=h1[1]+h2[1]
        a1=a2=(bx*cx+by*cy)/(cx*cx+cy*cy) if cx*cx+cy*cy>0 else 1.0
    return [('c',(p0[0]+a1*h1[0],p0[1]+a1*h1[1]),(p3[0]+a2*h2[0],p3[1]+a2*h2[1]),p3)]
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
