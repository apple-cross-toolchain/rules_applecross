import sys,subprocess,glob; sys.path.insert(0,'/tmp/cg'); import pathcmp, raster
tb=0;te=0;rows=[]; thin=0
for f in sorted(glob.glob('/tmp/accorp/w/*/b/*.pdf')):
    r=subprocess.run(['./pdfpaths',f,sys.argv[1] if len(sys.argv)>1 else '2','strokes'],capture_output=True,text=True)
    size=r.stderr.split()[1:3]; W,H=int(float(size[0])+0.999),int(float(size[1])+0.999)
    if W*H>160*160: continue
    fb=0;fe=0;n=0
    for b in [b for b in r.stdout.split('n\n') if b.strip()]:
        lines=b.strip().split('\n'); _,w,j,c=lines[-1].split(); path='\n'.join(lines[:-1])
        if float(w)<=1.0: thin+=1; continue
        n+=1
        stroke=subprocess.run(['./covs',str(W),str(H)],input='w %s\nj %s\nk %s\n%s\ns\n'%(w,j,c,path),capture_output=True,text=True).stdout.split('--\n')[0]
        obs=[list(map(int,x.split())) for x in stroke.strip().split('\n')][::-1]
        ol=subprocess.run(['./outline'],input='%s\ns %s %s %s\n'%(path,w,j,c),capture_output=True,text=True).stdout.split('--')[0]
        subs,rule=pathcmp.parse(ol+'\nf\n')
        m=raster.fill_path(subs,W,H,'nz')
        fb+=sum(1 for y in range(H) for x in range(W) if m[y][x]!=obs[y][x]); fe+=sum(1 for y in range(H) for x in range(W) if 0<obs[y][x]<255)
    if n: rows.append((f.split('/')[-1],W,n,fb,fe)); tb+=fb; te+=fe
for r in rows: print(r)
print('strokes total wrong',tb,'of',te,'edge px; thin strokes skipped',thin)
