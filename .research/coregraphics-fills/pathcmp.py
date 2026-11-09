import sys,subprocess,math; sys.path.insert(0,'/tmp/cg'); import raster, flat
TOL=0.05/0.75
CURVEPTS=set()
SPLIT=''
def extrema(c0,c1,c2,c3):
    # t in (0,1) where the derivative of one coordinate is zero
    a=-c0+3*c1-3*c2+c3; b=2*(c0-2*c1+c2); c=c1-c0
    ts=[]
    if abs(a)<1e-12:
        if abs(b)>1e-12: ts=[-c/b]
    else:
        disc=b*b-4*a*c
        if disc>=0:
            r=math.sqrt(disc); ts=[(-b-r)/(2*a),(-b+r)/(2*a)]
    return [t for t in ts if 1e-9<t<1-1e-9]
def split_at(q,t):
    p0,p1,p2,p3=q
    L=lambda a,b:(a[0]+(b[0]-a[0])*t,a[1]+(b[1]-a[1])*t)
    a=L(p0,p1); b=L(p1,p2); c=L(p2,p3); d=L(a,b); e=L(b,c); f=L(d,e)
    return (p0,a,d,f),(f,e,c,p3)
def flat_mono(q):
    ts=[]
    if 'y' in SPLIT: ts+=extrema(q[0][1],q[1][1],q[2][1],q[3][1])
    if 'x' in SPLIT: ts+=extrema(q[0][0],q[1][0],q[2][0],q[3][0])
    ts=sorted(set(ts)); pieces=[]; rest=q; prev=0.0
    for t in ts:
        a,rest=split_at(rest,(t-prev)/(1-prev)); pieces.append(a); prev=t
    pieces.append(rest)
    out=[]
    for pc in pieces: out+=flat.flat_rec(pc,'ddinf',TOL,False)
    return out
def parse(text):
    subs=[];cur=None;start=None;rule='nz'
    for line in text.split('\n'):
        t=line.split()
        if not t: continue
        v=list(map(float,t[1:]))
        if t[0]=='m':
            if cur and len(cur)>1: subs.append(cur)
            cur=[(v[0],v[1])]; start=cur[0]
        elif t[0]=='l': cur.append((v[0],v[1]))
        elif t[0]=='c':
            q=(cur[-1],(v[0],v[1]),(v[2],v[3]),(v[4],v[5]))
            pts=flat_mono(q)
            CURVEPTS.update(pts[1:])
            cur+=pts[1:]+[q[3]]
        elif t[0]=='h':
            if cur and len(cur)>1:
                if cur[-1]==cur[0]: cur=cur[:-1]
                subs.append(cur)
            cur=None
        elif t[0] in ('f','e'): rule='nz' if t[0]=='f' else 'eo'
    if cur and len(cur)>1: subs.append(cur)
    # drop consecutive duplicates
    out=[]
    for P in subs:
        Q=[P[0]]+[p for i,p in enumerate(P[1:],1) if p!=P[i-1]]
        out.append(Q)
    return out,rule
def compare(text,W,H):
    o=subprocess.run(['./cov',str(W),str(H)],input=text,capture_output=True,text=True).stdout.split('--\n')[0]
    obs=[list(map(int,r.split())) for r in o.strip().split('\n')][::-1]
    CURVEPTS.clear(); subs,rule=parse(text)
    raster.CHAIN_SET={(raster.vq(x),raster.vq(y)) for x,y in CURVEPTS}
    m=raster.fill_path(subs,W,H,rule)
    bad=[(x,y,m[y][x],obs[y][x]) for y in range(H) for x in range(W) if m[y][x]!=obs[y][x]]
    edge=sum(1 for y in range(H) for x in range(W) if 0<obs[y][x]<255)
    return bad,edge
if __name__=='__main__':
    font,size,txt=sys.argv[1],sys.argv[2],sys.argv[3]; W,H=int(sys.argv[4]),int(sys.argv[5])
    text=subprocess.run(['./glyphs',font,size,'2.0','5.0',txt],capture_output=True,text=True).stdout
    bad,edge=compare(text,W,H)
    print(font,size,txt,'wrong',len(bad),'of',edge,'edge pixels; |diff|>1:',sum(1 for b in bad if abs(b[2]-b[3])>1))
    print(bad[:10])
