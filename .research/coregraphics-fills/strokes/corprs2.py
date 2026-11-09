import sys,os,glob,subprocess,collections; sys.path.insert(0,'/tmp/cg'); os.chdir('/tmp/cg')
import corpus_rs
def grid(o): return [r.split() for r in o.split('--\n')[0].strip().split('\n')]
for sc in ('1','2','3'):
    for f in sorted(glob.glob('/tmp/accorp/w/*/b/*.pdf')):
        W,H,bl=corpus_rs.blocks(f,sc,True)
        for i,b in enumerate(bl):
            lines=b.strip().split('\n'); _,w,j,c=lines[-1].split(); path='\n'.join(lines[:-1])
            if float(w)<=1: continue
            st=grid(subprocess.run(['./covs',str(W),str(H)],input='w %s\nj %s\nk %s\n%s\ns\n'%(w,j,c,path),capture_output=True,text=True).stdout)
            B=grid(subprocess.run(['/tmp/cgclean/target/release/cgclean',str(W),str(H)],input='%s\ns %s %s %s\n'%(path,w,j,c),capture_output=True,text=True).stdout)
            e=sum(1 for ra,rb in zip(st,B) for x,y in zip(ra,rb) if x!=y)
            if e: print(sc,f.split('/')[-1],i,e)
