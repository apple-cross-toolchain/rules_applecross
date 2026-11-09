import sys, random, pickle, math, io, contextlib; sys.path.insert(0,'/tmp/cg')
sys.argv=['x','99']
with contextlib.redirect_stdout(io.StringIO()): import drift3 as d
random.seed(11); out=[]
for i in range(16):
    m=random.choice((-1,1))*random.uniform(1.05,6)   # dx/dy
    dy=380/abs(m); y0=random.uniform(10,390-dy); x0=10 if m>0 else 390; x1=x0+m*dy; y1=y0+dy
    x0+=random.uniform(0,1); x1+=random.uniform(-1,0) if m>0 else random.uniform(0,1)
    if i%2: (x0,y0),(x1,y1)=(x1,y1),(x0,y0)
    P=[(x0,y0),(x1,y1),(-5.0,y1),(-5.0,y0)]
    out.append((P,d.img(P)))
pickle.dump(out,open('/tmp/cg/shallow16.pkl','wb'))
