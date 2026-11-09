import sys,random,math,os; sys.path.insert(0,'/tmp/cg'); import rsc
def inp(k,n=20):
    random.seed(k); s=''
    for _ in range(n):
        r=10; cx,cy=random.uniform(15,17),random.uniform(15,17); ph=random.uniform(0,6.28)
        P=[(cx+r*math.cos(ph+2*math.pi*i/k),cy+r*math.sin(ph+2*math.pi*i/k)) for i in range(k)]
        s+='m %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\nf\nn\n'
    return s
D={k:inp(k) for k in (64,256,512)}
def run(env={}):
    old=dict(os.environ); os.environ.update(env)
    try: return [rsc.compare(D[k],32,32,20)[0] for k in (64,256,512)]
    finally: os.environ.clear(); os.environ.update(old)
if __name__=='__main__':
    print('base',run())
    for k in ('CG_START','CG_CSTART','CG_END','CG_BAND'):
        for m in ('out','in','ceil','round'): print(k,m,run({k:m}))
