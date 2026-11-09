import sys,math; sys.path.insert(0,'/tmp/cgw'); from gamma import gammas; from offs import d1, norm
from fractions import Fraction as Fr
def make(t1,t2,X=None,Y=None,a=3.0):
    p1=2*math.atan(t1); p2=2*math.atan(t2); th=p1+p2
    if X is None:
        X=10*math.cos(th/2)*1.0; Y=10*math.sin(th/2)
    b=((2*X-a)*math.sin(p1)-2*Y*math.cos(p1))/math.sin(p1-th)
    return [(0.0,0.0),(a,0.0),(X-b*math.cos(th),Y-b*math.sin(th)),(X,Y)],b
if __name__=='__main__':
    for t1,t2 in ((Fr(4,9),Fr(5,13)),(Fr(1,3),Fr(1,3)),(Fr(1,3),Fr(1,2)),(Fr(1,2),Fr(1,3)),(Fr(1,4),Fr(1,2)),(Fr(1,5),Fr(1,5)),(Fr(1,5),Fr(2,5)),(Fr(2,5),Fr(1,5)),(Fr(1,2),Fr(1,2)),(Fr(1,4),Fr(1,4)),(Fr(1,4),Fr(1,3)),(Fr(1,3),Fr(1,4))):
        for a in (2.0,4.0):
            q,b=make(float(t1),float(t2),a=a)
            if b<=0: continue
            g=gammas(q)
            if not g: print(t1,t2,'split'); continue
            print('t1',t1,'t2',t2,'a',a,'g1',Fr(-g[0]).limit_denominator(2000),'g2',Fr(-g[1]).limit_denominator(2000),'%.9f %.9f'%(-g[0],-g[1]))
