import sys,math,pickle; sys.path.insert(0,'/tmp/cgw'); from offs import bez,off; from offmodel import piece
from bnd2 import nearest, fam
B=pickle.load(open('/tmp/cgw/bnd.pkl','rb'))
sets={'odd5':(0.1,0.3,0.5,0.7,0.9),'all9':tuple(k/10 for k in range(1,10)),'3':(0.3,0.5,0.7),'q':(0.25,0.5,0.75),'odd5+q':(0.1,0.25,0.3,0.5,0.7,0.75,0.9)}
for name,ts in sets.items():
    vals=[]
    for s,u,v,w in B:
        ep=max(nearest(lambda x:bez(piece(fam(s,u,v),w/2),x),off(fam(s,u,v),t,w/2)) for t in ts)
        em=max(nearest(lambda x:bez(piece(fam(s,u,v),-w/2),x),off(fam(s,u,v),t,-w/2)) for t in ts)
        vals.append((ep,em))
    binding=[max(a,b) for a,b in vals]
    print(name,'binding min %.6f max %.6f'%(min(binding),max(binding)))
