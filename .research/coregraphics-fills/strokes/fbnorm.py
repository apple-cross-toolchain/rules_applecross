import sys,math,ast; sys.path.insert(0,'/tmp/cgw'); from offs import outline,bez,d1,norm; from offs6 import sub
import stroke2
def coord_infl(q): return stroke2._coord_infl(q)
cases=[(1.5,[(25.688, 39.844), (26.086, 39.336), (26.609, 39.289), (27.094, 39.047)]),(1.5,[(29.672, 32.578), (29.855, 32.328), (30.133, 32.18), (30.328, 31.969)]),(1.5,[(47.766, 31.969), (47.961, 32.18), (48.238, 32.328), (48.422, 32.578)]),(1.5,[(50.953, 39.047), (51.426, 39.289), (51.926, 39.348), (52.312, 39.844)])]
for w,q in cases:
    ti=coord_infl(q); s=sub(q,ti[0],ti[1]); d=w/2
    n,els=outline([q],w)[0]
    C=tuple(map(float,els[2][1:3]))   # middle piece first control point
    mid=((s[1][0]+s[2][0])/2,(s[1][1]+s[2][1])/2)
    v=((C[0]-mid[0])/d,(C[1]-mid[1])/d)
    cands={'chord':(s[3][0]-s[0][0],s[3][1]-s[0][1]),'u0':d1(s,0),'u3':d1(s,1),"B'(½)":d1(s,.5),'P2-P1':(s[2][0]-s[1][0],s[2][1]-s[1][1]),
           'whole chord':(q[3][0]-q[0][0],q[3][1]-q[0][1]),"whole B'(½)":d1(q,.5),'P0->mid':(mid[0]-s[0][0],mid[1]-s[0][1]),'mid->P3':(s[3][0]-mid[0],s[3][1]-mid[1])}
    u0=norm(d1(s,0)); u3=norm(d1(s,1)); n0=(-u0[1],u0[0]); n3=(-u3[1],u3[0])
    dot=n0[0]*n3[0]+n0[1]*n3[1]
    miter=((n0[0]+n3[0])/(1+dot),(n0[1]+n3[1])/(1+dot))
    um=norm(d1(s,.5)); nm=(-um[1],um[0])
    # miter of n0 and nm, nm and n3
    def mit(a,b):
        dd=a[0]*b[0]+a[1]*b[1]; return ((a[0]+b[0])/(1+dd),(a[1]+b[1])/(1+dd))
    extra={'miter n0,n3':miter,'avg n0,n3':((n0[0]+n3[0])/2,(n0[1]+n3[1])/2),'miter n0,nm':mit(n0,nm),'miter nm,n3':mit(nm,n3)}
    from offfit import inter
    P0p=(s[0][0]+n0[0]*d,s[0][1]+n0[1]*d); P3p=(s[3][0]+n3[0]*d,s[3][1]+n3[1]*d); M=bez(s,.5); Mp=(M[0]+nm[0]*d,M[1]+nm[1]*d)
    Xd=((8*Mp[0]-P0p[0]-P3p[0])/6,(8*Mp[1]-P0p[1]-P3p[1])/6)
    pts={'Xmidinterp':Xd,'X1p':inter(P0p,u0,Mp,um),'X2p':inter(P3p,u3,Mp,um),'Kp':inter(P0p,u0,P3p,u3),'Mp':Mp,'X1':inter(s[0],u0,M,um),'X2':inter(s[3],u3,M,um)}
    best=sorted([(math.dist(p,C),k) for k,p in pts.items() if p])
    print('   C',C,'mid',mid)
    print('|v| %.9f'%math.hypot(*v),'best',[('%s %.1e'%(k,e)) for e,k in best[:4]])
