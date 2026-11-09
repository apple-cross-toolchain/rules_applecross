import sys,math,pickle; sys.path.insert(0,'/tmp/cgw'); from gamma import gammas
from gtab3 import make
from fractions import Fraction as Fr
pts=[]
vals=[Fr(1,5),Fr(1,4),Fr(1,3),Fr(2,5),Fr(1,2),Fr(3,5),Fr(2,3),Fr(1,6),Fr(3,4),Fr(1,7)]
for t1 in vals:
    for t2 in vals:
        got=None
        for a in (1.0,2.0,3.0,5.0):
            for sc in (1.0,0.6,1.5):
                q,b=make(float(t1),float(t2),a=a)
                if b<=0: continue
                g=gammas(q)
                if g: got=g; break
            if got: break
        if got:
            g1=Fr(-got[0]).limit_denominator(5000); g2=Fr(-got[1]).limit_denominator(5000)
            if abs(float(g1)+got[0])<1e-10: pts.append((t1,t2,g1))
            if abs(float(g2)+got[1])<1e-10: pts.append((t2,t1,g2))
pts=list({(a,b):(a,b,g) for a,b,g in pts}.values())
pickle.dump(pts,open('/tmp/cgw/gpts.pkl','wb'))
print(len(pts)); print([(str(a),str(b),str(g)) for a,b,g in pts[:12]])
