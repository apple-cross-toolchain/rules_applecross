import sys,math,random,collections; sys.path.insert(0,'/tmp/cgw'); from offs import outline,bez,d1,norm; from offfit import inter; import stroke2
def handles(q,d):
    u0,u3=stroke2.seg_dirs(('c',)+tuple(q)); um=norm(d1(q,.5)); M=bez(q,.5)
    X1=inter(q[0],u0,M,um); X2=inter(q[3],u3,M,um)
    h1=(q[1][0]-q[0][0],q[1][1]-q[0][1]); h2=(q[2][0]-q[3][0],q[2][1]-q[3][1])
    L1=(2*(X1[0]-q[0][0])-h1[0]/2)*u0[0]+(2*(X1[1]-q[0][1])-h1[1]/2)*u0[1]
    L2=-((2*(X2[0]-q[3][0])-h2[0]/2)*u3[0]+(2*(X2[1]-q[3][1])-h2[1]/2)*u3[1])
    ang=lambda x,y: math.atan2(x[0]*y[1]-x[1]*y[0],x[0]*y[0]+x[1]*y[1])
    t1=math.tan(ang(u0,um)/2); t2=math.tan(ang(um,u3)/2)
    G=lambda a,b:(4/3)*a+(b*b-a*a)/(6*a*(1+b*b))
    p0=(q[0][0]-u0[1]*d,q[0][1]+u0[0]*d); p3=(q[3][0]-u3[1]*d,q[3][1]+u3[0]*d)
    L1d=L1-G(t1,t2)*d; L2d=L2-G(t2,t1)*d
    reg1=(p0[0]+u0[0]*L1d,p0[1]+u0[1]*L1d); reg2=(p3[0]-u3[0]*L2d,p3[1]-u3[1]*L2d)
    Mo=(M[0]-um[1]*d,M[1]+um[0]*d); fb=((8*Mo[0]-p0[0]-p3[0])/6,(8*Mo[1]-p0[1]-p3[1])/6)
    return L1,L2,L1d,L2d,reg1,reg2,fb
random.seed(3); st=collections.Counter()
for _ in range(3000):
    q=[(random.uniform(0,20),random.uniform(0,20)) for _ in range(4)]
    w=random.choice([1.5,2,3,4,6,8])
    n,els=outline([q],w)[0]
    if n!=1 or sum(1 for e in els if e[0]=='c')!=2: continue
    try: L1,L2,L1d,L2d,reg1,reg2,fb=handles(q,w/2)
    except Exception: continue
    c=list(map(float,els[1][1:])); C1=(c[0],c[1]); C2=(c[2],c[3])
    for L,Ld,reg,C in ((L1,L1d,reg1,C1),(L2,L2d,reg2,C2)):
        kind='reg' if math.dist(C,reg)<1e-8 else ('fb' if math.dist(C,fb)<1e-8 else 'other')
        st[('L0<0' if L<0 else 'L0>=0','Ld<0' if Ld<0 else 'Ld>=0',kind)]+=1
for k,v in sorted(st.items()): print(k,v)
print('--- split pieces')
from splitcmp import cg_ts
from offs6 import sub
random.seed(4); st=collections.Counter()
for _ in range(500):
    q=[(random.uniform(0,10),random.uniform(0,10)) for _ in range(4)]
    w=random.choice([1.5,2,3])
    n,els=outline([q],w)[0]
    if n<2 or n>6: continue
    if any(e[0]=='l' for e in els[1:n+1]): continue
    try: L,R=cg_ts(q,w)
    except Exception: continue
    if len(L)!=n-1: continue
    ts=[0.0]+L+[1.0]
    for k in range(n):
        s=sub(q,ts[k],ts[k+1])
        try: L1,L2,L1d,L2d,reg1,reg2,fb=handles(s,w/2)
        except Exception: continue
        c=list(map(float,els[1+k][1:])); C1=(c[0],c[1]); C2=(c[2],c[3])
        for Lz,Ld,reg,C in ((L1,L1d,reg1,C1),(L2,L2d,reg2,C2)):
            dr,df=math.dist(C,reg),math.dist(C,fb)
            kind='reg' if dr<2e-3 and dr<df else ('fb' if df<2e-3 else 'other')
            st[('L0<0' if Lz<0 else 'L0>=0','Ld<0' if Ld<0 else 'Ld>=0',kind)]+=1
for k,v in sorted(st.items()): print(k,v)
