import sys,math,ast; sys.path.insert(0,'/tmp/cgw'); from offs import outline,bez,d1,norm,off; from offs6 import sub
from splitcmp import cg_ts
from bnd2 import nearest
import stroke2
rows=[]
for line in open('/tmp/cgw/splitlist.txt'):
    w,q=ast.literal_eval(line)
    n,els=outline([q],w)[0]
    L,R=cg_ts(q,w)
    ts=[0.0]+L+[1.0]; d=w/2
    for k,e in enumerate(els[1:n+1]):
        v=list(map(float,e[1:])); same=abs(v[0]-v[2])<1e-12 and abs(v[1]-v[3])<1e-12
        a,b=ts[k],ts[k+1]; s=sub(q,a,b)
        try:
            reg=stroke2.seg_off_one(('c',)+tuple(s),d)[0]
            u0,u3=stroke2.seg_dirs(('c',)+tuple(s)); p0=(s[0][0]-u0[1]*d,s[0][1]+u0[0]*d)
            creg=[p0,reg[1],reg[2],reg[3]]
            um=norm(d1(s,.5)); m=((s[1][0]+s[2][0])/2,(s[1][1]+s[2][1])/2); fb=(m[0]-um[1]*d,m[1]+um[0]*d)
            cfb=[p0,fb,fb,reg[3]]
            er=max(nearest(lambda x:bez(creg,x),off(s,t,d)) for t in (0.1,0.3,0.5,0.7,0.9))
            ef=max(nearest(lambda x:bez(cfb,x),off(s,t,d)) for t in (0.1,0.3,0.5,0.7,0.9))
        except Exception as ex: er=ef=float('nan')
        rows.append((same, ef<er, round(er,6), round(ef,6), round(a,4),round(b,4)))
ok=sum(1 for r in rows if r[0]==r[1]); print('match',ok,'of',len(rows))
for r in rows:
    if r[0]!=r[1]: print(r)
