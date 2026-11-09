import sys,math,collections; sys.path.insert(0,'/tmp/cg'); import band1, intfit
S=4096
D=band1.load(2,30)+intfit.load(1,30)
def fl(a,b): return a//b
def ce(a,b): return -((-a)//b)
def rd(a,b): return (2*a+b)//(2*b)
F={}
for nm,q in (('fl',fl),('ce',ce),('rd',rd)):
    F['trap_'+nm]=lambda h,da,db,q=q: q(h*(da+db),2*S)
    F['ends_'+nm]=lambda h,da,db,q=q: q(q(h*da,S)+q(h*db,S),2)
    F['ends_'+nm+'_sum2']=lambda h,da,db,q=q: (q(h*da,S)+q(h*db,S))/2
    F['h16_'+nm]=lambda h,da,db,q=q: q((h>>4)*(da+db),2*S>>4)
    F['dx16_'+nm]=lambda h,da,db,q=q: q(h*((da+db)>>4),(2*S)>>4)
    F['mid_'+nm]=lambda h,da,db,q=q: q(h*((da+db)>>1),S)
    F['mid_ce_'+nm]=lambda h,da,db,q=q: q(h*(-((-(da+db))>>1)),S)
def test(f):
    full=0; badk=0; totk=0
    for d in D:
        W=d['W']; ok=True
        for k,o,_ in d['pts']:
            T=sum(f(yb-ya,W-xa-k,W-xb-k) for xa,ya,xb,yb in d['pcs'])
            r= min(255,math.floor(T/16))==o
            totk+=1; badk+= not r; ok&=r
        full+=ok
    return full,badk/totk
for k,f in F.items():
    full,rate=test(f); print('%-16s full %2d/60  k-miss %.4f'%(k,full,rate))
