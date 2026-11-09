import subprocess,random,os,math,collections
S=4096
def run(kind,seed=1,n=40,K=48,W=32,H=32,tool='/tmp/cgclean/target/release/cgclean',env=None):
    random.seed(seed); tot=0; bad=0; per=[]
    for _ in range(n):
        vx=random.uniform(12,20); vy=random.uniform(12,20)
        if kind=='bottom': A=(vx+random.uniform(-10,-2),30); B=(vx+random.uniform(2,10),30)
        elif kind=='top': A=(vx+random.uniform(2,10),2); B=(vx+random.uniform(-10,-2),2)
        else: A=(vx+random.uniform(5,10),30); B=(vx+random.uniform(5,10),2)   # side vertex (left)
        P=[(vx,vy),B,A] if kind!='side' else [(vx,vy),B,A]
        # ensure CCW irrelevant (nonzero)
        P=[(round(x*S)/S,round(y*S)/S) for x,y in P]
        inp=''
        for k in range(K):
            Q=[(x+k/S,y) for x,y in P]; inp+='m %r %r\n'%Q[0]+''.join('l %r %r\n'%p for p in Q[1:])+'h\nf\nn\n'
        g=lambda t,e:[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run([t,str(W),str(H)],input=inp,capture_output=True,text=True,env={**os.environ,**(e or {})}).stdout.split('--\n')[:K]]
        A_=g('./cov',None); B_=g(tool,env)
        px,py=int(P[0][0]),int(P[0][1]); nb=0
        for k in range(K):
            for xx in (px-1,px,px+1):
                o=A_[k][py][xx]; m=B_[k][py][xx]
                if o or m: tot+=1; nb+= o!=m
        bad+=nb; per.append(nb)
    return bad,tot,per
if __name__=='__main__':
    for kind in ('bottom','top','side'):
        b,t,per=run(kind); print(kind,'vertex row pixels mismatched',b,'of',t,'cases with any',sum(1 for p in per if p))
