import sys,math; sys.path.insert(0,'/tmp/cgw'); import stroke2
from offs import outline
from offfit import inter
sys.setrecursionlimit(10000)
def cg_ts(q,w):
    # use off2-based find_t robust to degenerate derivatives
    from offs import bez, d1
    def off2(t,d):
        v=d1(q,t)
        if math.hypot(*v)<1e-12: v=(q[3][0]-q[1][0],q[3][1]-q[1][1]) if t>0.5 else (q[2][0]-q[0][0],q[2][1]-q[0][1])
        l=math.hypot(*v); p=bez(q,t); return (p[0]-v[1]/l*d,p[1]+v[0]/l*d)
    def ft(p,d):
        best=min(((math.dist(off2(i/2000,d),p),i/2000) for i in range(2001)))
        lo,hi=max(0,best[1]-1/2000),min(1,best[1]+1/2000)
        for _ in range(50):
            a=lo+(hi-lo)/3; b=hi-(hi-lo)/3
            if math.dist(off2(a,d),p)<math.dist(off2(b,d),p): hi=b
            else: lo=a
        return round((lo+hi)/2,4)
    n,els=outline([q],w)[0]
    types=[e[0] for e in els]
    L=[ft((float(e[-2]),float(e[-1])),w/2) for e in els[1:n]]
    k=types.index('l'); R=[]
    j=k+1
    while j<len(els) and els[j][0]=='c': j+=1
    R=sorted(ft((float(e[-2]),float(e[-1])),-w/2) for e in els[k+1:j-1])
    return L,R
def pred_ts(q,w,d):
    from offs5 import inflections
    ts=[]
    def rec(a,b,depth=0):
        if b-a<1e-9: return
        sq=stroke2._sub(q,a,b)
        ok=depth>=8 or stroke2._accept(sq,d)
        if ok: ts.append(b); return
        m=(a+b)/2; rec(a,m,depth+1); rec(m,b,depth+1)
    try: hi=any(1e-9<t<1-1e-9 for t in inflections(q))
    except Exception: hi=False
    pre=stroke2._coord_infl(q) if hi else []
    cuts=[0.0]+pre+[1.0]
    for x,y in zip(cuts,cuts[1:]): rec(x,y)
    return [round(t,4) for t in ts[:-1]]
if __name__=='__main__':
    import ast
    for line in open('/tmp/cgw/splitlist.txt'):
        w,q=ast.literal_eval(line)
        L,R=cg_ts(q,w)
        pL=pred_ts(q,w,w/2); pR=pred_ts(q,w,-w/2)
        print('ok' if (L==pL and R==pR) else 'XX','w',w,'CG L',L,'pred',pL,'| CG R',R,'pred',pR)
