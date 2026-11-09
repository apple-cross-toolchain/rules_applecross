import sys, random, pickle, math; sys.path.insert(0,'/tmp/cg'); import io, contextlib
sys.argv=['x','99']
with contextlib.redirect_stdout(io.StringIO()): import drift3 as d
random.seed(int(sys.argv[1]) if len(sys.argv)>1 and sys.argv[1]!='99' else 7)
out=[]
for i in range(20):
    y0=random.uniform(2,40); y1=random.uniform(360,398); m=random.uniform(-0.98,0.98)
    x0=200-m*(y1-y0)/2+random.uniform(-20,20); x1=x0+m*(y1-y0)
    if i%2: (x0,y0),(x1,y1)=(x1,y1),(x0,y0)    # downward edge
    P=[(x0,y0),(x1,y1),(-5.0,y1),(-5.0,y0)]
    out.append((P,d.img(P)))
pickle.dump(out,open('/tmp/cg/long20.pkl','wb'))
