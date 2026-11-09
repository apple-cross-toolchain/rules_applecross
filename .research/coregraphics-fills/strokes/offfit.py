import sys,math,random; sys.path.insert(0,'/tmp/cgw'); from offs import outline, bez, d1, norm
def inter(p,u,q,v):
    det=u[0]*v[1]-u[1]*v[0]
    if abs(det)<1e-14: return None
    s=((q[0]-p[0])*v[1]-(q[1]-p[1])*v[0])/det
    return (p[0]+s*u[0],p[1]+s*u[1])
def fit(q,d,kind='A'):
    u0=norm(d1(q,0)); u3=norm(d1(q,1)); um=norm(d1(q,.5))
    n=lambda u:(-u[1]*d,u[0]*d)
    P0=(q[0][0]+n(u0)[0],q[0][1]+n(u0)[1]); P3=(q[3][0]+n(u3)[0],q[3][1]+n(u3)[1])
    M=bez(q,.5); M=(M[0]+n(um)[0],M[1]+n(um)[1])
    X1=inter(P0,u0,M,um); X2=inter(P3,u3,M,um)
    h1=(q[1][0]-q[0][0],q[1][1]-q[0][1]); h2=(q[2][0]-q[3][0],q[2][1]-q[3][1])
    C1=(P0[0]+2*(X1[0]-P0[0])-h1[0]/2,P0[1]+2*(X1[1]-P0[1])-h1[1]/2)
    C2=(P3[0]+2*(X2[0]-P3[0])-h2[0]/2,P3[1]+2*(X2[1]-P3[1])-h2[1]/2)
    return [P0,C1,C2,P3]
if __name__=='__main__':
    random.seed(1); n=0; ok=0; worst=0
    for _ in range(300):
        q=[(random.uniform(0,20),random.uniform(0,20)) for _ in range(4)]
        w=float(sys.argv[1]) if len(sys.argv)>1 else 0.02
        cnt,els=outline([q],w)[0]
        if cnt!=1: continue
        n+=1
        c=list(map(float,els[1][1:])); cg=[tuple(map(float,els[0][1:])),(c[0],c[1]),(c[2],c[3]),(c[4],c[5])]
        mine=fit(q,w/2)
        e=max(math.dist(a,b) for a,b in zip(cg,mine)); worst=max(worst,e); ok+=e<1e-9
    print('w',sys.argv[1:],'unsplit',n,'exact',ok,'worst',worst)

def beta_needed(q,w):
    d=w/2
    cnt,els=outline([q],w)[0]
    if cnt!=1: return None
    c=list(map(float,els[1][1:])); C1=(c[0],c[1]); C2=(c[2],c[3])
    u0=norm(d1(q,0)); u3=norm(d1(q,1)); um=norm(d1(q,.5))
    n=lambda u:(-u[1]*d,u[0]*d)
    P0=(q[0][0]+n(u0)[0],q[0][1]+n(u0)[1]); P3=(q[3][0]+n(u3)[0],q[3][1]+n(u3)[1])
    M=bez(q,.5); M=(M[0]+n(um)[0],M[1]+n(um)[1])
    X1=inter(P0,u0,M,um); X2=inter(P3,u3,M,um)
    h1=(q[1][0]-q[0][0],q[1][1]-q[0][1]); h2=(q[2][0]-q[3][0],q[2][1]-q[3][1])
    r1=(2*(X1[0]-P0[0])-(C1[0]-P0[0]),2*(X1[1]-P0[1])-(C1[1]-P0[1]))
    r2=(2*(X2[0]-P3[0])-(C2[0]-P3[0]),2*(X2[1]-P3[1])-(C2[1]-P3[1]))
    b1=2*(r1[0]*h1[0]+r1[1]*h1[1])/(h1[0]**2+h1[1]**2); b2=2*(r2[0]*h2[0]+r2[1]*h2[1])/(h2[0]**2+h2[1]**2)
    perp1=abs(r1[0]*h1[1]-r1[1]*h1[0])/math.hypot(*h1)
    return b1,b2,perp1
