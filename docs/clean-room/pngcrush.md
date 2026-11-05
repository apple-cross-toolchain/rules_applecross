# pngcrush

rules_apple copies an iOS, tvOS or watchOS bundle's PNG resources with
`xcrun pngcrush -q -iphone -f 0 -rem text IN OUT` (Xcode's copypng):
Xcode's own pngcrush (1.6.4, with libpng 1.2.7 and zlib 1.2.12) writes an
Apple-optimized ("CgBI") PNG. Off a Mac, `tools/apple_tools/crates/pngcrush`
does, byte for byte, for 8-bit RGB and RGBA PNGs that aren't interlaced,
which are 2,068 of the 2,138 PNGs of a corpus of 3,000 files on a Mac (the
rest palette, grey, 16-bit or interlaced). It accepts exactly that command
line. Other PNGs are refused, or copied as they are with
`--config=relax_pngcrush`. Before, the ported tools carried stock pngcrush
1.8.13, which has no `-iphone` and wrote an ordinary PNG, with its alpha
dropped when the image had none to use, and no error.

`tests/pngcrush` compares it with Xcode's pngcrush; all of the corpus's
RGB and RGBA PNGs matched, built with Cargo's zlib and Bazel's (1.3.2).

## The file

- **Chunks:** a `CgBI` chunk first, `0x50002006` when every pixel is opaque
  (an RGB image, or RGBA with alpha 255 throughout) and `0x50002002`
  otherwise (250 images). Then `IHDR`, with colour type 6 (RGBA) whatever
  the input's.
- **Pixels:** premultiplied BGRA, each colour `(c × a + 127) / 255` (all
  65,536 colour and alpha pairs), alpha kept; every row unfiltered (filter
  type 0).
- **Compression:** raw deflate (no zlib header or checksum), zlib's level 9,
  memLevel 9, `Z_FILTERED`, one stream.
- **Parallel decoding:** an image of 4,096 pixels or more and at least two
  rows has an `iDOT` chunk before the image data, and the stream is split in
  two, a full flush ending the first part: one `IDAT` chunk each. The first
  part takes the odd row. `iDOT` is seven big-endian 32-bit values: 2, then
  each part's first row, row count and offset from the `iDOT` chunk's start
  to its `IDAT` chunk's (40 for the first). Smaller images, and a single
  row, are one `IDAT` (14 probes around the bounds).
- **Ancillary chunks,** in libpng's order: `gAMA`, `sRGB`, `iCCP`, `sBIT`,
  `cHRM`, then `eXIf` (unknown to that libpng, but safe to copy), then
  `bKGD`, `pHYs`, `tIME`.
  - An `sRGB` chunk brings sRGB's own `gAMA` (45455) and `cHRM` (the
    D65 white point and the Rec. 709 primaries), in place of any given.
  - An `iCCP` profile is deflated again, raw, as the image data is.
  - Text (`tEXt`, `zTXt`, `iTXt`), `cICP` and an input's `iDOT` are dropped:
    `-rem text`, and chunks that libpng doesn't know and that aren't safe to copy.
- **Refused:** other colour types and bit depths, interlacing; chunks the
  corpus didn't have (`tRNS`, `PLTE` in an RGB image, `sPLT`, `oFFs`,
  among others); a kind given twice; anything but text after the image
  data; an RGB image's `sBIT`; and chromaticities or a profile libpng would
  ignore (a white point and primaries whose x and y exceed 1 together, a
  profile whose header gives another length).
