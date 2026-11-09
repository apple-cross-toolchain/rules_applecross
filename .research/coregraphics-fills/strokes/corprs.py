import sys,os,glob,subprocess,collections; sys.path.insert(0,'/tmp/cg'); os.chdir('/tmp/cg')
import corpus_rs
def grid(o): return [r.split() for r in o.split('--\n')[0].strip().split('\n')]
for sc in ('1','2','3'):
    tot=collections.Counter()
    for f in sorted(glob.glob('/tmp/accorp/w/*/b/*.pdf')):
        W,H,bl=corpus_rs.blocks(f,sc,True)
        for b in bl:
            lines=b.strip().split('\n'); _,w,j,c=lines[-1].split(); path='\n'.join(lines[:-1])
            if float(w)<=1: continue
            st=grid(subprocess.run(['./covs',str(W),str(H)],input='w %s\nj %s\nk %s\n%s\ns\n'%(w,j,c,path),capture_output=True,text=True).stdout)
            B=grid(subprocess.run(['/tmp/cgclean/target/release/cgclean',str(W),str(H)],input='%s\ns %s %s %s\n'%(path,w,j,c),capture_output=True,text=True).stdout)
            tot['wrong']+=sum(1 for ra,rb in zip(st,B) for x,y in zip(ra,rb) if x!=y)
            tot['edge']+=sum(1 for r in st for v in r if v not in ('0','255')); tot['blocks']+=1
    print('scale',sc,dict(tot),flush=True)
