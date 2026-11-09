import sys,os,glob,subprocess,collections; sys.path.insert(0,'/tmp/cgw'); sys.path.insert(0,'/tmp/cg'); os.chdir('/tmp/cg')
import corpus_rs, stroke2
from offs import outline
def grid(o): return [r.split() for r in o.split('--\n')[0].strip().split('\n')]
def unsplit_only(path,w):
    cur=None
    for l in path.split('\n'):
        t=l.split()
        if not t: continue
        if t[0] in ('m','l'): cur=(float(t[1]),float(t[2]))
        elif t[0]=='c':
            v=list(map(float,t[1:])); q=[cur,(v[0],v[1]),(v[2],v[3]),(v[4],v[5])]
            n,els=outline([q],w)[0]
            if n!=1 or sum(1 for e in els if e[0]=='c')!=2: return False
            cur=q[3]
    return True
def run(sc):
    tot=collections.Counter(); per=[]
    for f in sorted(glob.glob('/tmp/accorp/w/*/b/*.pdf')):
        W,H,bl=corpus_rs.blocks(f,sc,True)
        for b in bl:
            lines=b.strip().split('\n'); _,w,j,c=lines[-1].split(); path='\n'.join(lines[:-1])
            if float(w)<=1: continue
            us=unsplit_only(path,float(w))
            st=grid(subprocess.run(['./covs',str(W),str(H)],input='w %s\nj %s\nk %s\n%s\ns\n'%(w,j,c,path),capture_output=True,text=True).stdout)
            mine=''.join(stroke2.to_cov(stroke2.stroke(segs,float(w),int(j),int(c),closed)) for segs,closed in stroke2.parse(path))
            B=grid(subprocess.run(['/tmp/cgclean/target/release/cgclean',str(W),str(H)],input=mine+'f\n',capture_output=True,text=True).stdout)
            eb=sum(1 for ra,rb in zip(st,B) for x,y in zip(ra,rb) if x!=y)
            edge=sum(1 for r in st for v in r if v not in ('0','255'))
            k='unsplit' if us else 'split'
            tot[k+'_wrong']+=eb; tot[k+'_edge']+=edge; tot[k+'_blocks']+=1
            if eb and us: per.append((f.split('/')[-1],eb,edge))
    return tot,per
if __name__=='__main__':
    for sc in sys.argv[1:]:
        tot,per=run(sc); print(sc,dict(tot)); print('  unsplit with errors',per[:10])
