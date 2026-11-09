import sys,math,ast; sys.path.insert(0,'/tmp/cgw'); from offs import outline,bez,d1,norm; from offs6 import sub
from splitcmp import cg_ts
from offs5 import inflections
from offfit import inter
ang=lambda a,b: math.degrees(math.atan2(a[0]*b[1]-a[1]*b[0],a[0]*b[0]+a[1]*b[1]))
rows=[]
for line in open('/tmp/cgw/splitlist.txt'):
    w,q=ast.literal_eval(line)
    n,els=outline([q],w)[0]
    L,R=cg_ts(q,w)
    ts=[0.0]+L+[1.0]
    for k,e in enumerate(els[1:n+1]):
        v=list(map(float,e[1:])); same=abs(v[0]-v[2])<1e-12 and abs(v[1]-v[3])<1e-12
        a,b=ts[k],ts[k+1]; s=sub(q,a,b)
        try:
            u0=norm(d1(s,0)); um=norm(d1(s,.5)); u3=norm(d1(s,1))
            p1=ang(u0,um); p2=ang(um,u3)
        except Exception: p1=p2=float('nan')
        infl=[t for t in inflections(s) if 0<t<1] if True else []
        ch=(s[3][0]-s[0][0],s[3][1]-s[0][1]); Lc=math.hypot(*ch)
        dist=lambda p: abs((p[0]-s[0][0])*ch[1]-(p[1]-s[0][1])*ch[0])/Lc if Lc>0 else math.dist(p,s[0])
        flat=max(dist(s[1]),dist(s[2]))
        dd=lambda a,b,c: max(abs(a[0]-2*b[0]+c[0]),abs(a[1]-2*b[1]+c[1]))
        dd2=max(dd(s[0],s[1],s[2]),dd(s[1],s[2],s[3]))
        rows.append((same,round(a,4),round(b,4),'flat %.5f'%flat,'dd %.5f'%dd2,'chord %.4f'%Lc))
for r in sorted(rows,key=lambda r: float(r[3].split()[1])): print(r)
