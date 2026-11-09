import sys,math,pickle,collections; sys.path.insert(0,'/tmp/cgw'); from offs import bez,off; from offmodel import piece
from bnd2 import nearest
data=pickle.load(open('/tmp/cgw/accdata.pkl','rb')); side=pickle.load(open('/tmp/cgw/accside.pkl','rb'))
rows=[]
for i,L,R in side:
    q,w,_=data[i]
    for d,cnt in ((w/2,L),(-w/2,R)):
        c=piece(q,d)
        e1=max(nearest(lambda x:bez(c,x),off(q,t,d)) for t in (0.1,0.3,0.5,0.7,0.9))
        e2=max(nearest(lambda x:off(q,min(max(x,1e-9),1-1e-9),d),bez(c,t)) for t in (0.1,0.3,0.5,0.7,0.9))
        rows.append((cnt==1,e1,e2))
pickle.dump(rows,open('/tmp/cgw/a2orows.pkl','wb'))
for tol in (0.1,0.12,0.15,0.2,0.25,0.3,0.5,1.0):
    print(tol,sum(1 for acc,e1,e2 in rows if (e1<=0.1 and e2<=tol)==acc),'/',len(rows))
