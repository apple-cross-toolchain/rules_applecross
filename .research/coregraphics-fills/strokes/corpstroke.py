import sys,os,glob,subprocess,collections; sys.path.insert(0,'/tmp/cgw'); sys.path.insert(0,'/tmp/cg'); os.chdir('/tmp/cg')
import corpus_rs, stroke2
def grid(o): return [r.split() for r in o.split('--\n')[0].strip().split('\n')]
def run(sc):
    tot=collections.Counter(); per=[]
    for f in sorted(glob.glob('/tmp/accorp/w/*/b/*.pdf')):
        W,H,bl=corpus_rs.blocks(f,sc,True)
        for b in bl:
            lines=b.strip().split('\n'); _,w,j,c=lines[-1].split(); path='\n'.join(lines[:-1])
            if float(w)<=1: continue
            st=grid(subprocess.run(['./covs',str(W),str(H)],input='w %s\nj %s\nk %s\n%s\ns\n'%(w,j,c,path),capture_output=True,text=True).stdout)
            ol=subprocess.run(['./outline'],input='%s\ns %s %s %s\n'%(path,w,j,c),capture_output=True,text=True).stdout.split('--')[0]
            mine=''.join(stroke2.to_cov(stroke2.stroke(segs,float(w),int(j),int(c),closed)) for segs,closed in stroke2.parse(path))
            A=grid(subprocess.run(['/tmp/cgclean/target/release/cgclean',str(W),str(H)],input=ol+'\nf\n',capture_output=True,text=True).stdout)
            B=grid(subprocess.run(['/tmp/cgclean/target/release/cgclean',str(W),str(H)],input=mine+'f\n',capture_output=True,text=True).stdout)
            ea=sum(1 for ra,rb in zip(st,A) for x,y in zip(ra,rb) if x!=y); eb=sum(1 for ra,rb in zip(st,B) for x,y in zip(ra,rb) if x!=y)
            edge=sum(1 for r in st for v in r if v not in ('0','255'))
            tot['cg_outline']+=ea; tot['mine']+=eb; tot['edge']+=edge
            if eb>ea: per.append((f.split('/')[-1],eb,ea,edge))
    return tot,per
if __name__=='__main__':
    for sc in sys.argv[1:]:
        tot,per=run(sc); print(sc,dict(tot)); print('  worst',sorted(per,key=lambda x:-x[1])[:8])
