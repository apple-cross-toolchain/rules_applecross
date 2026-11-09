import os,subprocess,sys,random,math,pickle; sys.path.insert(0,'/tmp/cg'); import rsc, dense
random.seed(7); N=8
tris=[[(random.uniform(0,N),random.uniform(0,N)) for _ in range(3)] for _ in range(1500)]
TRI=''.join('m %r %r\nl %r %r\nl %r %r\nh\nf\nn\n'%(t[0]+t[1]+t[2]) for t in tris)
random.seed(1); MUL=''
for _ in range(200):
    polys=[[(random.uniform(1,15),random.uniform(1,15)) for _ in range(3)] for _ in range(2)]
    MUL+=''.join('m %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\n' for P in polys)+'f\nn\n'
G=[subprocess.run(['./glyphs',f,s,'2.0','5.0',t],capture_output=True,text=True).stdout+'n\n' for f,s,t in (('Helvetica','20','Rg&8'),('Times New Roman','24','Qa@'),('Helvetica','11','abcdefgh'),('Georgia','30','Sg'),('Helvetica','20','AXWNZ'))]
def run(env):
    old=dict(os.environ); os.environ.update(env)
    try:
        r=dict(tris=rsc.compare(TRI,8,8,1500)[0],multi=rsc.compare(MUL,16,16,200)[0],dense=[rsc.compare(dense.D[k],32,32,20)[0] for k in (64,256,512)],glyphs=[rsc.compare(g,64,32,1)[0] for g in G])
        r['total']=r['tris']+r['multi']+sum(r['dense'])+sum(r['glyphs']); return r
    finally: os.environ.clear(); os.environ.update(old)
