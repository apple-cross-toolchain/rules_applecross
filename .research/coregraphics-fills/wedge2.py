import sys, math, random, subprocess, collections, pickle; sys.path.insert(0,'/tmp/cg'); import raster
N=64
def gen(seed,n=150):
    random.seed(seed); W=[]
    for _ in range(n):
        vx,vy=random.uniform(28,36),random.uniform(28,36)
        a1=random.uniform(0,2*math.pi); a2=a1+random.uniform(0.3,math.pi-0.3); R=25
        W.append([(vx,vy),(vx+R*math.cos(a1),vy+R*math.sin(a1)),(vx+R*math.cos(a2),vy+R*math.sin(a2))])
    inp=''.join('m %r %r\nl %r %r\nl %r %r\nh\nf\nn\n'%(t[0]+t[1]+t[2]) for t in W)
    out=subprocess.run(['./cov',str(N),str(N)],input=inp,capture_output=True,text=True).stdout.split('--\n')
    return [(t,[list(map(int,r.split())) for r in o.strip().split('\n')][::-1]) for t,o in zip(W,out)]
def evaluate(D):
    cnt=collections.Counter(); bad=collections.Counter(); ex=[]
    for t,rows in D:
        raw=raster.fill_raw([t],N,N)
        vrows={math.floor(v[1]) for v in t}; vpix={(math.floor(v[0]),math.floor(v[1])) for v in t}
        for y in range(N):
            for x in range(N):
                a=abs(raw[y][x])*256; v=max(0,min(255,math.floor(a+1e-9))); o=rows[y][x]
                if v in (0,255) and o==v: continue
                w='vpix' if (x,y) in vpix else 'vrow' if y in vrows else 'other'
                cnt[w]+=1
                if v!=o: bad[w]+=1; ex.append((w,round(a,3),o,x,y,t))
    return {k:'%d/%d'%(bad[k],cnt[k]) for k in cnt}, ex
if __name__=='__main__':
    D=gen(1); pickle.dump(D,open('/tmp/cg/wedge2.pkl','wb'))
    s,ex=evaluate(D); print(s)
    for e in ex[:12]: print(e[:5],[tuple(round(c,3) for c in p) for p in e[5]])
