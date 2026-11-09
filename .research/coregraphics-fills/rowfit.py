import sys, math, io, contextlib, itertools; sys.argv=['x','99']; sys.path.insert(0,'/tmp/cg')
with contextlib.redirect_stdout(io.StringIO()): import drift3 as d
from model import clip
import poly
S=4096
def r(v): return math.floor(v*S+0.5)/S
def tq(v): return math.trunc(v*S)/S
def fl(v): return math.floor(v*S)/S
def run(x0,y0,x1,y1, show=False):
    im=d.img([(x0,y0),(x1,y1),(-5.0,y1),(-5.0,y0)])
    m=(x1-x0)/(y1-y0); xm=lambda yy: x0+m*(yy-y0)
    k0=math.ceil(y0)
    X0=r(xm(k0))
    stepped={k:X0+(k-k0)*tq(m) for k in range(k0,math.floor(y1)+1)}
    exact={k:r(xm(k)) for k in stepped}
    res={}
    for name,xs in (('stepped',stepped),('exact',exact)):
        for ycmode in ('exact','line','stepped_col'):
            bad=[0,0]; n=[0,0]
            for y in range(k0,math.floor(y1)):
                xb,xt=xs[y],xs[y+1]
                pts=[(xb,y)]
                lo,hi=sorted((xb,xt))
                for kc in range(math.floor(lo)+1,math.ceil(hi)):
                    if ycmode=='exact': yc=r(y0+(kc-x0)/m)
                    elif ycmode=='line': yc=y+(kc-xb)/(xt-xb)
                    else:
                        c0=math.ceil(x0) if m>0 else math.floor(x0)
                        Y0=r(y0+(c0-x0)/m); yc=Y0+(kc-c0)*tq(1/m)
                    pts.append((kc,yc))
                pts.append((xt,y+1))
                poly_=pts+[(-5,y+1),(-5,y)]
                part=[x for x in range(math.floor(lo),math.floor(hi)+1)]
                two=1 if len(part)>1 else 0
                for x in part:
                    a=max(0,min(255,math.floor(abs(poly.sarea(clip(poly_,x,y,x+1,y+1)))*256+1e-12)))
                    if a in (0,255) and im[y][x] in (0,255): continue
                    n[two]+=1; bad[two]+=a!=im[y][x]
            res[(name,ycmode)]='single %d/%d two %d/%d'%(bad[0],n[0],bad[1],n[1])
    return res
if __name__=='__main__':
    for args in ((98.4,5.0,302.1,395.0),(250.3,5.2,20.7,394.6),(150.1,3.3,190.9,396.2)):
        print(args); [print('  ',k,v) for k,v in run(*args).items()]
