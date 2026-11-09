import sys,math; sys.path.insert(0,'/tmp/cgw')
from thinsweep import alphas
def bot_at(a,b,k=6):
    A=alphas('m %r %r\nl %r %r'%(a+b),1.0,W=16,H=12)
    col=[(r,A[r][k]) for r in range(12) if A[r][k]>0]
    return col[0][0]*256+256-col[0][1], [v for r,v in col]
dx=12.0; dyu=37.0/256   # fixed slope
print('-- vary y0 by 1/4096 (x0=0.3)')
prev=None
for j in range(0,40):
    y0=5+j/4096; a=(0.3,y0); b=(0.3+dx,y0+dyu)
    bt,v=bot_at(a,b)
    if bt!=prev: print(j,bt,v); prev=bt
print('-- vary x0 by 1/4096 (y0=5)')
prev=None
for j in range(0,400,1):
    x0=0.3+j/4096; a=(x0,5.0); b=(x0+dx,5.0+dyu)
    bt,v=bot_at(a,b)
    if bt!=prev: print(j,bt,v); prev=bt
