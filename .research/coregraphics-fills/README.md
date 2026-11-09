# CoreGraphics fills (research, tabled 2026-11-09)

Can this actool draw the bitmaps Xcode makes from PDF vector images byte for
byte, instead of approximately (`--config=relax_actool_vector`)? Of the
corpus's 63 catalogs that compile only relaxed for their vector images, 46
hold only PDFs. This is a time-boxed probe, black-box throughout: CoreGraphics'
public API from Swift on the Mac. It is not for merging, and the actool is
unchanged.

Status: tabled at Logan's three-hour mark and tracked as outstanding. The
Metal project comes first.

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
- **Other paths get analytic area coverage.** Alpha is `floor(256 · area)`,
  capped at 255. An edge's vertical extent is quantized to the 1/256 grid:
  its lower end floors, and its upper end floors from just below itself, so
  a full pixel gives 255. Very small coverage gives 0. Exact area matches a
  45° sweep completely, 99.3% of 2000 random half-planes
  (`halfplanes.json`), and about 95% of triangle edge pixels (`tri.py`,
  `area2.py`).
- **Curves are flattened uniformly** into a power-of-two number of segments
  by size (8, 16 and 32 for curves of about 4, 14 and 57 pixels).

## What blocks exact fills

A residual error of up to about ±0.05 of an alpha level in the general
path, which flips about 1% of edge pixels (more where vertices lie inside
pixels). It is some internal arithmetic not yet identified. These were
tested and don't reproduce it:

- vertex quantization from 1/256 to 1/65536 (floor, round);
- quantizing where edges cross pixel sides;
- float32 arithmetic;
- rounding per 1/256 band;
- fixed-point stepping along edges (DDA);
- a FreeType-style cell rasterizer (`gray.py`).

An icon has hundreds of edge pixels, so even 1% means no catalog would
compile byte for byte. Also not yet examined: strokes (joins, caps, dashes),
clipping, and CoreSVG for SVGs.

## Tools

- `cov.swift`: fills paths read from stdin into an alpha-only context and
  prints alpha.
- `render.swift`, `raw.swift`: draw a PDF page as Xcode does, to a PNG or
  as raw pixels.
- `mkpdf.py`: writes a one-page PDF.
- `compile.sh`, `bitmap.sh`, `check.sh`, `payloads.py`: compare Xcode's
  vector renditions with CoreGraphics' drawing.
- `model.py`, `scan.py`, `area2.py`, `gray.py`, `tri.py`: the coverage
  models tested.
