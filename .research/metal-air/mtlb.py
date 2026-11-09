"""Write a metallib (FORMAT.md). rebuild.py checks it against Xcode's files;
the M1 writer will derive the header and metadata from the target and source."""
import struct,hashlib
def tag(name,data): return name+struct.pack('<H',len(data))+data
EMPTY=struct.pack('<I',4)+b'ENDT'
def metallib(funcs,hdr):
    """funcs: list of dict(name,type,module(wrapped+padded),pub(bytes block),priv(bytes block),air(2,9),lang(4,1))"""
    entries=b''; pub=b''; priv=b''; mods=b''
    for f in funcs:
        body=(tag(b'NAME',f['name'].encode()+b'\0')+tag(b'TYPE',bytes([f['type']]))+tag(b'HASH',hashlib.sha256(f['module']).digest())
              +tag(b'MDSZ',struct.pack('<Q',len(f['module'])))+tag(b'OFFT',struct.pack('<QQQ',len(pub),len(priv),len(mods)))
              +tag(b'VERS',struct.pack('<HHHH',*f['air'][:2],*f['lang'][:2]))+b'ENDT')
        entries+=struct.pack('<I',len(body)+4)+body
        pub+=f.get('pub',EMPTY); priv+=f.get('priv',EMPTY); mods+=f['module']
    flist=struct.pack('<I',len(funcs))+entries
    o_f=0x58; o_pub=o_f+len(flist)+4; o_priv=o_pub+len(pub); o_mod=o_priv+len(priv); total=o_mod+len(mods)
    head=b'MTLB'+struct.pack('<HHHHHH',*hdr)+struct.pack('<QQQQQQQQQ',total,o_f,len(flist)-4,o_pub,len(pub),o_priv,len(priv),o_mod,len(mods))
    return head+flist+b'ENDT'+pub+priv+mods
def wrap(bc):
    if bc[:4]!=bytes.fromhex('dec0170b'):
        bc=struct.pack('<IIIII',0x0B17C0DE,0,20,len(bc),0xFFFFFFFF)+bc
    return bc+b'\0'*(-len(bc)%16)
