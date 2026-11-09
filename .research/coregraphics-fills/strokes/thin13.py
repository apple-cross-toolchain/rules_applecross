import sys,math; sys.path.insert(0,'/tmp/cgw')
from thinsweep import alphas
W=210
for dyu in (4,7,12,-12,25,40,64,100,128,200):
    for x0u in (3,128):
        x0=x0u/256; y0=4+97/256; a=(x0,y0); b=(200+x0,y0+dyu/256)
        A=alphas('m %r %r\nl %r %r'%(a+b),1.0,W=W,H=8)
        def low(x):
            col=[(r,A[r][x]) for r in range(8) if A[r][x]>0]
            return col[0][0]*256+256-col[0][1] if col else None   # top of lowest row coverage -> band bottom
        b1,b2=low(2),low(197)
        print('dy %4d x0 %3d'%(dyu,x0u),'bot col2 %d col197 %d  delta %d  exact delta %.3f'%(b1,b2,b2-b1,dyu*195/200))
