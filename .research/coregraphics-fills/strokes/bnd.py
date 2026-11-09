import sys,math,pickle; sys.path.insert(0,'/tmp/cgw'); from offs import outline
def accepted(q,w):
    n,els=outline([q],w)[0]; return n==1 and sum(1 for e in els if e[0]=='c')==2
def fam(s,u,v): return [(0.0,0.0),(s*u,0.0),(float(s),s-s*v),(float(s),float(s))]
B=[]
for s,u,w in ((10,0.5,1.01),(10,0.4,1.01),(10,0.6,2.0),(40,0.5,1.01),(40,0.6,1.01),(40,0.7,3.0),(20,0.5,1.5),(20,0.45,1.5),(80,0.6,1.01),(80,0.5,2.0)):
    # scan v to find transitions
    vs=[0.05+0.01*i for i in range(100)]
    a=[accepted(fam(s,u,v),w) for v in vs]
    for i in range(len(vs)-1):
        if a[i]!=a[i+1]:
            lo,hi=vs[i],vs[i+1]; flo=a[i]
            for _ in range(45):
                m=(lo+hi)/2
                if accepted(fam(s,u,m),w)==flo: lo=m
                else: hi=m
            B.append((s,u,(lo+hi)/2,w))
            print('s',s,'u',u,'w',w,'boundary v %.9f'%((lo+hi)/2),'accepted side','low' if flo else 'high')
pickle.dump(B,open('/tmp/cgw/bnd.pkl','wb'))
