from probe import *
from coremltools.proto import Model_pb2
p = reg('MD3', [B(0,0,0,3.0,1,2), L(0,1,10.0), L(0,2,20.0)], nfeat=1)
m = Model_pb2.Model(); m.ParseFromString(open(p,'rb').read())
raw = m.description.SerializeToString() + b'\x62\x00' + b'\xa2\x06\x00'
m.ClearField('description')
body = b'\x12' + bytes([len(raw)]) + raw + m.SerializeToString()
open(os.path.join(PM,'MD3.mlmodel'),'wb').write(body)
compile_all(['MD3'], force=True)
