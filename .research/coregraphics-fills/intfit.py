import sys,subprocess,os,re,math; sys.path.insert(0,'/tmp/cg'); import excess
S=4096
def cd(a,b): return -((-a)//b)
def load(seed=1,n=30):
    R=excess.measure(seed,n=n); data=[]
    for r in R:
        P=r['P']; vx,vy=r['V']; px,py=int(vx),int(vy)
        inp='m %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\nf\n'
        err=subprocess.run(['/tmp/cgprobe/target/release/cgprobe','32','32'],input=inp,capture_output=True,text=True,env={**os.environ,'CG_ACC':'1','CG_DEBUG':'%d,%d'%(px,py)}).stderr
        pcs=[]; cs=[]
        for line in err.strip().split('\n'):
            m=re.search(r'piece \(([\d.]+),([\d.]+)\)->\(([\d.]+),([\d.]+)\) contrib (-?[\d.]+)',line)
            xa,ya,xb,yb,c=map(float,m.groups()); pcs.append((round(xa*S),round(ya*S),round(xb*S),round(yb*S))); cs.append(c*16)
        exact=sum(cs); data.append(dict(pts=r['pts'],W=(px+1)*S,pcs=pcs,lo=exact+r['lo']*16,hi=exact+r['hi']*16))
    return data
C={
 'ceil(trap)':lambda h,W,a,b: cd(h*(2*W-a-b),2*S),
 'ceil(rect)+ceil(tri)':lambda h,W,a,b: cd(h*(W-max(a,b)),S)+cd(h*abs(b-a),2*S),
 'ceil(rect_min)-floor(tri)':lambda h,W,a,b: cd(h*(W-min(a,b)),S)-(h*abs(b-a))//(2*S),
 'ceil(half a)+ceil(half b)':lambda h,W,a,b: cd(h*(W-a),2*S)+cd(h*(W-b),2*S),
 'floor(trap)+1':lambda h,W,a,b: h*(2*W-a-b)//(2*S)+1,
 'ceil(h*(W-floor((a+b)/2)))':lambda h,W,a,b: cd(h*(W-(a+b)//2),S),
 'ceil(rect)+ceil(tri) /S both':lambda h,W,a,b: cd(h*(W-max(a,b))+cd(h*abs(b-a),2),S),
 'floor(rect)+ceil(tri)+?':lambda h,W,a,b: h*(W-max(a,b))//S+cd(h*abs(b-a),2*S)+1,
 'ceil(tri) + ceil(rect) with rect from min':lambda h,W,a,b: cd(h*(W-min(a,b)),S)-cd(h*abs(b-a),2*S)+1,
}
def score(data,f):
    ok=0
    for d in data:
        tot=sum(f(yb-ya,d['W'],xa,xb) for xa,ya,xb,yb in d['pcs'])
        if d['lo']-1e-6<=tot<=d['hi']+1e-6: ok+=1
    return ok
if __name__=='__main__':
    D=load()
    for k,f in C.items(): print(score(D,f),'/',len(D),k)
