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
- CoreGraphics also pre-splits the source curve. The split points are the
  same on both sides and don't depend on width or scale; the rule is not
  found yet.
