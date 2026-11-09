import sys,subprocess,math,glob; sys.path.insert(0,'/tmp/cg'); import pathcmp, raster
tot_bad=0; tot_edge=0; rows=[]
for f in sorted(glob.glob('/tmp/accorp/w/*/b/*.pdf')):
    r=subprocess.run(['./pdfpaths',f,sys.argv[1] if len(sys.argv)>1 else '2'],capture_output=True,text=True)
    size=r.stderr.split()[1:3]; W,H=int(float(size[0])+0.999),int(float(size[1])+0.999)
    if W*H>160*160: continue
    blocks=[b for b in r.stdout.split('n\n') if b.strip()]
    fb=0; fe=0
    for b in blocks:
        bad,edge=pathcmp.compare(b,W,H); fb+=len(bad); fe+=edge
    rows.append((f.split('/')[-1],W,len(blocks),fb,fe)); tot_bad+=fb; tot_edge+=fe
for r in rows: print(r)
print('total wrong',tot_bad,'of',tot_edge,'edge pixels')
