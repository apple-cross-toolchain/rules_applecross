import struct,sys
def tags(b,i):
    out=[]
    while True:
        name=b[i:i+4]; i+=4
        if name==b'ENDT': out.append(('ENDT',b'')); return out,i
        n=struct.unpack('<H',b[i:i+2])[0]; i+=2; out.append((name.decode(),b[i:i+n])); i+=n
def parse(b):
    h=struct.unpack('<HHHHHH',b[4:16]); q=struct.unpack('<QQQQQQQQQ',b[16:88])
    total,of,lf,op,lp,opr,lpr,om,lm=q
    n=struct.unpack('<I',b[of:of+4])[0]; i=of+4; funcs=[]
    for _ in range(n):
        sz=struct.unpack('<I',b[i:i+4])[0]; t,_=tags(b,i+4); funcs.append(t); i+=sz
    rest=b[i:op]
    return dict(hdr=h,q=q,funcs=funcs,gap=rest,pub=b[op:op+lp],priv=b[opr:opr+lpr],mod_len=lm)
if __name__=='__main__':
    for f in sys.argv[1:]:
        d=parse(open(f,'rb').read())
        print('==',f.split('/')[-1],'hdr',d['hdr'],'gap',d['gap'],'pub',d['pub'].hex()[:80],'priv',d['priv'].hex()[:40])
        for t in d['funcs']:
            print('   ',[(k, v.hex() if k not in ('NAME',) else v) for k,v in t if k not in ('HASH',)])
