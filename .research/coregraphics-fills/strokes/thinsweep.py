import subprocess,os,sys
os.chdir('/tmp/cg')
def alphas(path,w,W=8,H=8):
    out=subprocess.run(['./thin2',str(W),str(H)],input=path+'\ns %r\n'%w,capture_output=True,text=True).stdout.split('\n')[0].split()
    a=list(map(int,out)); return [a[r*W:(r+1)*W] for r in range(H)][::-1]   # row index = y
if __name__=='__main__':
    for w in (1.0,0.5,0.25,0.75):
        print('w',w)
        for k in range(0,65,4):
            y=3+k/64
            A=alphas('m 0.5 %r\nl 7.5 %r'%(y,y),w)
            print('  y=%.4f'%y,'rows2-4 at x=3:',[A[r][3] for r in (2,3,4)],'band [%.4f,%.4f]'%(y-w/2,y+w/2))
