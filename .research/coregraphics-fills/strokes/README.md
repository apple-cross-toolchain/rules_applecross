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
  X = (8M' − P0' − P3')/6, where M' is B(½) offset by d along the normal
  there: the point that makes a cubic with C1 = C2 = X pass through M'
  (fbnorm.py, exact to 1e-13). Pieces with a chord under about 0.1 px, and
  the piece between the two coordinate pre-splits when its chord is under
  about 0.5 px, get C1 = C2 = X. These two thresholds are fitted to the
  corpus, not measured.
- The pre-split applies when the control polygon is S-shaped:
  cross(P1−P0, P2−P1)·cross(P2−P1, P3−P2) ≤ 0, which includes straight
  cubics.
- With these rules, 15 of the 22 corpus curves that split match
  CoreGraphics element for element to 1e-9 (elemcmp.py). Open cases:
  curves that turn faster than the offset radius (CoreGraphics inserts
  join-like runs of lines there), long chains of tiny pieces, and pieces
  with P2 = P3 that CoreGraphics rejects although every error measure I
  tried is tiny.
- Corpus pixels with my own outlines: blocks needing splits 39, 101 and
  158 wrong at 1×, 2× and 3×; all other wide-stroke blocks 2, 1 and 0.

## Rust port

`rust/cgstroke.rs` ports this stroker (the probe's `s W J C` command
strokes and fills; `S W J C` prints the outline). It matches
copy(strokingWithWidth:) on 600 of 600 random polylines. Two more fitted
rules are on top of the Python:
- A piece with a control point on an endpoint and a chord over 1 px is
  rejected. CoreGraphics does reject such pieces although every error
  measure I tried is tiny, but the threshold varies with width and shape,
  so this rule is fitted.
- A piece whose zero-width refit has a negative handle is rejected,
  except the middle piece and pieces under 0.1 px. This raises random
  single curves from 111 to 119 of 400 matching.

With these, 17 of 22 corpus curves that split match. End to end, stroking
and filling in Rust against CoreGraphics' own stroke render, the corpus
wide strokes are wrong in 25, 70 and 107 of 6,529, 15,302 and 23,914 edge
pixels at 1×, 2× and 3× (corprs.py). The five curves that still differ:
two that turn faster than the offset radius (CoreGraphics emits runs of
lines there), two chains of tiny pieces with one extra split each, and
one piece containing an inflection near its end that CoreGraphics halves
with errors near 0.002. A further 2% of convex-curve sides are rejected
within a band of widths for reasons I haven't found.

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
- Sloped width-1 lines (x-major) run on a fixed-point DDA in 1/4096 px.
  The slope is trunc((dy·4096 + dx/2)/dx) with C integer division: half-up
  for rising lines, and descending lines lose up to 1.5 units (thin14.py,
  exact on 60 slopes). Each column is a band [bot, bot + T] in 1/256 units,
  T = floor(256·secθ), with bot = floor((A + s·k)/16). For a horizontal
  line bot = floor((y − ½)·256) + 1 (thin15.py). The lower row gets
  ceil(area) and the upper row T minus that.
- Not found yet: the start offset A (it is about 9–16 units above the
  exact band bottom and depends on the endpoints in a way I haven't
  matched, thin19.py), columns where the band's edge crosses a row inside
  the column, three-row columns of steeper lines, the ends, and curves.
- The corpus has one thin block (logo-open live, w = 1.0 at 1×). It is
  now the largest remaining corpus gap: filling CoreGraphics' own width-1
  outline gets 770 of its 1,259 edge pixels wrong, mostly by ±1 to 3.
