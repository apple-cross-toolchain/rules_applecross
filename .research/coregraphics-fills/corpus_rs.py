import sys,subprocess,glob,os; sys.path.insert(0,'/tmp/cg'); import rsc
def blocks(f,scale,strokes):
    r=subprocess.run(['./pdfpaths',f,scale]+(['strokes'] if strokes else []),capture_output=True,text=True)
    size=r.stderr.split()[1:3]; W,H=int(float(size[0])+0.999),int(float(size[1])+0.999)
    return W,H,[b for b in r.stdout.split('n\n') if b.strip()]
def fills(env,scale='2'):
    old=dict(os.environ); os.environ.update(env); tb=te=0
    for f in sorted(glob.glob('/tmp/accorp/w/*/b/*.pdf')):
        W,H,bl=blocks(f,scale,False)
        for b in bl: x,e=rsc.compare(b+'n\n',W,H,1); tb+=x; te+=e
    os.environ.clear(); os.environ.update(old); return tb,te
def strokes(env,scale='2'):
    old=dict(os.environ); os.environ.update(env); tb=te=0
    for f in sorted(glob.glob('/tmp/accorp/w/*/b/*.pdf')):
        W,H,bl=blocks(f,scale,True)
        for b in bl:
            lines=b.strip().split('\n'); _,w,j,c=lines[-1].split(); path='\n'.join(lines[:-1])
            if float(w)<=1: continue
            st=subprocess.run(['./covs',str(W),str(H)],input='w %s\nj %s\nk %s\n%s\ns\n'%(w,j,c,path),capture_output=True,text=True).stdout.split('--\n')[0]
            ol=subprocess.run(['./outline'],input='%s\ns %s %s %s\n'%(path,w,j,c),capture_output=True,text=True).stdout.split('--')[0]
            m=subprocess.run(['/tmp/cgprobe/target/release/cgprobe',str(W),str(H)],input=ol+'\nf\n',capture_output=True,text=True).stdout.split('--\n')[0]
            A=[r.split() for r in st.strip().split('\n')]; B=[r.split() for r in m.strip().split('\n')]
            tb+=sum(1 for ra,rb in zip(A,B) for x,y in zip(ra,rb) if x!=y); te+=sum(1 for r in A for v in r if v not in ('0','255'))
    os.environ.clear(); os.environ.update(old); return tb,te
if __name__=='__main__':
    for env in ({},{'CG_PQ':'4096:floor','CG_PQ_V':'chainrow'}):
        for sc in ('1','2','3'):
            print(env,'scale',sc,'fills',fills(env,sc),'strokes',strokes(env,sc))
