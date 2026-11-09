import sys,subprocess,random,math,os,re
S=4096; W,H,K=32,32,48
def cases(kind='side',seed=1,n=40):
    random.seed(seed); out=[]
    for _ in range(n):
        vx=random.uniform(12,20); vy=random.uniform(12,20)
        if kind=='side': A=(vx+random.uniform(5,10),30); B=(vx+random.uniform(5,10),2)
        elif kind=='bottom': A=(vx+random.uniform(-10,-2),30); B=(vx+random.uniform(2,10),30)
        else: A=(vx+random.uniform(2,10),2); B=(vx+random.uniform(-10,-2),2)
        P=[(round(x*S)/S,round(y*S)/S) for x,y in [(vx,vy),B,A]]
        out.append(P)
    return out
def collect(P,env):
    px,py=int(P[0][0]),int(P[0][1]); rows=[]
    for k in range(K):
        Q=[(x+k/S,y) for x,y in P]; inp='m %r %r\n'%Q[0]+''.join('l %r %r\n'%p for p in Q[1:])+'h\nf\n'
        o=[list(map(int,r.split())) for r in subprocess.run(['./cov',str(W),str(H)],input=inp,capture_output=True,text=True).stdout.split('--\n')[0].strip().split('\n')][::-1][py][px]
        err=subprocess.run(['/tmp/cgprobe/target/release/cgprobe',str(W),str(H)],input=inp,capture_output=True,text=True,env={**os.environ,**env,'CG_DEBUG':'%d,%d'%(px,py)}).stderr
        parts=[(int(m.group(1)),float(m.group(2))*16,m.group(0)) for m in re.finditer(r'band \[[\d.]+,[\d.]+\] edge \d+ role (-?1) (?:full )?piece .*?contrib (-?[\d.]+)',err)]
        rows.append((o,parts))
    return rows
