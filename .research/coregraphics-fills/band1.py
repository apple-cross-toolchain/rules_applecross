import sys,subprocess,os,re,math,random; sys.path.insert(0,'/tmp/cg'); import cfit, intfit
S=4096
def load(seed=2,n=30):
    random.seed(seed); data=[]
    while len(data)<n:
        L0=(random.uniform(10.1,13.9),2.3); L1=(L0[0]+random.uniform(-2,2),29.6)
        yb=random.uniform(8,22)
        if not 0.1<yb%1<0.9: continue
        P=[L0,(30.5,2.1),(30.9,yb),(30.4,29.7),L1]
        y=int(yb); xl=L0[0]+(L1[0]-L0[0])*(yb-L0[1])/(L1[1]-L0[1]); px=int(xl)
        lo,hi,pts=cfit.sweep(P,px,y,K=160)
        if len(pts)<100: continue
        Q=[(cfit.q(a),cfit.q(b)) for a,b in P]
        inp='m %r %r\n'%Q[0]+''.join('l %r %r\n'%p for p in Q[1:])+'h\nf\n'
        err=subprocess.run(['/tmp/cgprobe/target/release/cgprobe','32','32'],input=inp,capture_output=True,text=True,env={**os.environ,'CG_ACC':'1','CG_DEBUG':'%d,%d'%(px,y)}).stderr
        pcs=[];cs=[]
        for line in err.strip().split('\n'):
            m=re.search(r'role 1 piece \(([\d.]+),([\d.]+)\)->\(([\d.]+),([\d.]+)\) contrib (-?[\d.]+)',line)
            if not m: continue
            xa,ya,xb,yb_,c=map(float,m.groups()); pcs.append((round(xa*S),round(ya*S),round(xb*S),round(yb_*S))); cs.append(c*16)
        exact=sum(cs); data.append(dict(pts=pts,W=(px+1)*S,pcs=pcs,lo=exact+lo*16,hi=exact+hi*16,exact=exact,cs=cs,P=Q,yb=yb))
    return data
if __name__=='__main__':
    D=load()
    for d in D[:12]: print('target [%.3f,%.3f] exact %.3f pieces %d contribs %s'%(d['lo'],d['hi'],d['exact'],len(d['pcs']),[round(c,3) for c in d['cs']]))
    for k,f in intfit.C.items(): print(intfit.score(D,f),'/',len(D),k)
