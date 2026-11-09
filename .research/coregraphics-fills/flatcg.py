import sys,os,random,math,subprocess; os.chdir('/tmp/cg')
N=40
def mid(a,b,mode):
    if mode=='f32':
        import struct
        f=lambda v: struct.unpack('f',struct.pack('f',v))[0]
        return (f((f(a[0])+f(b[0]))*0.5),f((f(a[1])+f(b[1]))*0.5))
    return ((a[0]+b[0])/2,(a[1]+b[1])/2)
def flatten(q,out,tol,crit,mode,depth=0):
    if crit=='dd':
        d=lambda a,b,c: max(abs(a[0]-2*b[0]+c[0]),abs(a[1]-2*b[1]+c[1]))
        v=max(d(q[0],q[1],q[2]),d(q[1],q[2],q[3]))
    elif crit=='dd2':
        d=lambda a,b,c: math.hypot(a[0]-2*b[0]+c[0],a[1]-2*b[1]+c[1])
        v=max(d(q[0],q[1],q[2]),d(q[1],q[2],q[3]))
    elif crit=='ddsum':
        d=lambda a,b,c: abs(a[0]-2*b[0]+c[0])+abs(a[1]-2*b[1]+c[1])
        v=max(d(q[0],q[1],q[2]),d(q[1],q[2],q[3]))
    if v<=tol or depth>12: return
    m=lambda a,b: mid(a,b,mode)
    a,b,c=m(q[0],q[1]),m(q[1],q[2]),m(q[2],q[3]); l,r=m(a,b),m(b,c); f=m(l,r)
    flatten([q[0],a,l,f],out,tol,crit,mode,depth+1); out.append(f); flatten([f,r,c,q[3]],out,tol,crit,mode,depth+1)
def gen(seed,n,lo,hi):
    random.seed(seed); C=[]
    for _ in range(n):
        c=(N/2+random.uniform(-.5,.5),N/2+random.uniform(-.5,.5)); r=random.uniform(lo,hi)
        a=random.uniform(0,2*math.pi)
        p0=(c[0]+r*math.cos(a),c[1]+r*math.sin(a)); p3=(c[0]-r*math.cos(a),c[1]-r*math.sin(a))
        rnd=lambda: (c[0]+random.uniform(-r,r),c[1]+random.uniform(-r,r))
        C.append([p0,rnd(),rnd(),p3,rnd(),rnd()])
    return C
def render(inps):
    out=subprocess.run(['./cov',str(N),str(N)],input=''.join(inps),capture_output=True,text=True).stdout.split('--\n')
    return [o.strip() for o in out[:len(inps)]]
def curve_inp(s): return 'm %r %r\nc %r %r %r %r %r %r\nc %r %r %r %r %r %r\nh\nf\nn\n'%tuple(s[0]+s[1]+s[2]+s[3]+s[4]+s[5]+s[0])
def poly_inp(s,tol,crit,mode):
    P=[s[0]]; flatten([s[0],s[1],s[2],s[3]],P,tol,crit,mode); P.append(s[3]); flatten([s[3],s[4],s[5],s[0]],P,tol,crit,mode)
    return 'm %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\nf\nn\n'
if __name__=='__main__':
    C=gen(1,300,2,18)
    ref=render([curve_inp(s) for s in C])
    for crit in ('dd',):
        for tol in (0.05/0.75,0.0625,0.07,0.08,0.06,0.05):
            for mode in ('d','f32'):
                got=render([poly_inp(s,tol,crit,mode) for s in C])
                print(crit,tol,mode,sum(a==b for a,b in zip(ref,got)),'/',len(C),flush=True)
