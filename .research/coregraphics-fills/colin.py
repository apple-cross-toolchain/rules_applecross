import subprocess,os
W,H=24,32
def shape(n,side='left'):
    a=(5.3,2.1); b=(9.7,28.4)
    left=[(a[0]+(b[0]-a[0])*i/n, a[1]+(b[1]-a[1])*i/n) for i in range(n+1)]
    P=[a,(20.2,2.3),(19.6,28.9),b]+list(reversed(left[1:-1]))
    if side=='right':
        r0=(20.2,2.3); r1=(19.6,28.9)
        P=[a]+[(r0[0]+(r1[0]-r0[0])*i/n, r0[1]+(r1[1]-r0[1])*i/n) for i in range(n+1)]+[b]
    return 'm %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\nf\nn\n'
ns=[1,2,5,10,30,100,300]
def grids(tool,inp,k,env={}): return [[list(map(int,r.split())) for r in o.strip().split('\n')] for o in subprocess.run([tool,str(W),str(H)],input=inp,capture_output=True,text=True,env={**os.environ,**env}).stdout.split('--\n')[:k]]
def run(env={}):
    res={}
    for side in ('left','right'):
        inp=''.join(shape(n,side) for n in ns)
        O=grids('./cov',inp,len(ns)); M=grids('/tmp/cgprobe/target/release/cgprobe',inp,len(ns),env)
        res[side]=[sum(1 for ra,rb in zip(o,m) for x,y in zip(ra,rb) if x!=y) for o,m in zip(O,M)]
    return res
if __name__=='__main__':
    for c in (None,'open','all'):
        print(c, run({'CG_CHAIN':c} if c else {}))
