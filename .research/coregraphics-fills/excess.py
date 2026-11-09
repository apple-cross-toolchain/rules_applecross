import sys,random,math; sys.path.insert(0,'/tmp/cg'); import cfit
S=4096
def fd(a,b): return a//b
def tdiv(a,b): return int(a/b)
def measure(seed,n=30):
    random.seed(seed); rows=[]
    while len(rows)<n:
        vx=random.uniform(10.1,13.9); vy=random.uniform(10,20)
        if not (0.1<vy%1<0.9) or not (0.15<vx%1<0.85): continue
        A=(vx+random.uniform(-1.5,1.5),2.3); B=(vx+random.uniform(-1.5,1.5),29.6)
        P=[A,(30.5,2.1),(30.9,29.7),B,(vx,vy)]
        lo,hi,pts=cfit.sweep(P,int(vx),int(vy),K=160)
        if len(pts)<100: continue
        q=lambda v:int(round(v*S))
        ax,ay,Vx,Vy,bx,by=q(A[0]),q(A[1]),q(vx),q(vy),q(B[0]),q(B[1])
        m_in=tdiv((Vx-ax)*S,(Vy-ay)); m_out=tdiv((bx-Vx)*S,(by-Vy))
        row=Vy//S
        x_lo=fd(ax*S+(row*S-ay)*m_in,S); E=fd(ax*S+(Vy-ay)*m_in,S)
        x_hi=Vx+fd(((row+1)*S-Vy)*m_out,S)
        ex_lo=ax+(row*S-ay)*(Vx-ax)/(Vy-ay)
        rows.append(dict(pts=pts,P=[(cfit.q(x),cfit.q(y)) for x,y in P],c=(lo+hi)/2,lo=lo,hi=hi,hb=(Vy-row*S)/S,ha=((row+1)*S-Vy)/S,dE=E-Vx,dlo=x_lo-ex_lo,mi=m_in,mo=m_out,V=(vx,vy)))
    return rows
if __name__=='__main__':
    R=measure(1)
    for r in R: print('c %.4f [%.4f,%.4f) excess units %.3f  hb %.3f ha %.3f  E-V %d  drift@row %.2f  m_in %d m_out %d'%(r['c'],r['lo'],r['hi'],r['c']*16,r['hb'],r['ha'],r['dE'],r['dlo'],r['mi'],r['mo']))
