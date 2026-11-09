import subprocess,os,sys
S=4096
def xrows(x0u,y0,x1u,y1,W=24,H=64,K=32,side='open'):
    # vertices in units (x) ; y in px ; returns per-row x (units) of edge, from flips in alpha under x translation
    inp=''
    for k in range(K):
        a=(x0u+k)/S; b=(x1u+k)/S
        if side=='open': P=[(a,y0),(W-0.5,y0),(W-0.5,y1),(b,y1)]
        elif side=='close': P=[(a,y0),(b,y1),(0.5,y1),(0.5,y0)]
        elif side=='close_rev': P=[(b,y1),(a,y0),(0.5,y0),(0.5,y1)]
        else: P=[(b,y1),(a,y0),(W-0.5,y0),(W-0.5,y1)]
        inp+='m %r %r\n'%P[0]+''.join('l %r %r\n'%p for p in P[1:])+'h\nf\nn\n'
    O=[[list(map(int,r.split())) for r in o.strip().split('\n')][::-1] for o in subprocess.run(['./cov',str(W),str(H)],input=inp,capture_output=True,text=True).stdout.split('--\n')[:K]]
    px=x0u//S; res={}
    for r in range(int(y0)+1,int(y1)):
        seq=[O[k][r][px] for k in range(K)]
        # open: alpha = floor(16*(W_px - x)/... ) decreasing with k; find k where it drops
        for k in range(1,K):
            if seq[k]!=seq[k-1]:
                # alpha changes when (Wpx*S - (x+k)) crosses a multiple of 16 (area units, h=1 row => units)
                v=seq[k-1]
                # area units before drop at k-1: >= 16*v ; at k: < 16*v  => (px+1)*S - x - k < 16*v <= (px+1)*S - x - (k-1)
                x=(px+1)*S - 16*v - k + 1 if side in ('open','open_rev') else px*S + 16*seq[k] - k + 16
                res[r]=x; break
    return res
