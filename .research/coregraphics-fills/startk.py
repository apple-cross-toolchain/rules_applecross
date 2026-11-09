import sys,subprocess,random,os,math
S=4096; W,H=32,32
def load(kind='start',seed=2,n=20,K=400,env=None):
    random.seed(seed); D=[]
    for _ in range(n):
        vx=random.uniform(12,19); vy=random.uniform(12,20)
        if kind=='start': A=(vx+random.uniform(3,10),30.0); P=[(29.5,vy),(vx,vy),A,(29.5,30.0)]
        else: B=(vx+random.uniform(3,10),2.0); P=[B,(vx,vy),(29.5,vy),(29.5,2.0)]
        P=[(round(x*S)/S,round(y*S)/S) for x,y in P]; V=P[1]
        px,py=int(V[0]),int(V[1]); inp=''
        for k in range(K):
            Q=[(x+k/S,y) for x,y in P]; inp+='m %r %r\n'%Q[0]+''.join('l %r %r\n'%p for p in Q[1:])+'h\nf\nn\n'
        O=[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run(['./cov',str(W),str(H)],input=inp,capture_output=True,text=True).stdout.split('--\n')[:K]]
        R=[[list(map(float,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run(['/tmp/cgprobe/target/release/cgprobe',str(W),str(H)],input=inp,capture_output=True,text=True,env={**os.environ,'CG_ACC':'1','CG_RECTQ':'1','CG_RAW':'1',**(env or {})}).stdout.split('--\n')[:K]]
        # the vertex pixel shifts with k; track pixel containing V at each k
        pts=[]
        for k in range(K):
            x=int(V[0]+k/S)
            if 0<O[k][py][x]<255: pts.append((k,O[k][py][x],R[k][py][x]*16))
        D.append((P,pts))
    return D
