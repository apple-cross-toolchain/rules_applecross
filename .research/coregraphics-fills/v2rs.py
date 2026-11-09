import sys,subprocess,random,os,collections
W,H=32,32
def gen(seed=9,n=300):
    random.seed(seed); polys=[]
    for _ in range(n):
        r=random.randint(10,20); y1=r+random.uniform(0.05,0.95); y2=r+random.uniform(0.05,0.95)
        lo,hi=min(y1,y2),max(y1,y2); x1=random.uniform(12,18); x2=x1+random.uniform(-1.5,1.5)
        P=[(x1+random.uniform(-3,3),2.3),(28.7,3.1),(27.9,29.4),(x2+random.uniform(-3,3),29.8),(x2,hi),(x1,lo)]
        polys.append(P)
    return polys
P=gen(); INP=''.join('m %r %r\n'%p[0]+''.join('l %r %r\n'%q for q in p[1:])+'h\nf\nn\n' for p in P)
g=lambda tool,env={}:[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run([tool,str(W),str(H)],input=INP,capture_output=True,text=True,env={**os.environ,**env}).stdout.split('--\n')[:len(P)]]
O=g('./cov')
def run(env):
    M=g('/tmp/cgprobe/target/release/cgprobe',env); c=collections.Counter()
    for p,o,m in zip(P,O,M):
        vr=int(min(p[4][1],p[5][1]))
        for y in range(H):
            for x in range(W):
                if o[y][x]!=m[y][x]: c[('vrow' if y==vr else 'other','left' if x<22 else 'right',m[y][x]-o[y][x])]+=1
    return sum(c.values()),dict(c)
if __name__=='__main__':
    for env in ({},{'CG_ACC':'1'},{'CG_ACC':'1','CG_JOIN':'open'}): print(env,run(env))
