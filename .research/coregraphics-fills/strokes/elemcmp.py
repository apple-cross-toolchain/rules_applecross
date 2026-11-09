import sys,math,ast; sys.path.insert(0,'/tmp/cgw'); import stroke2
from offs import outline
for line in open('/tmp/cgw/splitlist.txt'):
    w,q=ast.literal_eval(line)
    n,els=outline([q],w)[0]
    cg=[(e[0],)+tuple((float(e[i]),float(e[i+1])) for i in range(1,len(e),2)) for e in els]
    mine=stroke2.stroke([('c',)+tuple(q)],w,0,0,False)
    same=len(cg)==len(mine) and all(a[0]==b[0] and all(math.dist(p,r)<1e-3 for p,r in zip(a[1:],b[1:])) for a,b in zip(cg,mine))
    print('ok' if same else 'XX', 'w',w,[tuple(round(v,3) for v in p) for p in q],'CG',''.join(e[0] for e in cg),'mine',''.join(e[0] for e in mine))
