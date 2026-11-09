import sys, math, pickle; sys.path.insert(0,'/tmp/cg'); import anchor3, fastfit as ff
def seg_int(p,q,X):
    # integral of min(x,X) dy along segment p->q
    (x0,y0),(x1,y1)=p,q
    if y0==y1: return 0.0
    if (x0<=X and x1<=X): return (x0+x1)/2*(y1-y0)
    if (x0>=X and x1>=X): return X*(y1-y0)
    t=(X-x0)/(x1-x0); ym=y0+t*(y1-y0)
    a=seg_int((x0,y0),(X,ym),X); b=seg_int((X,ym),(x1,y1),X)
    return a+b
def F(pl,X): return sum(seg_int(pl[i],pl[i+1],X) for i in range(len(pl)-1))
def score(p0,p1,im,W,cfgr=('trunc',4096),cfgc=('trunc',4096),show=False):
    xs=ff.stepper(p0[1],p0[0],p1[1],p1[0],ff.pick('bottom',p0,p1,1),cfgr,None)
    ys=ff.stepper(p0[0],p0[1],p1[0],p1[1],ff.pick('bottom',p0,p1,0),cfgc,None)
    bad=0;n=0;miss=[]
    for y in range(math.ceil(min(p0[1],p1[1])),math.floor(max(p0[1],p1[1]))):
        if y not in xs or y+1 not in xs: continue
        xb,xt=xs[y],xs[y+1]
        cols=[(k,ys[k]) for k in range(math.floor(min(xb,xt))+1,math.ceil(max(xb,xt))) if k in ys]
        pl=[(xb,y)]+sorted(cols,key=lambda p:p[0],reverse=xt<xb)+[(xt,y+1)]
        for x in range(max(0,math.floor(min(xb,xt))-1),min(W,math.floor(max(xb,xt))+2)):
            a=(F(pl,x+1)-F(pl,x))
            if p1[1]<p0[1]: pass
            v=max(0,min(255,math.floor(a*256+1e-12)))
            if v in (0,255) and im[y][x]==v: continue
            n+=1
            if v!=im[y][x]: bad+=1; miss.append((y,x,round(a*256,3),im[y][x]))
    return bad,n,miss
if __name__=='__main__':
    data=pickle.load(open('/tmp/cg/long20.pkl','rb'))
    tot=0;N=0;per=[]
    for P,im in data:
        b,n,m=score(P[0],P[1],im,400); tot+=b;N+=n;per.append(b)
    print(tot,N,per)
