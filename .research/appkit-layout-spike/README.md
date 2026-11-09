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
