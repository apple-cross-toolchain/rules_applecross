"""Probe builder: constructs tree-ensemble .mlmodel specs directly and compiles them with xcrun coremlc."""
import os, sys, subprocess, concurrent.futures as cf
import coremltools as ct
from coremltools.proto import Model_pb2, FeatureTypes_pb2
D = os.path.dirname(os.path.abspath(__file__))
PM = os.path.join(D, 'pm'); PO = os.path.join(D, 'p')
os.makedirs(PM, exist_ok=True); os.makedirs(PO, exist_ok=True)
MODES = dict(LTE=0, LT=1, GTE=2, GT=3, EQ=4, NE=5, LEAF=6)
def B(tree, node, fi, val, t, f, mode='LT', miss=False, hit=0.0):
    return dict(treeId=tree, nodeId=node, nodeBehavior=MODES[mode], branchFeatureIndex=fi,
                branchFeatureValue=val, trueChildNodeId=t, falseChildNodeId=f,
                missingValueTracksTrueChild=miss, relativeHitRate=hit)
def L(tree, node, vals, hit=0.0):
    if not isinstance(vals, (list, tuple)): vals = [(0, vals)]
    elif vals and not isinstance(vals[0], tuple): vals = list(enumerate(vals))
    return dict(treeId=tree, nodeId=node, nodeBehavior=6, ev=vals, relativeHitRate=hit)
def fill(te, nodes, npd, base):
    te.numPredictionDimensions = npd
    te.basePredictionValue.extend(base)
    for n in nodes:
        p = te.nodes.add()
        for k, v in n.items():
            if k == 'ev':
                for i, x in v:
                    e = p.evaluationInfo.add(); e.evaluationIndex = i; e.evaluationValue = x
            else:
                setattr(p, k, v)
def reg(name, nodes, nfeat=1, npd=1, base=None, transform=0, sv=1, arrin=False, outarr=None, outname='y'):
    m = Model_pb2.Model(); m.specificationVersion = sv
    if arrin:
        i = m.description.input.add(); i.name = 'x'
        i.type.multiArrayType.shape.append(nfeat); i.type.multiArrayType.dataType = FeatureTypes_pb2.ArrayFeatureType.DOUBLE
    else:
        for k in range(nfeat):
            i = m.description.input.add(); i.name = 'x%d' % k; i.type.doubleType.SetInParent()
    o = m.description.output.add(); o.name = outname
    if npd == 1 and not outarr: o.type.doubleType.SetInParent()
    else:
        o.type.multiArrayType.shape.append(npd); o.type.multiArrayType.dataType = FeatureTypes_pb2.ArrayFeatureType.DOUBLE
    m.description.predictedFeatureName = outname
    fill(m.treeEnsembleRegressor.treeEnsemble, nodes, npd, base if base is not None else [0.0] * npd)
    m.treeEnsembleRegressor.postEvaluationTransform = transform
    return save(name, m)
def cls(name, nodes, labels, nfeat=1, npd=None, base=None, transform=0, sv=1, probname='classProbability', outname='label'):
    m = Model_pb2.Model(); m.specificationVersion = sv
    for k in range(nfeat):
        i = m.description.input.add(); i.name = 'x%d' % k; i.type.doubleType.SetInParent()
    ints = isinstance(labels[0], int)
    o = m.description.output.add(); o.name = outname
    if ints: o.type.int64Type.SetInParent()
    else: o.type.stringType.SetInParent()
    m.description.predictedFeatureName = outname
    if probname:
        o = m.description.output.add(); o.name = probname
        if ints: o.type.dictionaryType.int64KeyType.SetInParent()
        else: o.type.dictionaryType.stringKeyType.SetInParent()
        m.description.predictedProbabilitiesName = probname
    c = m.treeEnsembleClassifier
    if npd is None: npd = 1 if len(labels) == 2 else len(labels)
    fill(c.treeEnsemble, nodes, npd, base if base is not None else [0.0] * npd)
    c.postEvaluationTransform = transform
    if ints: c.int64ClassLabels.vector.extend(labels)
    else: c.stringClassLabels.vector.extend(labels)
    return save(name, m)
JOBS = []
def save(name, m):
    path = os.path.join(PM, name + '.mlmodel')
    with open(path, 'wb') as f: f.write(m.SerializeToString())
    JOBS.append(name); return path
def compile_all(names=None, force=False):
    names = names or JOBS
    def one(n):
        od = os.path.join(PO, n)
        if not force and os.path.exists(os.path.join(od, n + '.mlmodelc')): return n, 0, ''
        subprocess.run(['rm', '-rf', od]); os.makedirs(od)
        r = subprocess.run(['xcrun', 'coremlc', 'compile', os.path.join(PM, n + '.mlmodel'), od], capture_output=True, text=True)
        return n, r.returncode, (r.stdout + r.stderr)[-400:]
    with cf.ThreadPoolExecutor(8) as ex:
        for n, rc, out in ex.map(one, names):
            if rc: print('FAIL', n, out.strip().replace('\n', ' | '))
def dat(n):
    p = os.path.join(PO, n, n + '.mlmodelc', '_B0000.DAT')
    return open(p, 'rb').read() if os.path.exists(p) else None
