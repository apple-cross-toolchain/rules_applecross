import sys,math,random; sys.path.insert(0,'/tmp/cgw'); from offs import bez, d1, norm; from offfit import inter
def piece(q,d):
    """CoreGraphics' single-piece offset of cubic q at distance d (left)."""
    u0=norm(d1(q,0)); u3=norm(d1(q,1)); um=norm(d1(q,.5)); M=bez(q,.5)
    ang=lambda a,b: math.atan2(a[0]*b[1]-a[1]*b[0],a[0]*b[0]+a[1]*b[1])
    t1=math.tan(ang(u0,um)/2); t2=math.tan(ang(um,u3)/2)
    X1=inter(q[0],u0,M,um); X2=inter(q[3],u3,M,um)
    h1=(q[1][0]-q[0][0],q[1][1]-q[0][1]); h2=(q[2][0]-q[3][0],q[2][1]-q[3][1])
    C1=(2*X1[0]-q[0][0]-h1[0]/2,2*X1[1]-q[0][1]-h1[1]/2); C2=(2*X2[0]-q[3][0]-h2[0]/2,2*X2[1]-q[3][1]-h2[1]/2)
    L1=(C1[0]-q[0][0])*u0[0]+(C1[1]-q[0][1])*u0[1]; L2=-((C2[0]-q[3][0])*u3[0]+(C2[1]-q[3][1])*u3[1])
    G=lambda a,b: (4/3)*a+(b*b-a*a)/(6*a*(1+b*b))
    L1-=G(t1,t2)*d; L2-=G(t2,t1)*d
    n0=(-u0[1]*d,u0[0]*d); n3=(-u3[1]*d,u3[0]*d)
    P0=(q[0][0]+n0[0],q[0][1]+n0[1]); P3=(q[3][0]+n3[0],q[3][1]+n3[1])
    return [P0,(P0[0]+u0[0]*L1,P0[1]+u0[1]*L1),(P3[0]-u3[0]*L2,P3[1]-u3[1]*L2),P3]
if __name__=='__main__':
    from offfit2 import get_cg
    random.seed(int(sys.argv[1]) if len(sys.argv)>1 else 3); n=ok=0; worst=0
    for _ in range(3000):
        q=[(random.uniform(0,20),random.uniform(0,20)) for _ in range(4)]
        w=random.choice([1,1.5,2,3,4,8]); cg=get_cg(q,w)
        if not cg: continue
        n+=1; m=piece(q,w/2); e=max(math.dist(a,b) for a,b in zip(m,cg)); worst=max(worst,e); ok+=e<1e-9
        if n>=150: break
    print('unsplit',n,'exact(1e-9)',ok,'worst %.3e'%worst)
