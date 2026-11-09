import sys,os,subprocess; os.chdir('/tmp/cg')
def g(t,inp,n,W=8,H=8): return [[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run([t,str(W),str(H)],input=inp,capture_output=True,text=True).stdout.split('--\n')[:n]]
cases=[((12445/4096,14070/4096),(25203/4096,7641/4096),(12959/4096,2352/4096)),((16245/4096,16201/4096),(11472/4096,1377/4096),(3170/4096,2520/4096))]
for t in cases:
    K=64
    inp=''.join('m %r %r\nl %r %r\nl %r %r\nh\nf\nn\n'%tuple(v for p in t for v in (p[0],p[1]+k/4096)) for k in range(-32,K-32))
    A=g('./cov',inp,K); B=g('/tmp/cgclean/target/release/cgclean',inp,K)
    ax=int(t[0][0]); 
    rowsA=[]; 
    out=[]
    for i,k in enumerate(range(-32,K-32)):
        ay=int(t[0][1]+k/4096)
        out.append((k,A[i][ay][ax],B[i][ay][ax]))
    print('apex',t[0]); print([(k,a,b) for k,a,b in out if a!=b][:20]); print('cg',[a for k,a,b in out]); print('md',[b for k,a,b in out])
