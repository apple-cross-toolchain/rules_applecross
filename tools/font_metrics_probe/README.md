# font_metrics_probe

`generate.swift` measures the Mac's system fonts as AppKit does
(`NSString.size(withAttributes:)`) and writes the tables ibtool lays out
content-sized AppKit controls with (`tools/apple_tools/crates/ibtool/src/font_metrics.bin`,
read by `font_metrics.rs`): each character's width and each adjacent pair's
adjustment (its kerning) for the system font and its bold at 9 to 36 points
and the menu font, and each font's line height. A string's width is their
sum, as AppKit's (measured for every character in the tables).

The data comes from the fonts macOS gives every app (the system font, SF),
measured through AppKit; nothing is decompiled. To regenerate, on a Mac (a
leased VM) with the Xcode the corpus is compared against:

```sh
xcrun swift generate.swift font_metrics.bin
```

and copy `font_metrics.bin` over the crate's.
