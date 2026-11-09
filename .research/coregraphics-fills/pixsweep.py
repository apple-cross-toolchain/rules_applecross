import subprocess,os,math
S=4096
def q(v): return round(v*S)/S
def obs_model(P,px,py,W=8,H=8,K=96,env=None):
    shapes=[]
    for k in range(K):
        Q=[(q(x)+k/S,q(y)) for x,y in P]
        shapes.append('m %r %r\n'%Q[0]+''.join('l %r %r\n'%p for p in Q[1:])+'h\nf\nn\n')
    inp=''.join(shapes)
    O=[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run(['./cov',str(W),str(H)],input=inp,capture_output=True,text=True).stdout.split('--\n')[:K]]
    M=[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run(['/tmp/cgprobe/target/release/cgprobe',str(W),str(H)],input=inp,capture_output=True,text=True,env={**os.environ,**(env or {})}).stdout.split('--\n')[:K]]
    return [O[k][py][px] for k in range(K)],[M[k][py][px] for k in range(K)]
