import sys, os, json
import coremltools as ct
from coremltools.models.tree_ensemble import TreeEnsembleRegressor, TreeEnsembleClassifier
from coremltools.models import datatypes
out = sys.argv[1]
def reg(name, nfeat, build, base=None, dims=1):
    feats = [("x%d" % i, datatypes.Double()) for i in range(nfeat)]
    b = TreeEnsembleRegressor(feats, "y" if dims == 1 else [("y", datatypes.Array(dims))])
    b.set_default_prediction_value(base if base is not None else [0.0] * dims)
    build(b)
    spec = b.spec
    spec.specificationVersion = 1
    ct.utils.save_spec(spec, os.path.join(out, name + ".mlmodel"))
def leaf(v): return lambda b: b.add_leaf_node(0, 0, v)
reg("L1", 1, leaf(1.0))
reg("L2", 1, leaf(2.0))
reg("L1b", 1, leaf(1.0), base=[0.5])
reg("L1f3", 3, leaf(1.0))
def split(b, mode="BranchOnValueLessThan", fi=0, val=3.0, missing=False):
    b.add_branch_node(0, 0, fi, val, mode, 1, 2, missing_value_tracks_true_child=missing)
    b.add_leaf_node(0, 1, 10.0); b.add_leaf_node(0, 2, 20.0)
reg("S1", 2, split)
reg("S1v", 2, lambda b: split(b, val=4.5))
reg("S1f", 2, lambda b: split(b, fi=1))
reg("S1m", 2, lambda b: split(b, missing=True))
for m in ["BranchOnValueLessThanEqual", "BranchOnValueLessThan", "BranchOnValueGreaterThanEqual", "BranchOnValueGreaterThan", "BranchOnValueEqual", "BranchOnValueNotEqual"]:
    reg("M_" + m[13:], 2, lambda b, m=m: split(b, mode=m))
def two(b):
    b.add_leaf_node(0, 0, 1.0); b.add_leaf_node(1, 0, 2.0)
reg("T2", 1, two)
def ids(b):
    b.add_branch_node(0, 5, 0, 3.0, "BranchOnValueLessThan", 9, 7)
    b.add_leaf_node(0, 9, 10.0); b.add_leaf_node(0, 7, 20.0)
reg("Ids", 1, ids)
