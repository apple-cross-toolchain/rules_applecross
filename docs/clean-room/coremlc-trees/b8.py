from probe import *
from coremltools.proto import Model_pb2
def s1(): return [B(0,0,0,3.0,1,2), L(0,1,10.0), L(0,2,20.0)]
p = reg('MD1', s1(), nfeat=2)
m = Model_pb2.Model(); m.ParseFromString(open(p,'rb').read())
md = m.description.metadata; md.author='me'; md.shortDescription='desc'; md.license='MIT'; md.versionString='1.2'; md.userDefined['k']='v'
m.description.input[0].shortDescription = 'in0'; m.description.output[0].shortDescription='out'
save('MD1', m)
# reorder raw: output field (10) before input fields; and unknown field
p = reg('MD2', s1(), nfeat=2)
m = Model_pb2.Model(); m.ParseFromString(open(p,'rb').read())
desc = m.description
ins = b''.join(b'\x0a' + bytes([len(i.SerializeToString())]) + i.SerializeToString() for i in desc.input)
outs = b''.join(b'\x52' + bytes([len(o.SerializeToString())]) + o.SerializeToString() for o in desc.output)
raw = outs + b'\x5a\x01y' + ins
m.ClearField('description')
body = m.SerializeToString()
body = b'\x12' + bytes([len(raw)]) + raw + body
open(os.path.join(PM,'MD2.mlmodel'),'wb').write(body)
compile_all(['MD1','MD2'], force=True)
