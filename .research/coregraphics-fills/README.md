# CoreGraphics fills (research, 2026-11-09)

Can this actool draw the bitmaps Xcode makes from PDF vector images byte for
byte, instead of approximately (`--config=relax_actool_vector`)? Of the
corpus's 63 catalogs that compile only relaxed for their vector images, 46
hold only PDFs. This is a time-boxed probe, black-box throughout: CoreGraphics'
public API from Swift on the Mac. It is not for merging, and the actool is
unchanged.

Status: the three-hour time-box (16:00Z to 19:00Z on 2026-11-09) found how
CoreGraphics scan-converts general paths. On straight-edged shapes the
model matches 99–100% of edge pixels; on glyph curves, 85–97%. Every
remaining miss is off by one level, and most are within a few hundredths
of a level of the boundary.

## Established

- **Xcode's bitmaps are CoreGraphics' drawing.** Xcode draws a vector image's
  page with `drawPDFPage` into an 8-bit premultiplied sRGB context at each
  scale. Seven of eight probe PDFs, drawn so and compiled by Xcode as PNGs,
  gave the same dmp2 payloads as Xcode's own vector renditions (`check.sh`).
  The eighth is likely an encoder tie.
- **Colours and blending.** DeviceRGB and DeviceGray become sRGB as
  `floor(v·255 + 0.5)`, and fills composite as 8-bit source-over. CMYK goes
  through a profile.
- **Rectangles take a fast path, solved.** A lone axis-aligned
  rectangle gets coverage per axis, 8 bits. Both edges are floored to
  1/8192 of a pixel, from float32. The start edge snaps down to its 1/256
  step (`floor((s + 1)/32) · 32`), and a pixel's coverage on that axis is
  `(w − 1) >> 5`. Alpha is `floor(ax · ay / 255)`. This matches 0 wrong of
  7721 edge pixels of random multi-pixel rectangles, as well as the single-pixel
  rectangles and strips (`span_table.json`, `rects.json`). It's in the Rust
  prototype (`rect_fast`).
