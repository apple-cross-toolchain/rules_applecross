import subprocess,math,os
S=4096; W,H,K=32,32,240
def delta(A,dirx,Vy):
    t=(A[1]-Vy); Vx=A[0]+dirx*t
    P=[(29.5,Vy),(Vx,Vy),A,(29.5,30.0)]; P=[(round(x*S)/S,round(y*S)/S) for x,y in P]
    py=int(P[1][1]); inp=''
    for k in range(K):
        Q=[(x+k/S,y) for x,y in P]; inp+='m %r %r\n'%Q[0]+''.join('l %r %r\n'%p for p in Q[1:])+'h\nf\nn\n'
    O=[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run(['./cov',str(W),str(H)],input=inp,capture_output=True,text=True).stdout.split('--\n')[:K]]
    R=[[list(map(float,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run(['/tmp/cgprobe/target/release/cgprobe',str(W),str(H)],input=inp,capture_output=True,text=True,env={**os.environ,'CG_ACC':'1','CG_RECTQ':'1','CG_RAW':'1'}).stdout.split('--\n')[:K]]
    pts=[(O[k][py][int(P[1][0]+k/S)],R[k][py][int(P[1][0]+k/S)]*16) for k in range(K) if 0<O[k][py][int(P[1][0]+k/S)]<255]
    if not pts: return None
    return max(16*o-a for o,a in pts), min(16*(o+1)-a for o,a in pts), P
