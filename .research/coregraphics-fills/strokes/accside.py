import sys,math,pickle,collections; sys.path.insert(0,'/tmp/cgw'); from offs import outline,bez,off; from offmodel import piece
from bnd2 import nearest
data=pickle.load(open('/tmp/cgw/accdata.pkl','rb'))
qs=[q for q,w,a in data]; ws=[w for q,w,a in data]
side=[]
for w in set(ws):
    idx=[i for i in range(len(qs)) if ws[i]==w]
    res=outline([qs[i] for i in idx],w)
    for i,(n,els) in zip(idx,res):
        types=[e[0] for e in els]
        L=n
        # right side curves: after first 'l' up to next 'l'
        k=types.index('l'); R=0
        for t in types[k+1:]:
            if t=='c': R+=1
            else: break
        side.append((i,L,R))
side.sort()
st=collections.Counter(); bad=[]
for i,L,R in side:
    q,w=qs[i],ws[i]
    for nm,d,cnt in (('L',w/2,L),('R',-w/2,R)):
        c=piece(q,d); e=max(nearest(lambda x:bez(c,x),off(q,t,d)) for t in (0.1,0.3,0.5,0.7,0.9))
        ok=(e<=0.1)==(cnt==1); st[ok]+=1
        if not ok and len(bad)<12: bad.append((nm,cnt,round(e,4),w,[tuple(round(x,3) for x in p) for p in q]))
print(st); [print(b) for b in bad]
pickle.dump(side,open('/tmp/cgw/accside.pkl','wb'))
