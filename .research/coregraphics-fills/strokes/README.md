# Stroke outlines

Strokes wider than one pixel are the fill of `CGPath.copy(strokingWithWidth:)`.
These scripts compare my own outline construction with that call (`outline`
in the parent directory prints its elements).

## Lines: solved

`stroke.py` matches the call to 1e-9 on 9,000 random open and closed
polylines with every join and cap (`strokecmp.py SEED N CLOSED JOIN CAP`):

- Each segment's offset ends are joined directly.
- If the gap between one segment's end offset and the next one's start
  offset has gap² < 0.01, nothing is emitted for the join and the next
  start is dropped.
- Outer side: a miter point if the miter ratio is within the limit and
  gap² ≥ 0.05, otherwise a bevel. A round join is one cubic with handles
  4/3·tan(θ/4)·d if gap² ≥ 0.05, otherwise a line. θ is snapped to π when
  the turn is within 0.001 of a reversal (|cross| < 0.001), and the arc
  then goes on the reverse traversal.
- Inner side: a straight line, through the vertex when either segment is
  shorter than d.
- Closed paths: the left contour runs forward; the right contour starts
  at the first segment's right offset, takes the first vertex's join, then
  runs the segments backward. If that join collapses, it starts at the
  closing segment's offset instead.

## Curves: partly solved

- Each output piece runs between exact offsets of its ends, with handles
  along the original handle directions, and their lengths are linear in d.
- At d = 0 the piece is not the curve itself but a refit: C1 = 2·X1 −
  (P0+P1)/2, where X1 is where the start tangent line meets the tangent
  line at B(½), and likewise C2. This is affine-invariant and rational.
- Collinear cubics give C1 = C2 = (P1+P2)/2.
- The lengths change linearly with d at a rate that depends only on two
  angles. With t1 = tan(∠(u0, um)/2) and t2 = tan(∠(um, u3)/2), where um
  is the direction of B'(½): L1(d) = L1(0) − G(t1, t2)·d and L2(d) =
  L2(0) − G(t2, t1)·d, with G(a, b) = (4/3)·a + (b² − a²)/(6·a·(1 + b²)).
  I found G by an exact rational fit to 48 measured slopes (ratfit.py,
  gtab4.py). `offmodel.py` matches the call to 2.5e-14 on random unsplit
  curves. For symmetric curves G reduces to the arc rule (4/3)·tan(θ/4).
- With this, my own outlines of the corpus strokes that need no split fill
  to 2, 1 and 0 wrong pixels (of 5,058, 12,435 and 19,428 edge pixels) at
  1×, 2× and 3× (corpstroke2.py).
## Curve splits: mostly known

- Acceptance is decided per side. A piece is accepted if, for t in
  {0.1, 0.3, 0.5, 0.7, 0.9}, every point of the true offset at t is
  within 0.1 px of the piece. This gives exactly 0.100000 at all 20
  measured boundaries (bnd6.py, bnd7.py); even tenths would exceed it. On
  3,000 random curves it matches 96.8% of whole-curve decisions; the rest
  are rejections of small, sharply curved pieces in a band of widths that
  I haven't explained (accside.py).
- Rejected pieces are halved at ½.
- If the cubic has a true inflection in (0, 1), it is first split where
  x''(t) = 0 and where y''(t) = 0, the inflections of the device-space
  coordinate functions (t = a/(a − b) with a = P0 − 2P1 + P2 and
  b = P1 − 2P2 + P3, per axis). The split is width- and scale-independent
  but depends on rotation. The family (0,0),(3,λ),(7,−1),(10,0) splits at
  ½ (x'') and (2λ+1)/(3λ+3) (y''), exactly.
- A handle whose zero-width refit length is negative is replaced by
  (P1+P2)/2 offset by d along (roughly) the normal of B'(½); the exact
  direction is about 1e-4 rad off every candidate I tried. Pieces with a
  chord under about 0.1 px, and the piece between the two coordinate
  pre-splits when its chord is under about 0.5 px, get C1 = C2 = that
  point. These thresholds are fitted to the corpus, not measured.
- With these rules, 11 of the 22 corpus curves that split match
  CoreGraphics element for element (elemcmp.py). Open cases: pieces with
  P2 = P3 that CoreGraphics rejects although every error measure I tried
  is tiny, and curves whose outline contains runs of lines at offset
  cusps.
- Corpus pixels with my own outlines: blocks needing splits 40, 100 and
  153 wrong at 1×, 2× and 3× (bubble tail 0/0/1, userAvatar 23/59/94,
  icon-add 16/39/57, icon-setting 1/2/1); all other wide-stroke blocks 2,
  1 and 0.

## Thin strokes (w ≤ 1)

- Axis-aligned lines take their own path: coverage per axis in 1/256,
  with the band's low edge floored and its high edge ceiled; alpha =
  floor(hx·vy/256) − 1 (`thinsweep.py`).
- Sloped lines narrower than 1 are roughly the width-1 result scaled by
  floor(256·w)/256 (90% of pixels; the rest are near zero).
- Sloped width-1 lines: each interior column totals floor(256·secθ)
  exactly, but the split between rows differs from area by up to ±2, and
  a 1/256 translation sweep jitters step by step. That looks like
  fixed-point area arithmetic I haven't identified.
- The corpus has one thin block (logo-open live, w = 1.0 at 1×).
