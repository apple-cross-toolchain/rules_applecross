import subprocess,os,collections,math,sys,random; sys.path.insert(0,'/tmp/cg'); import raster
W,H=32,32; S=4096
random.seed(4); cases=[]
for _ in range(1500):
    L0=(random.uniform(8,14),random.uniform(1.2,2.8)); L1=(L0[0]+random.uniform(-4,4),random.uniform(28,30.5))
    yb=random.uniform(4,27)   # closing-side vertex makes a band on the opening edge
    cases.append((L0,L1,yb))
inp=''.join('m %r %r\nl %r %r\nl %r %r\nl %r %r\nl %r %r\nh\nf\nn\n'%(L0+(28.7,1.0)+(29.9,yb)+(28.9,31.0)+L1) for L0,L1,yb in cases)
g=lambda tool,env={}:[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run([tool,str(W),str(H)],input=inp,capture_output=True,text=True,env={**os.environ,**env}).stdout.split('--\n')[:len(cases)]]
O=g('./cov'); offs=(1,0,-1,-2,-3)
R={o:g('/tmp/cgprobe/target/release/cgprobe',{'CG_ACC':'1','CG_BANDOFF_O':str(o)}) for o in offs}
cls=collections.Counter(); rows=[]
for i,(L0,L1,yb) in enumerate(cases):
    y=int(yb); o=O[i]
    for x in range(0,22):
        if not 0<o[y][x]<255: continue
        ok=tuple(off for off in offs if R[off][i][y][x]==o[y][x])
        if len(ok)==len(offs): continue
        bx=round(raster.vq(L0[0])*S); by=round(raster.vq(L0[1])*S); tx=round(raster.vq(L1[0])*S); ty=round(raster.vq(L1[1])*S)
        mr=int((tx-bx)*S/(ty-by)); yv=round(raster.vq(yb)*S); fr=((bx*S+(yv-by)*mr)/S)%1
        rows.append((ok,round(fr,3),round(yb-y,3))); cls[ok]+=1
print(cls.most_common())
for k,_ in cls.most_common(6):
    v=sorted(fr for ok,fr,_ in rows if ok==k); print(k,len(v),'frac',v[:3],v[-3:])
