# Take each metallib apart and rebuild it with mtlb.py; report any that differ.
# Usage: python3 rebuild.py out/*/*.metallib
import os,sys,struct; sys.path.insert(0,os.path.dirname(os.path.abspath(__file__))); import mtlb, parse
def split(b):
    d=parse.parse(b); total,of,lf,op,lp,opr,lpr,om,lm=d['q']; fl=d['funcs']; funcs=[]
    for k,t in enumerate(fl):
        T=dict(t); po,pr,mo=struct.unpack('<QQQ',T['OFFT']); ms=struct.unpack('<Q',T['MDSZ'])[0]
        nxt=[struct.unpack('<QQQ',dict(x)['OFFT']) for x in fl[k+1:k+2]]
        pe=op+(nxt[0][0] if nxt else lp); pre=opr+(nxt[0][1] if nxt else lpr)
        v=struct.unpack('<HHHH',T['VERS'])
        funcs.append(dict(name=T['NAME'][:-1].decode(),type=T['TYPE'][0],module=b[om+mo:om+mo+ms],pub=b[op+po:pe],priv=b[opr+pr:pre],air=v[:2],lang=v[2:],tags=[a for a,_ in t]))
    return d,funcs
if __name__=='__main__':
    ok=bad=0; badf=[]
    for f in sys.argv[1:]:
        b=open(f,'rb').read(); d,funcs=split(b)
        if mtlb.metallib(funcs,d['hdr'])==b: ok+=1
        else: bad+=1; badf.append(f)
    print('identical',ok,'differ',bad); print('\n'.join(badf[:12]))
