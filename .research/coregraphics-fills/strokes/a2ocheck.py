import sys,math,pickle,collections; sys.path.insert(0,'/tmp/cgw'); from offs import bez,off; from offmodel import piece
from bnd2 import nearest, fam
B=pickle.load(open('/tmp/cgw/bnd.pkl','rb'))
mx=[]
for s,u,v,w in B:
    q=fam(s,u,v)
    for d in (w/2,-w/2):
        c=piece(q,d)
        mx.append(max(nearest(lambda x:off(q,x,d),bez(c,t)) for t in (0.1,0.3,0.5,0.7,0.9)))
print('boundaries: max a2o5 over sides %.5f'%max(mx))
data=pickle.load(open('/tmp/cgw/accdata.pkl','rb')); side=pickle.load(open('/tmp/cgw/accside.pkl','rb'))
st=collections.Counter()
for i,L,R in side:
    q,w,_=data[i]
    for d,cnt in ((w/2,L),(-w/2,R)):
        c=piece(q,d)
        e1=max(nearest(lambda x:bez(c,x),off(q,t,d)) for t in (0.1,0.3,0.5,0.7,0.9))
        e2=max(nearest(lambda x:off(q,x,d),bez(c,t)) for t in (0.1,0.3,0.5,0.7,0.9))
        st[('o2a',(e1<=0.1)==(cnt==1))]+=1; st[('both',(max(e1,e2)<=0.1)==(cnt==1))]+=1
print(sorted(st.items()))
