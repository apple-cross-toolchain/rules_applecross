import subprocess,os,math
S=4096
def q(v): return round(v*S)/S
def sweep(P,px,py,W=32,H=32,K=256,axis=0,step=1,env=None):
    env={**os.environ,'CG_ACC':'1',**(env or {})}
    shapes=[]
    for k in range(K):
        d=k*step/S
        Q=[(q(x)+(d if axis==0 else 0), q(y)+(d if axis==1 else 0)) for x,y in P]
        shapes.append('m %r %r\n'%Q[0]+''.join('l %r %r\n'%p for p in Q[1:])+'h\nf\nn\n')
    inp=''.join(shapes)
    O=[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run(['./cov',str(W),str(H)],input=inp,capture_output=True,text=True).stdout.split('--\n')[:K]]
    R=[[list(map(float,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run(['/tmp/cgprobe/target/release/cgprobe',str(W),str(H)],input=inp,capture_output=True,text=True,env={**env,'CG_RAW':'1'}).stdout.split('--\n')[:K]]
    lo,hi=-1e9,1e9; pts=[]
    for k in range(K):
        o=O[k][py][px]; a=R[k][py][px]
        if 0<o<255:
            lo=max(lo,o-a); hi=min(hi,o+1-a); pts.append((k,o,a))
    return lo,hi,pts
