import sys,os,subprocess,random,math,ast; sys.path.insert(0,'/tmp/cgw'); os.chdir('/tmp/cg')
def parse(o):
    res=[]
    for blk in o.split('--\n'):
        els=[]
        for l in blk.strip().split('\n'):
            if not l: continue
            t=l.split()
            if t[0] not in ('m','l','c','h'): continue
            v=list(map(float,t[1:]))
            els.append((t[0],)+tuple((v[i],v[i+1]) for i in range(0,len(v),2)))
        if els: res.append(els)
    return res
def same(a,b,tol=1e-9):
    return len(a)==len(b) and all(x[0]==y[0] and all(math.dist(p,q)<tol for p,q in zip(x[1:],y[1:])) for x,y in zip(a,b))
def run(inputs):
    inp=''.join(p+'s %r %d %d\n'%(w,j,c) for p,w,j,c in inputs)
    cg=parse(subprocess.run(['./outline'],input=inp,capture_output=True,text=True).stdout)
    rs=parse(subprocess.run(['/tmp/cgclean/target/release/cgclean','8','8'],input=inp.replace('\ns ','\nS '),capture_output=True,text=True).stdout)
    return cg,rs
random.seed(5); tests=[]
for _ in range(600):
    n=random.randint(2,5); closed=random.random()<0.5
    pts=[(round(random.uniform(0,20),3),round(random.uniform(0,20),3)) for _ in range(n+closed)]
    p='m %r %r\n'%pts[0]+''.join('l %r %r\n'%q for q in pts[1:])+('h\n' if closed else '')
    tests.append((p,random.choice([1.5,2,3,5]),random.randint(0,2),random.randint(0,2)))
cg,rs=run(tests); print('polylines',sum(same(a,b) for a,b in zip(cg,rs)),'/',len(tests))
tests=[]
for _ in range(400):
    q=[(round(random.uniform(0,20),3),round(random.uniform(0,20),3)) for _ in range(4)]
    p='m %r %r\nc %r %r %r %r %r %r\n'%tuple(v for pt in q for v in pt)
    tests.append((p,random.choice([1.5,2,3,5]),random.randint(0,2),random.randint(0,2)))
cg,rs=run(tests); print('single curves (random, many split)',sum(same(a,b) for a,b in zip(cg,rs)),'/',len(tests))
tests=[]
for line in open('/tmp/cgw/splitlist.txt'):
    w,q=ast.literal_eval(line)
    tests.append(('m %r %r\nc %r %r %r %r %r %r\n'%tuple(v for pt in q for v in pt),w,0,0))
cg,rs=run(tests); print('corpus split curves',sum(same(a,b) for a,b in zip(cg,rs)),'/',len(tests))
