import subprocess,sys,random,math
def grid(out,n): return [[list(map(int,r.split())) for r in o.strip().split('\n')] for o in out.split('--\n')[:n]]
def compare(inp,W,H,n):
    a=grid(subprocess.run(['./cov',str(W),str(H)],input=inp,capture_output=True,text=True).stdout,n)
    b=grid(subprocess.run(['/tmp/cgprobe/target/release/cgprobe',str(W),str(H)],input=inp,capture_output=True,text=True).stdout,n)
    bad=sum(1 for A,B in zip(a,b) for ra,rb in zip(A,B) for x,y in zip(ra,rb) if x!=y)
    edge=sum(1 for A in a for r in A for x in r if 0<x<255)
    return bad,edge
if __name__=='__main__':
    random.seed(1); N=8
    tris=[[(random.uniform(0,N),random.uniform(0,N)) for _ in range(3)] for _ in range(240)]
    print('tris',compare(''.join('m %r %r\nl %r %r\nl %r %r\nh\nf\nn\n'%(t[0]+t[1]+t[2]) for t in tris),8,8,240))
    for f,t,W in (('Helvetica','Rg&8',64),('Helvetica','AXWNZ',72),('Helvetica','EHLTI',72),('Times New Roman','Qa@',64)):
        x=subprocess.run(['./glyphs',f,'20','2.0','5.0',t],capture_output=True,text=True).stdout
        print(t,compare(x+'n\n',W,26,1))
    random.seed(1); inp=''
    for _ in range(60):
        polys=[[(random.uniform(1,15),random.uniform(1,15)) for _ in range(3)] for _ in range(2)]
        inp+=''.join('m %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\n' for P in polys)+'f\nn\n'
    print('multi nz',compare(inp,16,16,60))
    random.seed(3); inp=''
    for _ in range(30):
        k=random.randint(12,60); r=random.uniform(5,20); cx,cy=random.uniform(23,25),random.uniform(23,25); ph=random.uniform(0,6.28)
        P=[(cx+r*math.cos(ph+2*math.pi*i/k),cy+r*math.sin(ph+2*math.pi*i/k)) for i in range(k)]
        inp+='m %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\nf\nn\n'
    print('ngons',compare(inp,48,48,30))
