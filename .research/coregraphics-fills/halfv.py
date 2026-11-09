import subprocess,random,os,math
S=4096; W,H,K=32,32,48
def run(kind,seed=2,n=40,tool='/tmp/cgclean/target/release/cgclean',env=None):
    random.seed(seed); bad=0; tot=0; cases_bad=0
    for _ in range(n):
        vx=random.uniform(12,20); vy=random.uniform(12,20)
        if kind=='end':   # incoming edge from below-right ends at V, then horizontal edge to the right
            B=(vx+random.uniform(3,10),2.0); P=[(vx,vy),(28.5,vy),(28.5,2.0),B] if False else [B,(vx,vy),(29.5,vy),(29.5,2.0)]
        else:             # horizontal edge from right arrives at V, then outgoing edge to upper-right
            A=(vx+random.uniform(3,10),30.0); P=[(29.5,vy),(vx,vy),A,(29.5,30.0)]
        P=[(round(x*S)/S,round(y*S)/S) for x,y in P]
        inp=''
        for k in range(K):
            Q=[(x+k/S,y) for x,y in P]; inp+='m %r %r\n'%Q[0]+''.join('l %r %r\n'%p for p in Q[1:])+'h\nf\nn\n'
        g=lambda t,e:[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run([t,str(W),str(H)],input=inp,capture_output=True,text=True,env={**os.environ,**(e or {})}).stdout.split('--\n')[:K]]
        O=g('./cov',None); M=g(tool,env)
        py=int(vy); nb=0
        for k in range(K):
            for xx in range(int(vx)-1,int(vx)+2):
                if O[k][py][xx] or M[k][py][xx]: tot+=1; nb+=O[k][py][xx]!=M[k][py][xx]
        bad+=nb; cases_bad+=nb>0
    return bad,tot,cases_bad
if __name__=='__main__':
    for kind in ('end','start'): print(kind,run(kind))