- **Other paths: fixed-point DDAs per edge and a band sweep.** Found by
  black-box probing with `cov` (`raster.py`, `fill_path`, is the model):
  1. Path coordinates go through float32, then round half up to 1/4096 of a
     pixel. The path is clipped to the canvas.
  2. Each edge is stepped from its lower endpoint by two independent DDAs.
     Row crossings (x at each integer y) step by dx/dy truncated to 1/4096.
     Column crossings (y at each integer x) step by dy/dx truncated to
     1/4096. The first crossing is the vertex plus
     `floor4096(frac · slope)`. The stepper keeps that fraction, so any
     other x on the edge (its end, and its band points) is the floor of the
     line through the lower vertex at the truncated slope. The edge ends
     there at its top y, not at the vertex.
  3. Each edge also gets a point at every vertex y of the path inside its
     span, at that same floored x.
  4. Between consecutive points, a column crossing belongs to the row whose
     two row crossings bracket it in x, even when it has drifted outside
     that row in y.
  5. Each row is swept in bands, cut at every vertex y and every edge
     intersection. In each band the edges are ordered by x, and the winding
     rule (nonzero or even-odd) picks the span boundaries. Only those
     contribute their area to the right, by Green's theorem per pixel.
     Alpha is `floor(256 · area)`, capped at 255.
  6. Vertical edges stay exact here (the 1/256 snap is the rectangle fast
     path's).
  7. Cubics are halved recursively at t = ½ while
     ¾ · max(|Δ²x|, |Δ²y|) of the control points exceeds 1/20 px. This is
     adaptive, so the middle of an arc stays coarser than its ends. The
     same tolerance (≈ 0.0667) fits every one of 40 random ellipses.
  8. Strokes 1 px wide or wider equal the fill of their outline. Thinner
     strokes take another path.

  The truncated slopes make long edges drift measurably: −0.022 px over 390
  rows for a slope of 0.522, exactly 390 times the truncation error. That
  drift is how the steppers were found.
- **Accuracy of the model** (pixels with 0 < alpha < 255, or wrongly
  predicted):

  | Test | Wrong / total |
  | --- | --- |
  | 20 random edges, 360–396 rows long, both windings (`gen20.py`) | **0 / 10291** |
  | 16 long shallow edges (`genshallow.py`) | 4 / 6980 |
  | 150 wedges in 64×64, unclipped (`wedge2.py`) | 13 / 14159 |
  | 200 wedges in 12×12, clipped (`wedge.py`) | 9 / 3216 |
  | 240 random triangles in 8×8 (`trieval.py`) | 21 / ~3000 |
  | 2 overlapping triangles, nonzero / even-odd (`multi.py`) | 26 / 2444, 24 / 2528 |
  | 40 ellipses of 4 cubics (`ell.py`) | 53 / 2679 |
  | Helvetica "AXWNZ", "EHLTI" at 20 px (`pathcmp.py`) | 7 / 427, 1 / 266 |
  | Helvetica "Rg&8", Times "Qa@" (curves) | 54 / 369, 56 / 324 |

- **The residual.** Every miss is ±1 level, in rows that hold a vertex. In
  those rows CG's coverage averages about 1/32 of a level above the model's
  (`bias.py`). Its own pixel gains about 0.04 per vertex, and curves with
  many vertices per row add these up. A single vertex of any kind (bottom,
  top or middle) swept in 1/4096 steps matches (`vprobe.py`). These don't
  explain it: per-piece or per-band area rounding, a constant or role-based
  bias, chaining edges through vertices, splitting curves at extrema,
  fixed-point curve subdivision, coarser vertex y, and outward rounding.
  Float and 16-bit contexts, and partial-alpha fills, show only the 8-bit
  coverage (`covf.swift`, `cova.swift`), so the probes see nothing finer.

- **Clipping.** The clip path becomes an 8-bit mask by the same
  rasterizer. The fill is clipped to the mask's integer bounding box,
  rasterized, and combined as `(a · b + 127) // 255`: 1 of 451 pixels
  wrong (`covc.swift`).
- **Strokes.** A stroke wider than 1 device pixel is exactly the nonzero
  fill of `CGPath.copy(strokingWithWidth:lineCap:lineJoin:miterLimit:)`:
  60 of 60 random polylines and curves, every join and cap
  (`strokeq.swift`). That public call shows the outline element by element
  (`outline.swift`): exact double offsets, per-segment rectangles stitched
  with join wedges into one self-overlapping polygon, round joins and caps
  as cubics, and curves offset as cubics (two per side for a simple curve).
  A stroke 1 px wide or less, including exactly 1, takes another rasterizer.
  The outline construction is plain:
  - **Polyline:** the left offset of each segment; straight across to the
    next segment's left offset on the inner side, with no join, just a
    self-overlap; a butt cap; then the right side back, with the miter
    point at the outer corner (none for bevel). All coordinates are exact
    double offsets.
  - **Curves:** each side is offset cubics (one per side for a symmetric
    curve, two for an asymmetric one). Their ends are the exact normal
    offsets and their handles keep the original tangent directions, scaled
    (1.1635 in the probe) so the curve's midpoint lands on the offset of
    the original midpoint. This is not Tiller–Hanson; the split rule isn't
    found yet.
- **Corpus PDFs.** `pdfpaths.swift` walks a page with `CGPDFScanner`,
  including form XObjects, and prints each fill in device pixels. Over the
  26 PDFs left in the corpus's working directory at 2×, the model gets
  1632 of 1687 fill edge pixels right (`pdffills.py`). Every one of those
  icons also uses strokes and clips.

- **Rust prototype** (`rust/`). `cgfill.rs` is the model in Rust: integer
  1/4096 steppers, the band sweep, and flattening. It reproduces `raster.py`
  (triangles 17, Rg&8 54, overlaps 26, n-gons 57 wrong) and runs every
  suite in about a second (`rsc.py`). `cgfill_base.rs` is the same with
  environment knobs for the experiments. In it, these all scored no better
  than the model: outward, inward, ceil or round for the first crossings,
  ends and band points; band points on the polyline; and flattening by
  other measures, tolerances, or uniform halving.
- **Corpus, with the rectangle path** (`corpus_rs.py`). Fills are right on
  1607 of 1625 edge pixels at 1×, 3231 of 3285 at 2×, and 4948 of 5011 at
  3× (98.4–98.9%). The misses are curves.
- **Corpus strokes.** CG's outline filled by the model gets 6588 of 6884
  stroke edge pixels right (`pdfstrokes.py`). Straight strokes are exact;
  the misses are on curves.
- **Thin strokes.** A thin axis-aligned stroke follows the rectangle fast
  path exactly. A thin diagonal one is within ±1 of the 1 px outline's fill
  along its length, but its ends differ (`thin.swift`).

- **The residual, narrowed** (`dense.py`, `suite.py`, and the knobs in
  `rust/cgfill_base.rs`):
  - It grows with vertex density. Regular polygons of radius 10 miss 0.4%
    of edge pixels at 8 vertices and 22% at 512.
  - The model is always one level low.
  - The misses are only on edges that open a span (the shape's left side),
    for either winding. Edges that close a span are exact.
  - Flooring an opening piece's left area at 1/4096 px², or adding half a
    unit per opening piece, in rows a vertex splits into bands, fixes the
    dense polygons and the glyphs. Dense polygons go from 319 misses to 52;
    Rg&8 from 54 to 11; small text from 95 to 8.
  - But either rule over-corrects triangles (113 to 258), so neither is
    the mechanism.
  - A cleaner probe (`colin.py`): split one side of a quadrilateral into n
    collinear segments. CG's output on the opening side hardly changes with
    n (at most 3 pixels), while the model drifts. Splitting the closing
    side, the model follows CG exactly up to n = 30. Long opening edges are
    exact (0 of about 15,000 edge pixels), so the steppers are right; the
    difference is what happens at vertices. The misses that remain sit
    where one edge's stepped end and the next edge's start are a unit apart,
    and on the closing edge's band point in a row holding an opening-side
    vertex. That band point fits rounded up there but not in general. These
    didn't help: chaining edges through vertices on either side, band
    points on only one side, and truncating toward zero. These didn't help: any rounding of closing edges,
    side-dependent vertex rounding, and float32 curve subdivision.

- **The band point on a closing edge** (`sweep1.py`). One vertex is swept
  along a straight opening edge, splitting it into two collinear segments.
  Next to the vertex, CG's output is the same as for the unsplit edge (1200
  of 1200 pixels). On the closing edge in that row it changes. There the
  model is one low with the stepped, floored band point, and one high with
  no band point; rounding to nearest leaves 17 low. So the closing edge
  gets a band point a little right of the floored one. These didn't place
  it: stepping from the row crossing above or the nearest one, the true
  line, and quantizing each band's coverage per pixel at 1/4096 to 1/32768.
  - **Solved:** the band point sits at the floor of the exact stepped value.
    The stepper keeps the fraction its first step dropped (0.6045 for this
    edge), and CG's data splits exactly at 1 − 0.6045. With that, this
    sweep goes from 101 misses to 8, the suite from 996 to 986, and corpus
    fills to 98.3–99.0%. The 8 left are the vertex's own pixel on the
    opening side. Snapping edge y extents to 1/256 makes things far worse,
    so vertex y stays at 1/4096.

- **Where the residual stands (18:10Z).** With the exact stepper, the
  remaining misses are the opening side's vertex pixels in rows with
  vertices: the model is about 0.03 of a level low per vertex. Corrections
  that score better without being the mechanism (`CG_ACC=1` plus the knob
  shown; suite total 986 with none):

  | Knob | Suite total | Triangles | Dense | Glyphs Rg&8 / Qa@ / 11 px |
  | --- | --- | --- | --- | --- |
  | `CG_PQ=4096:floor CG_PQ_V=chainrow` | 436 | 154 | 41 / 47 / 51 | 11 / 23 / 8 |
  | `CG_PQ=4096:floor CG_PQ_V=short` | 555 | 118 | 46 / 55 / 65 | 33 / 50 / 72 |
  | `CG_PADD=0.5 CG_PQ_V=chain` | 527 | 128 | 25 / 37 / 42 | 46 / 53 / 93 |

  - `chainrow`: rows holding an opening-chain vertex.
  - `short`: edges that cross no row boundary.
  - `chain`: pieces at an opening-chain vertex.

  Each trades triangles against dense outlines. These made things worse:
  chaining edges through vertices, exactly or floored; exact ends at chain
  vertices; and snapping vertex y.

- **More on the vertex residual (18:15Z).**
  - Joining an opening edge's first point to the previous edge's stepped
    end (`CG_JOIN=open`) fixes the one-vertex collinear sweep (8 to 0) and
    is neutral elsewhere (suite 981).
  - The rest is on opening edges in rows a vertex splits into bands,
    including vertices on the other side of the shape (`v2rs.py`). There CG
    covers about 0.03 of a level more.
  - No band-point position explains it (`obsweep.py`). Matching offsets
    range over +1 to −3 units and don't follow the fraction; "floored
    minus 2" fits 167 of 204 cases.
  - These also didn't help: side-dependent slope rounding, and coarser x
    on opening pieces (`CG_XQO`).

## Remaining before exact fills

- The vertex-row residual above. It matters most for curves.
- Stroke outlines: reproduce `copy(strokingWithWidth:)` (curve offsetting,
  joins, caps, dashes) against the API's own output, and the thin-stroke
  rasterizer.
- Gradients and images in PDFs, and CoreSVG for SVGs.
- A Rust implementation in actool, checked against `cov` and then against
  Xcode on the corpus's 46 PDF-only catalogs.

## Tools

- `cov.swift`: fills paths read from stdin into an alpha-only context and
  prints alpha.
- `render.swift`, `raw.swift`: draw a PDF page as Xcode does, to a PNG or
  as raw pixels.
- `mkpdf.py`: writes a one-page PDF.
- `compile.sh`, `bitmap.sh`, `check.sh`, `payloads.py`: compare Xcode's
  vector renditions with CoreGraphics' drawing.
- `raster.py`: the current model of the general fill (`fill_path`), and
  `flat.py`: curve flattening.
- `glyphs.swift`, `pathcmp.py`: glyph outlines as paths, compared with the
  model. `covs.swift`: strokes. `covf.swift`, `cova.swift`: float, 16-bit and
  partial-alpha probes.
- `pdfpaths.swift`, `pdffills.py`: corpus PDFs' fills against the model.
  `covc.swift`: clips. `strokeq.swift`, `outline.swift`: strokes against
  their outlines, and the outlines themselves.
- `ell.py`, `curves.py`, `multi.py`, `wedge2.py`, `genshallow.py`,
  `bias.py`, `vprobe.py`, `perturb.py`: probes for ellipses, curves,
  overlapping subpaths, wedges, shallow edges, the vertex-row bias, vertex
  sweeps, and unit perturbations.
- `trieval.py`, `wedge.py`, `gen20.py`, `signed.py`, `fastfit.py`,
  `anchor3.py`, `rowfit.py`, `drift3.py`: probes and scorers for triangles,
  wedges, long edges and drift. They import each other from `/tmp/cg`, with
  `cov` built there from `cov.swift`.
- `model.py`, `scan.py`, `area2.py`, `gray.py`, `tri.py`: earlier coverage
  models, superseded by `raster.py`.
