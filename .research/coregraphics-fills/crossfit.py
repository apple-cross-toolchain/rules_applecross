import sys,subprocess,random,math,os,itertools,collections
from fractions import Fraction as F
S=4096; W,H,K=32,32,240
def cases(seed=3,n=80):
    random.seed(seed); out=[]
    while len(out)<n:
        vx=random.uniform(12,19); vy=random.uniform(12,20)
        A=(vx+random.uniform(3,14),30.0)
        P=[(29.5,vy),(vx,vy),A,(29.5,30.0)]; P=[(round(x*S)/S,round(y*S)/S) for x,y in P]
        vxu,vyu,axu,ayu=[round(c*S) for c in (P[1][0],P[1][1],P[2][0],P[2][1])]
        row=vyu//S; h=(row+1)*S-vyu; m=int((axu-vxu)*S/(ayu-vyu)); xt=(vxu*S+h*m)//S
        if xt//S==vxu//S+1: out.append((P,vxu,vyu,axu,ayu,row,h,m,xt))
    return out
def obs(P,row,px):
    inp=''
    for k in range(K):
        Q=[(x+k/S,y) for x,y in P]; inp+='m %r %r\n'%Q[0]+''.join('l %r %r\n'%p for p in Q[1:])+'h\nf\nn\n'
    O=[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1][row] for o in subprocess.run(['./cov',str(W),str(H)],input=inp,capture_output=True,text=True).stdout.split('--\n')[:K]]
    return [(o[px],o[px+1]) for o in O]
