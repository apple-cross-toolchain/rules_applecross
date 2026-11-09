# CoreGraphics fills (research, 2026-11-09)

Can this actool draw the bitmaps Xcode makes from PDF vector images byte for
byte, instead of approximately (`--config=relax_actool_vector`)? Of the
corpus's 63 catalogs that compile only relaxed for their vector images, 46
hold only PDFs. This is a time-boxed probe, black-box throughout: CoreGraphics'
public API from Swift on the Mac. It is not for merging, and the actool is
unchanged.

Status: in progress, inside a three-hour time-box (16:00Z to 19:00Z on
2026-11-09). An earlier version of this file said "tabled"; it was written
before the time-box ended. The scan conversion of sloped edges is now
found (below).

## Established

- **Xcode's bitmaps are CoreGraphics' drawing.** Xcode draws a vector image's
  page with `drawPDFPage` into an 8-bit premultiplied sRGB context at each
  scale. Seven of eight probe PDFs, drawn so and compiled by Xcode as PNGs,
  gave the same dmp2 payloads as Xcode's own vector renditions (`check.sh`).
  The eighth is likely an encoder tie.
- **Colours and blending.** DeviceRGB and DeviceGray become sRGB as
  `floor(v·255 + 0.5)`, and fills composite as 8-bit source-over. CMYK goes
  through a profile.
- **Rectangles take a fast path, solved.** A path that is a rectangle gets
  coverage per axis: 8 bits, `(w − 1) >> 5`, where `w` is measured in
  1/8192 of a pixel from the start edge snapped to its 1/256 step
  (`floor((s + 1)/32)`). The end edge is exact. Alpha is
  `floor(ax · ay / 255)`. This matches 400 of 400 random rectangles and every
  strip (`span_table.json`, `rects.json`).
- **Other paths: two fixed-point DDAs per edge and signed area per pixel.**
  Found by black-box probing with `cov` (`raster.py` is the model):
  1. Path coordinates go through float32, then round half up to 1/4096 of a
     pixel.
  2. The path is clipped to the canvas first. The top boundary fits best at
     H + 1, which is not understood yet.
  3. Each edge is stepped from its lower endpoint by two independent DDAs.
     Row crossings (x at each integer y) step by dx/dy truncated to 1/4096.
     Column crossings (y at each integer x) step by dy/dx truncated to
     1/4096. In each, the first crossing is the vertex plus
     `floor4096(frac · slope)`.
  4. The crossings, merged in order along the edge, make a polyline. Pieces
     between two row crossings belong to that row, even where a column
     crossing has drifted outside it (the two DDAs drift apart on long
     edges).
  5. Each pixel accumulates the signed integral of `clamp(x − X, 0, 1) dy`
     over its row's pieces (Green's theorem). Alpha is
     `floor(256 · |area|)`, capped at 255.

  The truncated slopes make long edges drift measurably: −0.022 px over 390
  rows for a slope of 0.522, exactly 390 times the truncation error. That
  drift is how the steppers were found.
- **Accuracy of the model** (pixels with 0 < alpha < 255, or wrongly
  predicted):

  | Test | Wrong / total |
  | --- | --- |
  | 20 random edges, 360–396 rows long, both windings (`gen20.py`, `signed.py`) | **0 / 10291** |
  | 240 random triangles in 8×8, vertices inside pixels (`trieval.py`) | 30 / ~3000 (1%) |
  | 200 wedges in 12×12, far vertices clipped (`wedge.py`) | 20 / 3208 (0.6%) |

  Nearly every remaining miss is within 0.07 of an alpha level, which is
  one 1/4096 unit of some crossing. A few sit exactly on an integer area
  next to a clip edge. They are unit-level rounding details of the
  vertex and clip handling, not structure. They were tested and don't come
  from per-piece area rounding, a constant bias, float32 slopes, or
  snapping edge extents to 1/256.
- **Curves are flattened uniformly** into a power-of-two number of segments
  by size (8, 16 and 32 for curves of about 4, 14 and 57 pixels).

## Remaining before exact fills

- The unit-level residuals at vertices and clip edges (about 1% of edge
  pixels).
- The clip box (why H + 1 at the top) and vertical edges. Vertical edges
  probe at `ceil(x · 4096) − 1`, and rectangles take their own fast path,
  solved.
- Curves through the same rasterizer: the flattening rule exactly.
- Even-odd fills, several subpaths, and overlapping paths composited in
  8 bits.
- Not examined yet: strokes (joins, caps, dashes), clipping paths, and
  CoreSVG for SVGs.
- A Rust implementation in actool, checked against `cov` on the corpus's
  46 PDF-only catalogs.

## Tools

- `cov.swift`: fills paths read from stdin into an alpha-only context and
  prints alpha.
- `render.swift`, `raw.swift`: draw a PDF page as Xcode does, to a PNG or
  as raw pixels.
- `mkpdf.py`: writes a one-page PDF.
- `compile.sh`, `bitmap.sh`, `check.sh`, `payloads.py`: compare Xcode's
  vector renditions with CoreGraphics' drawing.
- `raster.py`: the current model of the general fill.
- `trieval.py`, `wedge.py`, `gen20.py`, `signed.py`, `fastfit.py`,
  `anchor3.py`, `rowfit.py`, `drift3.py`: probes and scorers for triangles,
  wedges, long edges and drift. They import each other from `/tmp/cg`, with
  `cov` built there from `cov.swift`.
- `model.py`, `scan.py`, `area2.py`, `gray.py`, `tri.py`: earlier coverage
  models, superseded by `raster.py`.
