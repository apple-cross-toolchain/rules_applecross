import sys, math, random, subprocess, collections, pickle; sys.path.insert(0,'/tmp/cg'); import raster
N=12
def gen(seed,n=200):
    random.seed(seed); W=[]
    for _ in range(n):
        vx,vy=random.uniform(4,8),random.uniform(4,8)
        a1=random.uniform(0,2*math.pi); a2=a1+random.uniform(0.3,math.pi-0.3)
        R=30
        W.append([(vx,vy),(vx+R*math.cos(a1),vy+R*math.sin(a1)),(vx+R*math.cos(a2),vy+R*math.sin(a2))])
    inp=''.join('m %r %r\nl %r %r\nl %r %r\nh\nf\nn\n'%(t[0]+t[1]+t[2]) for t in W)
    out=subprocess.run(['./cov',str(N),str(N)],input=inp,capture_output=True,text=True).stdout.split('--\n')
    return [(t,[list(map(int,r.split())) for r in o.strip().split('\n')][::-1]) for t,o in zip(W,out)]
def kind(t):
    v=t[0]; ys=[t[1][1],t[2][1]]
    return 'bottom' if v[1]<min(ys) else 'top' if v[1]>max(ys) else 'side'
def evaluate(D,verbose=0):
    cnt=collections.Counter(); bad=collections.Counter(); ex=[]
    for t,rows in D:
        raw=raster.fill_raw([t],N,N); k=kind(t); vy=math.floor(t[0][1]); vx=math.floor(t[0][0])
        for y in range(N):
            for x in range(N):
                a=abs(raw[y][x])*256; v=max(0,min(255,math.floor(a+1e-9))); o=rows[y][x]
                if v in (0,255) and o==v: continue
                where='vpix' if (x,y)==(vx,vy) else 'vrow' if y==vy else 'other'
                cnt[(k,where)]+=1
                if v!=o: bad[(k,where)]+=1; ex.append((k,where,round(a,3),o,x,y,t))
    return {k:'%d/%d'%(bad[k],cnt[k]) for k in sorted(cnt)}, ex
if __name__=='__main__':
    D=gen(1); pickle.dump(D,open('/tmp/cg/wedge1.pkl','wb'))
    s,ex=evaluate(D); print(s)
    for e in ex[:15]: print(e[:6], [tuple(round(c,3) for c in p) for p in e[6]])
