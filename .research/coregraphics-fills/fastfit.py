import sys, math, itertools, pickle; sys.path.insert(0,'/tmp/cg')
from model import clip
import poly
S=4096
Q={'round':lambda v: math.floor(v*S+0.5)/S,'trunc':lambda v: math.trunc(v*S)/S,'floor':lambda v: math.floor(v*S)/S}
def stepper(a0,b0,a1,b1,start,sq,s0):
    # b at integer a strictly between the endpoints, stepping from endpoint index start (0 or 1)
    if a0==a1: return {}
    (pa,pb),(qa,qb)=((a0,b0),(a1,b1)) if start==0 else ((a1,b1),(a0,b0))
    dirn=1 if qa>pa else -1
    m=Q[sq]((qb-pb)/abs(qa-pa))
    k0=math.floor(pa)+1 if dirn>0 else math.ceil(pa)-1
    B0=Q[s0](pb+(qb-pb)*abs(k0-pa)/abs(qa-pa))
    out={}; k=k0; i=0
    while (k<qa if dirn>0 else k>qa): out[k]=B0+i*m; k+=dirn; i+=1
    return out
def pick(f,p0,p1,axis):
    if f=='p0': return 0
    if f=='p1': return 1
    if f=='bottom': return 0 if p0[1]<p1[1] else 1
    if f=='top': return 0 if p0[1]>p1[1] else 1
    if f=='left': return 0 if p0[0]<p1[0] else 1
    if f=='right': return 0 if p0[0]>p1[0] else 1
def score(p0,p1,im,cfg,W):
    rfm,rsq,rs0,cfm,csq,cs0=cfg
    (x0,y0),(x1,y1)=p0,p1
    xs=stepper(y0,x0,y1,x1,pick(rfm,p0,p1,1),rsq,rs0)
    ys=stepper(x0,y0,x1,y1,pick(cfm,p0,p1,0),csq,cs0)
    lo,hi=sorted((y0,y1)); bad=0;n=0
    for y in range(math.ceil(lo),math.floor(hi)):
        if y not in xs or y+1 not in xs: continue
        xb,xt=xs[y],xs[y+1]
        cols=[(k,ys[k]) for k in range(math.floor(min(xb,xt))+1,math.ceil(max(xb,xt))) if k in ys]
        cols.sort(key=lambda p:p[0], reverse=xt<xb)
        P=[(xb,y)]+cols+[(xt,y+1),(-5,y+1),(-5,y)]
        for x in range(max(0,math.floor(min(xb,xt))-1),min(W,math.floor(max(xb,xt))+2)):
            a=max(0,min(255,math.floor(abs(poly.sarea(clip(P,x,y,x+1,y+1)))*256+1e-12)))
            if a in (0,255) and im[y][x]==a: continue
            n+=1; bad+=a!=im[y][x]
    return bad,n
if __name__=='__main__':
    data=pickle.load(open('/tmp/cg/long5.pkl','rb'))
    res=[]
    for rfm,cfm in itertools.product(('bottom','top','p0','p1'),('bottom','top','p0','p1','left','right')):
      for rsq,csq in (('trunc','trunc'),('floor','floor'),('trunc','floor'),('floor','trunc')):
        for s0 in ('round','trunc','floor'):
            cfg=(rfm,rsq,s0,cfm,csq,s0)
            sc=[score(P[0],P[1],im,cfg,400) for P,im in data]
            res.append((sum(b for b,n in sc),[b for b,n in sc],cfg))
    for r in sorted(res)[:8]: print(r)
