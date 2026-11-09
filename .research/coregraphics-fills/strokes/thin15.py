import sys,math; sys.path.insert(0,'/tmp/cgw')
from thinsweep import alphas
out=[]
for k in range(0,64):
    y0=4+k/64+0.3/4096*0
    a=(0.5,y0); b=(20.5,y0+1/256/20*3)   # tiny slope -> slope_q 0
    A=alphas('m %r %r\nl %r %r'%(a+b),1.0,W=22,H=8)
    col=[(r,A[r][8]) for r in range(8) if A[r][8]>0]
    bot=col[0][0]*256+256-col[0][1]
    exact=(y0-0.5)*256
    out.append((k,round(exact,3),bot,[v for r,v in col]))
for o in out: print(o)
