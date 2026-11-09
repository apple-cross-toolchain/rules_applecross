import sys,math; sys.path.insert(0,'/tmp/cgw'); from offs import *; from offs5 import find_t, inflections
def d2(q,t):
    u=1-t; return tuple(6*u*(q[2][i]-2*q[1][i]+q[0][i])+6*t*(q[3][i]-2*q[2][i]+q[1][i]) for i in (0,1))
def curv(q,t):
    a=d1(q,t); b=d2(q,t); return (a[0]*b[1]-a[1]*b[0])/max(1e-300,math.hypot(*a)**3)
def roots(f,n=4000):
    out=[]; prev=f(0)
    for i in range(1,n+1):
        t=i/n; v=f(t)
        if prev==0 or prev*v<0:
            lo,hi=(i-1)/n,t
            for _ in range(60):
                m=(lo+hi)/2
                if f(lo)*f(m)<=0: hi=m
                else: lo=m
            out.append((lo+hi)/2)
        prev=v
    return out
def special(q,d):
    r={}
    r['infl']=[t for t in inflections(q) if 0<t<1]
    r['xext']=roots(lambda t:d1(q,t)[0]); r['yext']=roots(lambda t:d1(q,t)[1])
    r['cusp']=roots(lambda t:1+d*curv(q,t))+roots(lambda t:1-d*curv(q,t))
    h=1e-6
    r['kext']=roots(lambda t:abs(curv(q,min(1,t+h)))-abs(curv(q,max(0,t-h))))
    return r
if __name__=='__main__':
    for q,w in (([(5.692, 8.023), (0.631, 1.179), (7.61, 4.722), (3.796, 2.1)],4),([(3.041, 16.66), (9.691, 9.342), (0.908, 10.206), (14.895, 8.452)],3)):
        n,els=outline([q],w)[0]
        t1=[find_t(q,w/2,(float(e[-2]),float(e[-1])))[0] for e in els[1:1+n-1]]
        print('side1',['%.5f'%t for t in t1])
        for k,v in special(q,w/2).items(): print('  ',k,['%.5f'%t for t in v])
