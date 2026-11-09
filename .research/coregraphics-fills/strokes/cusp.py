import sys,os,subprocess,math,itertools; sys.path.insert(0,'/tmp/cgw'); os.chdir('/tmp/cg')
from rsstroke import parse, same
qs=[[(50.344,45.891),(50.277,45.93),(50.914,45.496),(50.766,44.25)],[(27.234, 44.25), (27.086, 45.496), (27.723, 45.93), (27.656, 45.891)]]
inp=''.join('m %r %r\nc %r %r %r %r %r %r\ns 1.5 0 0\n'%tuple(v for p in q for v in p) for q in qs)
cg=parse(subprocess.run(['./outline'],input=inp,capture_output=True,text=True).stdout)
print('CG',[''.join(e[0] for e in c) for c in cg])
for eps in ('0','1e-5','1e-4','3e-4','1e-3'):
    for md in ('8','10','12'):
        rs=parse(subprocess.run(['/tmp/cgclean/target/release/cgclean','8','8'],input=inp.replace('\ns ','\nS '),capture_output=True,text=True,env={**os.environ,'CG_LINEEPS':eps,'CG_MAXDEPTH':md}).stdout)
        print(eps,md,[''.join(e[0] for e in c) for c in rs],[same(a,b,1e-6) for a,b in zip(cg,rs)])
