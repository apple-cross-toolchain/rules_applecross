import struct,hashlib,sys
def tag(name,data): return name+struct.pack('<H',len(data))+data
def metallib(funcs,module,air=(2,9,0),lang=(4,1,0),hdr=(1,2,9,0x8200,0x1b,2)):
    """funcs: list of (name, type) ; module: wrapped bitcode bytes (single module shared)"""
    h=hashlib.sha256(module).digest()
    entries=b''; pub=b''; priv=b''
    for name,ftype in funcs:
        body=(tag(b'NAME',name.encode()+b'\0')+tag(b'TYPE',bytes([ftype]))+tag(b'HASH',h)
              +tag(b'MDSZ',struct.pack('<Q',len(module)))+tag(b'OFFT',struct.pack('<QQQ',0,0,0))
              +tag(b'VERS',struct.pack('<HHHH',air[0],air[1],lang[0],lang[1])))
        entries+=struct.pack('<I',len(body)+8)+body
        tails=b'ENDT'
        pub+=struct.pack('<I',4)+b'ENDT'; priv+=struct.pack('<I',4)+b'ENDT'
    flist=struct.pack('<I',len(funcs))+entries
    gap=b'ENDT'*(len(funcs)+1)   # entry end tag(s) and list end, outside the counted section
    head_len=0x58
    o_f=head_len; o_pub=o_f+len(flist)+len(gap); o_priv=o_pub+len(pub); o_mod=o_priv+len(priv)
    total=o_mod+len(module)
    head=b'MTLB'+struct.pack('<HHHHHH',*hdr)+struct.pack('<QQQQQQQQQ',total,o_f,len(flist),o_pub,len(pub),o_priv,len(priv),o_mod,len(module))
    assert len(head)==head_len, len(head)
    return head+flist+gap+pub+priv+module
def wrap(bc):
    """bitcode wrapper (LLVM writes one itself for apple triples), padded to 16 bytes"""
    if bc[:4]!=bytes.fromhex('dec0170b'):
        bc=struct.pack('<IIIII',0x0B17C0DE,0,20,len(bc),0xFFFFFFFF)+bc
    return bc+b'\0'*(-len(bc)%16)
if __name__=='__main__':
    ref=open('/tmp/metalprobe/k.metallib','rb').read()
    w=ref.find(bytes.fromhex('dec0170b'))
    out=metallib([('k',2)],ref[w:])
    print('identical',out==ref,len(out),len(ref))
    if out!=ref:
        for i,(x,y) in enumerate(zip(out,ref)):
            if x!=y: print('first diff at',hex(i)); break
