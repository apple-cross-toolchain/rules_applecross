import sys,math; sys.path.insert(0,'/tmp/cgw'); from offs import *
def find_t(q,d,p):
    best=min(((math.dist(off(q,i/2000,d),p),i/2000) for i in range(0,2001)))
    lo,hi=max(0,best[1]-1/2000),min(1,best[1]+1/2000)
    for _ in range(60):
        a=lo+(hi-lo)/3; b=hi-(hi-lo)/3
        if math.dist(off(q,a,d),p)<math.dist(off(q,b,d),p): hi=b
        else: lo=a
    t=(lo+hi)/2; return t,math.dist(off(q,t,d),p)
def splits(q,w):
    n,els=outline([q],w)[0]
    d=w/2
    ts=[]
    for e in els[1:1+n-1]:
        p=(float(e[-2]),float(e[-1])); ts.append(find_t(q,d,p))
    return n,ts
def inflections(q):
    # solve cross(B',B'')=0
    import cmath
    ax=-q[0][0]+3*q[1][0]-3*q[2][0]+q[3][0]; bx=3*q[0][0]-6*q[1][0]+3*q[2][0]; cx=-3*q[0][0]+3*q[1][0]
    ay=-q[0][1]+3*q[1][1]-3*q[2][1]+q[3][1]; by=3*q[0][1]-6*q[1][1]+3*q[2][1]; cy=-3*q[0][1]+3*q[1][1]
    # B'=3a t^2+2b t+c ; B''=6a t+2b ; cross = (3a t^2+2bt+c)x(6at+2b)
    A=6*(ax*by-ay*bx)*1.0; B=6*(ax*cy-ay*cx); C=2*(bx*cy-by*cx)
    # cross = 6(a×b) t^2 + 6(a×c) t + 2(b×c)
    if abs(A)<1e-12: return [-C/B] if B else []
    D=B*B-4*A*C
    if D<0: return []
    return sorted([(-B-math.sqrt(D))/(2*A),(-B+math.sqrt(D))/(2*A)])
if __name__=='__main__':
    tests={'S':[(0,0),(10,0),(10,20),(20,20)],'S2':[(0,0),(20,0),(0,20),(20,20)],'flat':[(0,0),(10,1),(20,-1),(30,0)],'loop':[(0,0),(30,20),(-10,20),(20,0)],'asym2':[(0,0),(19.5,0),(20,0.5),(20,20)],'sharp':[(0,0),(20,0),(20,0),(20,20)]}
    for k,q in tests.items():
        for w in (2,6):
            n,ts=splits(q,w)
            print(k,w,n,['t=%.5f(err %.1e)'%x for x in ts],'inflections',['%.5f'%v for v in inflections(q)])
