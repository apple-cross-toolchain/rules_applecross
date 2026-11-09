import sys,os,subprocess,random,math; os.chdir('/tmp/cg')
def outline(paths,w):
    inp=''.join('m %r %r\nc %r %r %r %r %r %r\ns %r 0 0\n'%(tuple(q[0]+q[1]+q[2]+q[3])+(w,)) for q in paths)
    out=subprocess.run(['./outline'],input=inp,capture_output=True,text=True).stdout.split('--\n')
    res=[]
    for o in out[:len(paths)]:
        els=[l.split() for l in o.strip().split('\n')]
        n=0
        for e in els[1:]:
            if e[0]=='c': n+=1
            else: break
        res.append((n,els))
    return res
def bez(q,t):
    u=1-t; return tuple(u**3*q[0][i]+3*u*u*t*q[1][i]+3*u*t*t*q[2][i]+t**3*q[3][i] for i in (0,1))
def d1(q,t):
    u=1-t; return tuple(3*u*u*(q[1][i]-q[0][i])+6*u*t*(q[2][i]-q[1][i])+3*t*t*(q[3][i]-q[2][i]) for i in (0,1))
def norm(v): l=math.hypot(*v); return (v[0]/l,v[1]/l)
def off(q,t,d):
    p=bez(q,t); n=norm(d1(q,t)); return (p[0]-n[1]*d,p[1]+n[0]*d)
def approx(q,d):
    p0=off(q,0,d); p3=off(q,1,d)
    h1=(q[1][0]-q[0][0],q[1][1]-q[0][1]); h2=(q[2][0]-q[3][0],q[2][1]-q[3][1])
    m=off(q,.5,d)
    # midpoint = (p0+3(p0+a h1)+3(p3+a h2)+p3)/8 -> solve a by least squares over x,y
    bx=(m[0]*8-4*p0[0]-4*p3[0]); by=(m[1]*8-4*p0[1]-4*p3[1])
    cx=3*(h1[0]+h2[0]); cy=3*(h1[1]+h2[1])
    a=(bx*cx+by*cy)/(cx*cx+cy*cy)
    return [p0,(p0[0]+a*h1[0],p0[1]+a*h1[1]),(p3[0]+a*h2[0],p3[1]+a*h2[1]),p3],a
if __name__=='__main__':
    q=[(0,0),(10,0),(20,10),(20,20)]
    n,els=outline([q],2)[0]; print(n,els[:3]); print(approx(q,1))
