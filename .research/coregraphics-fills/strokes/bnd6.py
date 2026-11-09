import sys,math,pickle; sys.path.insert(0,'/tmp/cgw'); from offs import bez,off; from offmodel import piece
from bnd2 import nearest, fam
B=pickle.load(open('/tmp/cgw/bnd.pkl','rb'))
for ts in ((0.3,0.5,0.7),(0.1,0.3,0.5,0.7,0.9),(0.2,0.3,0.5,0.7,0.8)):
    for sides in (('p',),('m',),('p','m')):
        mx=[]
        for s,u,v,w in B:
            q=fam(s,u,v); e=0
            for side in sides:
                d=w/2 if side=='p' else -w/2
                c=piece(q,d)
                e=max(e,max(nearest(lambda x:bez(c,x),off(q,t,d)) for t in ts))
            mx.append(e)
        print(ts,sides,'min %.6f max %.6f'%(min(mx),max(mx)))
