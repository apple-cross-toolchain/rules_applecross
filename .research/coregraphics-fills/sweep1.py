import subprocess,collections,os
W,H=24,32; a=(5.3,2.1); b=(9.7,28.4)
def shape(t):
    v=(a[0]+(b[0]-a[0])*t, a[1]+(b[1]-a[1])*t)
    P=[a,(20.2,2.3),(19.6,28.9),b,v]
    return 'm %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\nf\nn\n'
ts=[0.3+i/2000 for i in range(400)]
inp=''.join(shape(t) for t in ts)
g=lambda tool,env={}:[[list(map(int,r.split())) for r in o.strip().split('\n')] for o in subprocess.run([tool,str(W),str(H)],input=inp,capture_output=True,text=True,env={**os.environ,**env}).stdout.split('--\n')[:len(ts)]]
O=g('./cov')
def run(env):
    M=g('/tmp/cgprobe/target/release/cgprobe',env)
    return sum(1 for o,m in zip(O,M) for ro,rm in zip(o,m) for x,y in zip(ro,rm) if x!=y)
if __name__=='__main__':
    for env in ({},{'CG_CHAIN':'open'},{'CG_BANDSIDE':'open'},{'CG_CHAIN':'open','CG_BANDSIDE':'open'},{'CG_ENDCHAIN':'open'},{'CG_PQ':'4096:floor','CG_PQ_V':'chainrow'}):
        print(env,run(env))
