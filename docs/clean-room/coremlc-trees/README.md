# Core ML tree ensembles: how the compiled layout was found

The provenance record behind the tree ensemble support in
`toolchain/stubs/coremlc.py` ([coremlc.md](../coremlc.md#tree-ensembles)).
Every rule below was inferred only by building models with coremltools's
protobuf classes, compiling them with Xcode 27.0's `xcrun coremlc compile`,
and comparing the outputs. No Apple binary was inspected.

The probe models come from the scripts here, run on a Mac with coremltools
installed: `mk.py` (using coremltools's tree ensemble builder) and `b1.py` to
`b12.py` (on `probe.py`, which writes each model to `pm/` and Xcode's
compiled output to `p/`). The names in brackets are the probes that establish
each rule. `b11.py` and `b12.py` make random ensembles, generated after the
rules were fitted: the first 80 (RND*) exposed the "all spec leaves" rule
below; the next 100 (RNE*) then passed with no change. With the 48 Create ML
tree models, all 714 compiled models match.

Output is deterministic: the same spec compiled twice, and the same model in
two directories, give identical DAT, coremldata.bin and analytics
[Det1/Det2, S1 recompiled].

## Notation

- `W(m)` = 0 if m == 0, else `bitlen(m-1) + 1`. This is the width rule used for
  record fields [FI* sweep: fi 1..65536 → c = 1,2,3,3(4),4,4,4,4(8),5…;
  Ch* sweep for leaves; R_*/T_* for records].
- `bl1(m)` = max(1, bitlen(m)). This is the width rule for packed arrays
  [R_* roots: max 3 → 2 bits, 5 → 3; K4_max3: max dim 3 → 2 bits].
- "kept trees": trees whose root is a branch. Trees whose root is a leaf are
  folded into the base (see below).

## `_B0000.DAT` (all integers little-endian)

```
u64 0
u64 kind | (8 if double)          kind 3 = npd 1; 4 = one value per leaf;
                                  5 = sparse; 6 = dense      [N*, K4*, K5*, K6*, G_*]
u64 npd                           numPredictionDimensions    [N2_*, N3_*]
f64 base[npd]                     folded base (below)
u32 0
u16 recbytes                      = T + (8 if double else 4)
u8  c                             feature-index width
u8  d                             flag bits: 4, or 6 with the general mode
u8  e                             child-field width
u8  0
u8  g                             W(number of records)       [Ch*, R_*]
u8  h                             W(npd-1), +1 when kind 5   [K4n*, K6n*, K5n*]
u8  i                             kind 5: W(#spec leaves * npd), else 0 [S5_2t, S5_lf, GT_3_*, RND62]
u8  j                             mode family (below)
u16 0
u64 0
u64 npd
u64 nb = (records + 1) * recbytes  (one trailing all-zero record)
records, then pad to 8
ARRAY roots                       record index of each kept tree's root [T_*, R_*]
ARRAY idx                         kind 4: dim per leaf (width bl1(max dim));
                                  kind 5: per value (dim | last<<W(npd-1)), width h;
                                  else empty
u64 nvals; nvals x f32 (or f64); pad to 8
ARRAY tail                        kind 5: value offset of each kept tree [S5_2t, S5_3t]; else empty
u32 postEvaluationTransform       [TF0..TF3, TF1_n2]
```

ARRAY = `u32 0, u32 w, u64 count`. If count is 0, then `u64 0` (and w is
always 1). Otherwise `u64 a, u64 a`, then `a` u64 words holding the entries
LSB-first at `w` bits each, with
`a = floor((count-1)*w/64) + 3` [arrfit.py over all outputs; G_6_4_13 is
the only 65-bit case, giving 3 words where 70 bits give 4].

The unknown constant fields (the first u64, the u32 after base, the u64 0
before npd, header bytes 1, 5, 10 and 11) were 0 in all 487 files scanned.

### Base folding
`base[d] = basePredictionValue[d]` (zeros if the list is empty [NoBase]).
Then the value of every tree whose root is a leaf is added, in ascending
treeId order [FoldA: listed 2,0,1 → sorted; FoldB: the base is added first;
Fold2, FoldC]. Folded trees are dropped from the records. The sum is in f64
[Fold2 = 0.30000000000000004].

### Float vs double
Everything uses doubles (records' thresholds, all values, flag 8 in kind)
iff some branch threshold or some leaf evaluationValue in the spec (folded
leaves included) is not exactly representable as f32. NaN counts as not
representable; ±inf and -0.0 count as representable [D_thr, D_leaf,
D_fold, F_nan, F_inf, F_negz, D_thr_big]. basePredictionValue and the folded
sum are not checked [D_base, BaseD_leaf, FoldF, ThrF64]. A leaf node's own
branchFeatureValue is ignored [Leaf_thr]. relativeHitRate has no effect
[H_hit]. specificationVersion has no effect on the DAT [SV*].

### Tree order and records
Trees are taken in ascending treeId (ids need not be contiguous) [S2_rev,
Tree_order, Gap]. The root is the node no other node references; node ids and
their order in the spec do not matter [Ids, S1_ord, Sparse, Root_late]. Each
kept tree is written as a preorder walk over its branch nodes only, with the
true child first. Normalization: if exactly one child is a leaf and it is the
true child, the children are swapped, the mode is negated (LTE↔GT, LT↔GTE,
EQ↔NE) and the missing flag is inverted [Chain3/ChainR3, Mm_tf/Mm_ft,
Swap_eq/ne/lte/gte/gt_m]. Both-leaf and both-branch nodes are never swapped
[M_*, BB_*].

Leaves are numbered per tree (from 0) in record order: when a record is
emitted, its leaf children (true, then false) get the next numbers, before
the walk descends [ChainR3: leaves 10, 20, 40, 30; T_1_3].

Record = tag of `T = ceil((c+d+e)/8)` bytes (LE), then the threshold as f32
or f64.
- bits `[0, c)`: branchFeatureIndex. `c = W(max fi over records)` [FI*, S1f,
  FI8_n100: it depends on the indices used, not the input count].
- bit c: true child is a leaf. Bit c+1: false child is a leaf.
- bit c+2: missing goes to the (stored) true child [S1m, BB_*_1].
- with `d == 4`: bit c+3 = negate (mode is GT or GTE); with `d == 6`: bits
  c+3..c+5 = the TreeNodeBehavior enum (LTE 0 … NE 5) [MM_*, MM3, MM3n].
- bits `[8T-e, 8T)` (top-aligned) hold the child field [Bal2, Bal4, Ch8, Ch300]:
  - both children branch: record offset of the false child from this
    record (the true child is the next record) [Bal3, Bal4];
  - otherwise the number of the first leaf child (the second, if any, is
    +1). For kind 5 this is instead the value offset of that leaf inside its
    tree [S5_c3].
- `e = W(max leaves in one kept tree)` [Ch1..Ch300], or `W(that * npd)` for kind 5 [K5n*, S5_*].

No records (every tree is a single leaf, or there are no trees): c = 65, e = 65,
g = 0, so recbytes = 21 (f32) or 25 (f64), and nb = recbytes
[Bal0, L1, Fold2, N2_leaf, createml TreeR_*].

### j (mode family) and d
Over the spec's branch nodes, using their original modes and flags:
- all in {LTE, GT}: j = 0, or 1 if every missingValueTracksTrueChild is
  set [M_LessThanEqual, M_GreaterThan, MA_lte, Bal0 (vacuous: 1)];
- all in {LT, GTE}: j = 2, or 3 if every flag is set [S1, MA_lt2, MA_gte,
  MAs1, MAs2, MAs3, MA_lt_mix];
- anything else (mixed families, or any EQ/NE): j = 4, d = 6 [MM_*, M_Equal,
  MA_eq, MA_ltlte].
d = 4 when j < 4.

### Value layout (kind) [G_* grid: 180 points; GT_*, RND*]
L = all leaf nodes in the spec, folded ones included, and n = their total
evaluationInfo count.
- npd == 1 → kind 3: one value per leaf, in leaf order.
- else if `n < (3*L*npd)//4` → kind 4 if every leaf has exactly one value
  (idx = that dim), else kind 5 (values per leaf sorted by dim; idx entry =
  dim | last-of-leaf flag at bit W(npd-1)) [N3_sparse2, N3_order, K5n*].
- else kind 6: L × npd dense, 0.0 where a dim is missing [K5_a, K6_rev].
Values and idx cover kept-tree leaves only.

## coremldata.bin of a tree part (also the top-level file of a standalone tree model)

```
u32 302 (regressor) | 402 (classifier), u32 1, 20 x 00
u64 7 "generic"
u32 specificationVersion (as given) [SV1..SV9], 28 x 00
u64 len, ModelDescription (re-serialized, see below)
u64 0
u64 10 "_B0000.DAT"
u64 npd                                       [S1: 1, N2_s: 2, C2s: 1, C3s: 3]
classifier only:
  u64 nStringLabels, each u64 len + UTF-8 bytes  [C3s, C3s_utf]
  u64 nInt64Labels, each i64                     [C2i, C2i_neg, C3i]
```
Labels are in spec order. Output and probability names do not appear here
[C3s_names, C3s_noprob], and neither does the transform (it is in the DAT).

ModelDescription is re-serialized in canonical form [MD1, MD2, MD3]: the
fields are in field-number order (MD2 put outputs before inputs in the raw
spec), empty proto3 strings are dropped (MD3's explicit empty field 12), and
metadata (field 100) is always present, as `a2 06 00` when empty. Metadata
contents, shortDescriptions and userDefined are kept [MD1]. The encoder sorts
only the top-level fields; it copies nested messages and raises if a
FeatureDescription's fields are out of order (nested canonicalization was not
probed).

## Other files (not encoded)
- metadata.json: the usual schema. modelType.name is
  `MLModelType_treeEnsembleRegressor` / `MLModelType_treeEnsembleClassifier`.
  `availability` follows specificationVersion (sv1: macOS 10.13 / iOS 11.0;
  sv9: macOS 15.0 / iOS 18.0). Metadata strings go to author, license,
  version, shortDescription and userDefinedMetadata [MD1].
- analytics/coremldata.bin: `SpecificationDetails` with `modelHash`
  (base64) and `modelName`, the same generic shape as other model types. No
  tree-specific content was seen.

## Not established (the encoder raises `Unsupported`, or copies input as-is)
- Model.isUpdatable (field 10), any other non-tree Model field, and
  postEvaluationTransform > 3.
- Malformed trees: no unique root, unreachable or shared nodes, missing
  children, leaves without values, duplicate dims, dim ≥ npd. Xcode's
  validator rejects some of these [N1_empty, N1_dup].
- Nested ModelDescription canonicalization beyond FeatureDescription field
  order. Map ordering of userDefined with more than one entry.
- Widths above 255 bits (u8 header fields) and the meaning of g, i, and j's
  odd values for the runtime (they are encoded but not interpreted).
- The u64 0 before "_B0000.DAT" in coremldata.bin was always 0.
