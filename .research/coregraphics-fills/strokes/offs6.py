import sys,math,random,pickle; sys.path.insert(0,'/tmp/cgw'); from offs import *; from offs5 import find_t
def sub(q,a,b):
    # de Casteljau sub-curve on [a,b]
    def split(q,t):
        l=lambda p,r:((1-t)*p[0]+t*r[0],(1-t)*p[1]+t*r[1])
        a1,b1,c1=l(q[0],q[1]),l(q[1],q[2]),l(q[2],q[3]); d1,e1=l(a1,b1),l(b1,c1); f=l(d1,e1)
        return [q[0],a1,d1,f],[f,e1,c1,q[3]]
    r=split(q,b)[0] if b<1 else q
    return split(r,a/b)[1] if a>0 else r
def side_ts(q,w):
    n,els=outline([q],w)[0]
    d=w/2
    cs=[i for i,e in enumerate(els) if e[0]=='c']
    s1=els[1:1+n]
    # side 2: after the cap line
    rest=els[1+n:]
    j=[i for i,e in enumerate(rest) if e[0]=='c']
    s2=[rest[i] for i in j]
    t1=[find_t(q,d,(float(e[-2]),float(e[-1]))) for e in s1[:-1]]
    t2=[find_t(q,-d,(float(e[-2]),float(e[-1]))) for e in s2[:-1]]
    return [t for t,err in t1 if err<1e-6], [t for t,err in t2 if err<1e-6], len(t1)+len(t2)-sum(1 for _,e in t1+t2 if e<1e-6)
def nodes_from(ts):
    S=set(round(t*1024) for t in ts)
    out=[]
    def rec(a,b,depth):
        m=(a+b)//2
        sp=m in S and b-a>1
        out.append((a/1024,b/1024,sp))
        if sp: rec(a,m,depth+1); rec(m,b,depth+1)
    rec(0,1024,0)
    return out,S
if __name__=='__main__':
    random.seed(int(sys.argv[1])); data=[]
    bad=0
    for it in range(int(sys.argv[2])):
        s=random.choice([5,10,20,40])
        q=[(random.uniform(0,s),random.uniform(0,s)) for _ in range(4)]
        w=random.choice([1.5,2,3,4,6,8])
        try: t1,t2,miss=side_ts(q,w)
        except Exception: continue
        if miss: bad+=1; continue
        for ts,d in ((t1,w/2),(t2,-w/2)):
            nd,S=nodes_from(ts)
            # check all ts explained by dyadic tree
            if len(S)!=sum(1 for a,b,sp in nd if sp): bad+=1; continue
            for a,b,sp in nd: data.append((q,d,a,b,sp))
    pickle.dump(data,open('/tmp/cgw/offs6_%s.pkl'%sys.argv[1],'wb'))
    print('nodes',len(data),'split',sum(1 for x in data if x[4]),'bad curves',bad)
