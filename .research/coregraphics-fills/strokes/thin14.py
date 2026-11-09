import sys,math; sys.path.insert(0,'/tmp/cgw')
from thinsweep import alphas
res=[]
for dyu in list(range(-40,41,2))+[-13,-11,-9,-7,-5,-3,-1,1,3,5,7,9,11,13]:
    x0=3/256; y0=4+97/256; a=(x0,y0); b=(200+x0,y0+dyu/256)
    A=alphas('m %r %r\nl %r %r'%(a+b),1.0,W=210,H=8)
    def bot(x):
        col=[(r,A[r][x]) for r in range(8) if A[r][x]>0]
        return col[0][0]*256+256-col[0][1]
    d=bot(194)-bot(2)
    m4096=dyu*16/200
    res.append((dyu,m4096,d*16/192))
for r in sorted(res): print('dy %4d  m*4096 %+.3f  measured %+.3f'%r)
