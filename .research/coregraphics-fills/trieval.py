import sys, collections, math, random, subprocess, pickle, os; sys.path.insert(0,'/tmp/cg'); import raster
def load(seeds=(1,2,3,4),N=8,n=60):
    fn='/tmp/cg/tris_%s.pkl'%'_'.join(map(str,seeds))
    if os.path.exists(fn): return pickle.load(open(fn,'rb'))
    D=[]
    for seed in seeds:
        random.seed(seed)
        tris=[[(random.uniform(0,N),random.uniform(0,N)) for _ in range(3)] for _ in range(n)]
        inp=''.join('m %r %r\nl %r %r\nl %r %r\nh\nf\nn\n'%(t[0]+t[1]+t[2]) for t in tris)
        out=subprocess.run(['./cov',str(N),str(N)],input=inp,capture_output=True,text=True).stdout.split('--\n')
        for t,img in zip(tris,out):
            D.append((t,[list(map(int,r.split())) for r in img.strip().split('\n') if r][::-1]))
    pickle.dump(D,open(fn,'wb')); return D
def score(D,N=8):
    bad=0
    for t,rows in D:
        m=raster.fill([t],N,N)
        bad+=sum(1 for y in range(N) for x in range(N) if m[y][x]!=rows[y][x])
    return bad
