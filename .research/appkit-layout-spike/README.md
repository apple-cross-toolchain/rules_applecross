# AppKit layout spike (2026-11-09)

A time-boxed prototype: can a constraint solver plus the Mac's control and
text metrics reproduce the frames Xcode's ibtool lays out for constrained
AppKit documents? Not for merging; the compiler is unchanged.

- `solve.py DOC`: solves each view's constraints with kiwisolver (Cassowary)
  over alignment rects in top-down coordinates, with intrinsic sizes as
  hugging and compression-resistance inequalities at their priorities,
  views translating their autoresizing masks fixed, weak stays at the
  document's values, and frames snapped to whole points. Intrinsic sizes
  from probes of Xcode 27.2 (`ctl.py`): a push button `ceil(text) + 24` by
  24 (small: `+ 20` by 20), a label `ceil(text) + 4` wide by its line
  height and 2 points out on each side, a checkbox `+ 22` by 16, a pop-up
  `+ 48` by 24, an editable field 24 high. Text widths come from AppKit on
  the Mac (`tm.swift`), which Linux would need reproduced.
- `eval.py`: the prototype's frames against Xcode's for the corpus
  documents strict mode refuses only for layout.

Result: 21 of the 45 such documents laid out exactly as Xcode does; 9
wrong in a view or two (control metrics not yet probed), 15 blocked
(image views 8, wrapping labels 2, baselines 2, a search field, a progress
indicator, a light font).

## Status: tabled (2026-11-09)

Tabled at the estimate below, unmerged, and tracked as outstanding. Strict
mode keeps refusing these documents, and `--config=relax_ibtool_layout`
keeps compiling them with their own frames.

What blocks shipping it, about 2 to 3 weeks in all:

1. **Text measurement on Linux** (tabled with this spike). Intrinsic widths
   are CoreText's measure of the system font, rounded up. Linux needs the
   font, likely copied from the Mac into the SDK archive by
   `tools/package-sdks.sh` as the colour profiles are, and a reimplementation
   matching CoreText's advances, kerning, tracking and optical size closely
   enough. 3 to 5 days, uncertain.
2. **A Rust Cassowary solver** that resolves ambiguous layouts as AppKit's
   engine does (stays at the document's frames). 1 to 2 days.
3. **The remaining controls' metrics**, probed from Xcode: sliders and other
   kinds, small and mini sizes, text field variants, search fields,
   progress indicators. 2 to 3 days.
4. **Image views**, whose intrinsic size is their image's: the asset
   catalog's image sizes passed to ibtool by the rules. About 2 days.
5. **Wrapping labels and baseline constraints**: line breaking and font
   ascent metrics. Riskiest.

With 1 to 3, about 30 of the 45 documents should match strictly.
