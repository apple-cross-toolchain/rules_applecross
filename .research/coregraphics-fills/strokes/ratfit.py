import pickle,itertools
from fractions import Fraction as Fr
pts=pickle.load(open('/tmp/cgw/gpts.pkl','rb'))
def monos(deg):
    return [(i,j) for i in range(deg+1) for j in range(deg+1-i)]
def nullspace(M):
    M=[row[:] for row in M]; rows=len(M); cols=len(M[0]); piv=[]; r=0
    for c in range(cols):
        p=next((k for k in range(r,rows) if M[k][c]!=0),None)
        if p is None: continue
        M[r],M[p]=M[p],M[r]; inv=1/M[r][c]; M[r]=[v*inv for v in M[r]]
        for k in range(rows):
            if k!=r and M[k][c]!=0:
                f=M[k][c]; M[k]=[a-f*b for a,b in zip(M[k],M[r])]
        piv.append(c); r+=1
        if r==rows: break
    free=[c for c in range(cols) if c not in piv]
    sols=[]
    for f in free:
        v=[Fr(0)]*cols; v[f]=Fr(1)
        for i,c in enumerate(piv): v[c]=-M[i][f]
        sols.append(v)
    return sols
for dn,dd in ((2,2),(3,2),(3,3),(4,2),(4,3),(4,4),(5,4)):
    mn=monos(dn); md=monos(dd)
    M=[]
    for t1,t2,g in pts:
        M.append([t1**i*t2**j for i,j in mn]+[-g*t1**i*t2**j for i,j in md])
    ns=nullspace(M)
    print('deg',dn,dd,'nullspace dim',len(ns))
    if ns:
        v=ns[0]
        N={m:c for m,c in zip(mn,v[:len(mn)]) if c!=0}; D={m:c for m,c in zip(md,v[len(mn):]) if c!=0}
        print(' N',{f't1^{i}t2^{j}':str(c) for (i,j),c in N.items()}); print(' D',{f't1^{i}t2^{j}':str(c) for (i,j),c in D.items()})
        break
