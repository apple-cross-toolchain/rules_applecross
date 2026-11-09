import sys,subprocess,math,random,statistics,pickle,os; sys.path.insert(0,'/tmp/cg'); import raster
N=48
def gen(seed=3,n=30):
    fn='/tmp/cg/ngons%d.pkl'%seed
    if os.path.exists(fn): return pickle.load(open(fn,'rb'))
    random.seed(seed); G=[]
    for _ in range(n):
        k=random.randint(12,60); r=random.uniform(5,20); cx,cy=random.uniform(23,25),random.uniform(23,25); ph=random.uniform(0,6.28)
        G.append([(cx+r*math.cos(ph+2*math.pi*i/k),cy+r*math.sin(ph+2*math.pi*i/k)) for i in range(k)])
    inp=''.join('m %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\nf\nn\n' for P in G)
    out=subprocess.run(['./cov',str(N),str(N)],input=inp,capture_output=True,text=True).stdout.split('--\n')
    D=[(P,[list(map(int,r.split())) for r in o.strip().split('\n')][::-1]) for P,o in zip(G,out)]
    pickle.dump(D,open(fn,'wb')); return D
def measure(D):
    vr=[];nv=[];miss=0;tot=0
    for P,obs in D:
        acc=raster.fill_path_wind([P],N,N)
        Pq=[(raster.vq(x),raster.vq(y)) for x,y in P]
        vrows={math.floor(p[1]) for p in Pq}
        for y in range(N):
            for x in range(N):
                o=obs[y][x]
                if not 0<o<255: continue
                a=min(1,max(0,acc[y][x]))*256
                (vr if y in vrows else nv).append(o-a); tot+=1; miss+=math.floor(a+1e-9)!=o
    return round(statistics.mean(vr),4),len(vr),round(statistics.mean(nv),4),len(nv),miss,tot
if __name__=='__main__':
    print(measure(gen()))
