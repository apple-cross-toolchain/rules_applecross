import sys,math; sys.path.insert(0,'/tmp/cgw'); from offs import outline
from fractions import Fraction
def alphas(q,w=1):
    cnt,els=outline([q],w)[0]
    if cnt!=1: return None
    m=tuple(map(float,els[0][1:])); c=list(map(float,els[1][1:]))
    h1=math.dist(q[0],q[1]); h2=math.dist(q[3],q[2])
    a1=math.dist(m,c[0:2])/h1; a2=math.dist(c[4:6],c[2:4])/h2
    # side 2
    i=[k for k,e in enumerate(els) if e[0]=='l'][0]
    s=list(map(float,els[i][1:])); c2=list(map(float,els[i+1][1:]))
    b2=math.dist(s,c2[0:2])/h2; b1=math.dist(c2[4:6],c2[2:4])/h1
    return (a1+b1)/2,(a2+b2)/2,(b1-a1)/w,(b2-a2)/w
if __name__=='__main__':
    for a in (3,4,5,6,8):
        for b in (3,5,6):
            q=[(0.0,0.0),(float(a),0.0),(10.0,10.0-b),(10.0,10.0)]
            r=alphas(q)
            if r: print('a',a,'b',b,'alpha0 %.9f %.9f'%r[:2],Fraction(r[0]).limit_denominator(200),Fraction(r[1]).limit_denominator(200),'slopes c1 %.9f c2 %.9f'%r[2:])
