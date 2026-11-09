import sys, math; sys.path.insert(0,'/tmp/cg'); import fastfit as ff
S=4096
def q(v,mode,qq):
    return {'round':math.floor(v*qq+0.5),'trunc':math.trunc(v*qq),'floor':math.floor(v*qq)}[mode]/qq
def stepper(a0,b0,a1,b1,start,sq,s0):
    if a0==a1: return {}
    sqm,sqq=sq
    (pa,pb),(qa,qb)=((a0,b0),(a1,b1)) if start==0 else ((a1,b1),(a0,b0))
    dirn=1 if qa>pa else -1
    m=q((qb-pb)/abs(qa-pa),sqm,sqq)
    k0=math.floor(pa)+1 if dirn>0 else math.ceil(pa)-1
    frac=abs(k0-q(pa,'round',S))
    B0=q(pb,'round',S)+q(frac*m,'floor',S)
    out={}; k=k0; i=0
    while (k<qa if dirn>0 else k>qa): out[k]=B0+i*m; k+=dirn; i+=1
    return out
ff.stepper=stepper
