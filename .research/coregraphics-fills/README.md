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
- **Rectangles take a fast path, solved.** A path that is a rectangle gets
  coverage per axis: 8 bits, `(w − 1) >> 5`, where `w` is measured in
  1/8192 of a pixel from the start edge snapped to its 1/256 step
  (`floor((s + 1)/32)`). The end edge is exact. Alpha is
  `floor(ax · ay / 255)`. This matches 400 of 400 random rectangles and every
  strip (`span_table.json`, `rects.json`).
- **Other paths: fixed-point DDAs per edge and a band sweep.** Found by
  black-box probing with `cov` (`raster.py`, `fill_path`, is the model):
  1. Path coordinates go through float32, then round half up to 1/4096 of a
     pixel. The path is clipped to the canvas.
  2. Each edge is stepped from its lower endpoint by two independent DDAs.
     Row crossings (x at each integer y) step by dx/dy truncated to 1/4096.
     Column crossings (y at each integer x) step by dy/dx truncated to
     1/4096. The first crossing is the vertex plus
     `floor4096(frac · slope)`. The edge ends where the row stepper reaches
     its top y, not at the vertex.
  3. Each edge also gets a point at every vertex y of the path inside its
     span. Its x is stepped from the previous row crossing, with the same
     floor.
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
- **Corpus strokes.** CG's outline filled by the model gets 6588 of 6884
  stroke edge pixels right (`pdfstrokes.py`). Straight strokes are exact;
  the misses are on curves.
- **Thin strokes.** A thin axis-aligned stroke follows the rectangle fast
  path exactly. A thin diagonal one is within ±1 of the 1 px outline's fill
  along its length, but its ends differ (`thin.swift`).

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
