import sys,struct,math; sys.path.insert(0,'/tmp/cgw'); import flatcg as F
f32=lambda v: struct.unpack('f',struct.pack('f',v))[0]
T=0.05/0.75
def flat(q,out,m,depth=0):
    d=lambda a,b,c: max(abs(a[0]-2*b[0]+c[0]),abs(a[1]-2*b[1]+c[1]))
    if max(d(q[0],q[1],q[2]),d(q[1],q[2],q[3]))<=T or depth>12: return
    a,b,c=m(q[0],q[1]),m(q[1],q[2]),m(q[2],q[3]); l,r=m(a,b),m(b,c); f=m(l,r)
    flat([q[0],a,l,f],out,m,depth+1); out.append(f); flat([f,r,c,q[3]],out,m,depth+1)
md=lambda a,b: ((a[0]+b[0])/2,(a[1]+b[1])/2)
mf=lambda a,b: (f32((a[0]+b[0])/2),f32((a[1]+b[1])/2))
S=4096
def variant(name):
    pre={'d':lambda p:p,'f':lambda p:(f32(p[0]),f32(p[1])),'fix':lambda p:(math.floor(f32(p[0])*S+.5)/S,math.floor(f32(p[1])*S+.5)/S),'fixd':lambda p:(math.floor(p[0]*S+.5)/S,math.floor(p[1]*S+.5)/S)}
    cp,mid,post=name.split('/')
    def build(s):
        s=[pre[cp](p) for p in s]
        m={'d':md,'f':mf}[mid]
        P=[s[0]]; flat(s[:4],P,m); P.append(s[3]); flat([s[3],s[4],s[5],s[0]],P,m)
        P=[pre[post](p) for p in P]
        return 'm %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\nf\nn\n'
    return build
if __name__=='__main__':
    C=F.gen(1,300,2,18)+F.gen(2,300,1,6)
    ref=F.render([F.curve_inp(s) for s in C])
    for name in sys.argv[1:]:
        got=F.render([variant(name)(s) for s in C])
        bad=[i for i in range(len(C)) if ref[i]!=got[i]]
        print(name,len(C)-len(bad),'/',len(C),bad[:10],flush=True)
