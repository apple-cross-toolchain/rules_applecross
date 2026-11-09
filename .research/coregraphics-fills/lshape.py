import subprocess,random,os,sys
S=4096
def run(seed=1,n=40,K=48,W=24,H=24,tool='/tmp/cgclean/target/release/cgclean'):
    random.seed(seed); stats={'pix':0,'bad':0}; ex=[]
    for _ in range(n):
        x0=random.uniform(2,6); x1=random.uniform(12,20); xm=random.uniform(x0+1,x1-1)
        y0=random.uniform(2,6); y1=random.uniform(12,20); ym=random.uniform(y0+1,y1-1)
        P=[(x0,y0),(x1,y0),(x1,ym),(xm,ym),(xm,y1),(x0,y1)]   # L shape
        P=[(round(x*S)/S,round(y*S)/S) for x,y in P]
        inp=''
        for k in range(K):
            Q=[(x+k/S,y) for x,y in P]; inp+='m %r %r\n'%Q[0]+''.join('l %r %r\n'%p for p in Q[1:])+'h\nf\nn\n'
        g=lambda t:[o.split() for o in subprocess.run([t,str(W),str(H)],input=inp,capture_output=True,text=True).stdout.split('--\n')[:K]]
        A=g('./cov'); B=g(tool)
        for k,(a,b) in enumerate(zip(A,B)):
            for i,(p,q) in enumerate(zip(a,b)):
                if p not in ('0','255') or q not in ('0','255'):
                    stats['pix']+=1
                    if p!=q:
                        stats['bad']+=1
                        if len(ex)<8: ex.append((k,i%W,H-1-i//W,p,q,[tuple(round(c,4) for c in v) for v in P]))
    return stats,ex
if __name__=='__main__':
    st,ex=run(); print(st)
    for e in ex: print(e)
