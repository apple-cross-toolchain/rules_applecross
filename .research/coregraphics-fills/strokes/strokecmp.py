import sys,os,subprocess,random,math; sys.path.insert(0,'/tmp/cgw'); import stroke
os.chdir('/tmp/cg')
def cg(paths):
    inp=''
    for P,w,jt,ct,closed in paths:
        inp+='m %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+('h\n' if closed else '')+'s %r %d %d\n'%(w,jt,ct)
    out=subprocess.run(['./outline'],input=inp,capture_output=True,text=True).stdout.split('--\n')
    res=[]
    for o in out[:len(paths)]:
        els=[]
        for l in o.strip().split('\n'):
            t=l.split(); v=list(map(float,t[1:]))
            els.append((t[0],)+tuple((v[i],v[i+1]) for i in range(0,len(v),2)))
        res.append(els)
    return res
def same(a,b,tol):
    if len(a)!=len(b): return False
    for x,y in zip(a,b):
        if x[0]!=y[0] or len(x)!=len(y): return False
        for p,q in zip(x[1:],y[1:]):
            if abs(p[0]-q[0])>tol or abs(p[1]-q[1])>tol: return False
    return True
def show(a):
    return '\n'.join(' '.join([e[0]]+['%.6f,%.6f'%p for p in e[1:]]) for e in a)
if __name__=='__main__':
    random.seed(int(sys.argv[1])); N=int(sys.argv[2]); closed=sys.argv[3]=='1'; jt=int(sys.argv[4]); ct=int(sys.argv[5])
    paths=[]
    for _ in range(N):
        n=random.randint(2,5) if not closed else random.randint(3,5)
        paths.append(([(round(random.uniform(0,20),3),round(random.uniform(0,20),3)) for _ in range(n)],random.choice([1.5,2,3,5]),jt,ct,closed))
    C=cg(paths); ok=0; shown=0
    for p,c in zip(paths,C):
        m=stroke.stroke(*p)
        if same(c,m,1e-9): ok+=1
        elif shown<1:
            shown+=1; print('path',p); print('CG:'); print(show(c)); print('mine:'); print(show(m))
    print('match',ok,'/',N)
