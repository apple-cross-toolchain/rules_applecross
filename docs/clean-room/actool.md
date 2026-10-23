# actool

Reproduced: `actool --compile DIR --platform iphoneos
--minimum-deployment-target V --app-icon NAME --target-device
iphone|ipad --output-partial-info-plist P CATALOG` for a catalog holding one
`.appiconset` with a single universal 1024x1024 iOS image.

Also reproduced, for every catalog in the examples except tvOS's and
visionOS's: `--platform iphonesimulator`, `watchsimulator` and `macosx`,
app icon sets with an image per size (below), and catalogs that compile to
nothing. visionOS's and tvOS's (`xrsimulator`, `appletvsimulator`: image
stacks, below) are byte-identical too (tvOS's radiosity image but for
rare rounding, below).

Outputs, all byte-identical to Xcode 27.0's for `examples/ios/CleanRoom`:
`NAME60x60@2x.png` (iPhone), `NAME76x76@2x~ipad.png` (iPad), `Assets.car`
and the partial Info.plist (`CFBundleIcons`, `CFBundleIcons~ipad`). Xcode's
output is deterministic: repeated runs are identical, and the timestamp
`assetutil --info` prints is the file's modification time, not content.

## Resampling

Observed: the 120 and 152 px icons are downscaled from the 1024 px image.
Step-edge and impulse images (black, grey and coloured, every amplitude
1-255) measured the filter: Lanczos-2, sampling output pixel *i* at
`c = i·(S−1)/(D−1)` with the kernel stretched by the same factor, separable,
horizontal pass first, through an 8-bit intermediate, with 14-bit
fixed-point weights.

Established through a public API: drawing the source with
`CGContextDrawImage` at `kCGInterpolationHigh` reproduces actool's pixels
exactly, while vImage's, ImageIO's thumbnails and Core Image's Lanczos do
not. Drawing impulses into a 32-bit float context then gave the weights to
nine digits: the kernel's value at every tap except the one nearest `c`,
which weighs 1; normalized over the taps (taps beyond an edge count for the
edge pixel); the rounding residual of the 14-bit weights on that nearest tap.
This reproduces both icons with no differing sample.

## CgBI PNGs

Observed chunk sequence: `CgBI` (0x50002006), `IHDR` (8-bit RGBA),
`gAMA` 45455, `sRGB` 0, `cHRM` (sRGB primaries), `eXIf` (ColorSpace 1 and
the pixel dimensions), `iDOT`, two `IDAT`, `IEND`. Pixels are premultiplied
BGRA, every row filter 0. The two IDATs are one raw deflate stream with a
full flush between halves; searching zlib's parameters against five icons
leaves one setting: level 9, window 15, memLevel 9, `Z_FILTERED`. `iDOT`
records the halves' row counts and their IDAT offsets.

## Simulator PNGs

For `iphonesimulator` the bundle icons are plain PNGs of the same pixels:
`IHDR` (8-bit RGBA, straight alpha), `sRGB` 0, the same `eXIf`, `IDAT`
chunks of 16 KiB, `IEND`. Rows are filtered as libpng's adaptive heuristic
chooses (least sum of absolute signed values; the lower filter on a tie),
matching every row of four icons; searching zlib's parameters against them
leaves level 1, window 15, memLevel 8. When an icon set holds an image of
exactly the bundle icon's size, Xcode copies that file byte for byte (a
re-encoded source is copied as given).

## Catalogs that compile to nothing

An app icon set whose entries name no files, and a colour set with no colour
components, compile to nothing: no `Assets.car`, and an empty dictionary as
the partial Info.plist, with or without `--app-icon`
(`examples/ios/HelloWorldSwiftUI`).

## Assets.car

A BOM store (big-endian): 512-byte header area, blocks 16-byte aligned in
allocation order, the variables table, then the index (at least 256 entries,
else the block count plus one rounded up to 8) and an empty free list.
Variables, in order: `CARHEADER`, `RENDITIONS`, `FACETKEYS`,
`APPEARANCEKEYS`, `KEYFORMAT`, `EXTENDED_METADATA`, `BITMAPKEYS`.

Trees are a 29-byte header (`tree`, version 1, page block, block size, entry
count, an inline-keys flag, the longest key) and one leaf page: entries
(value block, key block), four zero bytes, then a copy of every key; the
page's length is the block size plus the keys' lengths. When the keys
differ in length the header records 0xffffffff and the page holds no
copies (FACETKEYS with names of different lengths). `BITMAPKEYS` stores
its keys (name identifiers) inline in a 1024-byte page; their value blocks
are allocated in the order a CoreFoundation hash table enumerates the
identifiers (the integer CFNumber hash: the value times 2654435761, 64
bits) when filled with any image stacks' identifiers first and then the
rest, largest first, an atlas's 0 among them. That reproduces all 13
catalogs examined with more than one identifier.

For an app icon: per idiom, an Icon Image rendition (part 220, dimension2
1: the 1024 px image as LZFSE-compressed premultiplied BGRA, named after
the source file) and a MultiSized Image rendition (part 218: a `SISM` list
of the sizes). Renditions are keyed by
(appearance, localization, scale, idiom, subtype, dimension2, identifier,
element 85, part); the tree lists them by key, sorted as bytes (16-bit
little-endian values, so identifier 7412, `f4 1c`, comes after 51442,
`f2 c8`). Their blocks (key, then value) are allocated in preorder of a
balanced tree of the keys sorted by value: the
middle one (the upper of an even number), then the lower half, then the
upper, each the same way. Found by compiling sets of images whose names had
consecutive identifiers: 2000-2008 come out 2004, 2002, 2001, 2000, 2003,
2007, 2006, 2005, 2008; it reproduces every icon catalog in the examples.
Image payloads are an `MLEC` header and `KCBC` chunks of up to a third of
4 MiB of rows, each an LZFSE stream (lzfse.md).

The name identifier: observed for 2,300 names, a 64-bit djb2 of the UTF-8
name with seed 100823, folded by evaluating its four 16-bit limbs, low first,
as a base-33 polynomial, modulo 2^16. Names that collide are renumbered (the
one later in byte order); that renumbering is not reproduced and is refused.

Versions recorded in the car are Xcode 27.0's (CoreUI 1010, storage 17,
CoreThemeDefinition 664; CMPhoto 219 for iOS, 218 for watchOS and macOS);
the Xcode name and build come from the SDK's `version.plist`.

Image payloads under 16 KiB (stride times height) are one LZFSE stream
(`MLEC` version 2, method 4, the stream's length); larger ones are `KCBC`
chunks: as many as the bitmap has 8 KiB, at most three, of a whole number
of rows each, the remainder in a fourth (`MLEC` version 3; 1 in macOS
catalogs). Rows are 32-byte aligned (`0x3ef`). Measured over 24 atlas and
image sizes.

## Icon Composer documents

rules_apple passes an `.icon` document as an input beside any catalogs
(`actool ... --app-icon Visor CATALOG... Visor.icon`). For Visor's document
(a solid fill, one group of three SVG layers with a neutral shadow and
translucency), Xcode 27.0 writes the same bundle PNGs and partial Info.plist
as for an icon set, and an Assets.car holding, besides Icon Image and
MultiSized Image renditions for phone and pad, the document's colours, a
named gradient, its SVGs as Vector renditions, IconGroup and IconImageStack
renditions per appearance (light, dark, tintable), and flattened 1024 px
Icon Images for the Any, Dark and Tintable appearances. Two runs of Xcode's
actool on the same document differ: the Icon Image renditions are named
with a fresh UUID each time
(`Visor1024x1024_UIAppearanceAny_<UUID>-<pid>-<time>.png`).

Not reproducible, then, and this actool approximates: it renders the
document's default appearance flat (the fill, then each group's layers back
to front with a blurred, downward-offset shadow of the group's opacity, no
glass, specular or translucency; SVG layers through resvg, PNG layers
scaled about the centre) and compiles that image exactly as an icon set's
1024 px image, named `NAME1024x1024.png`. The result has the renditions an
icon set gives, which iOS before 26 draws icons from; how iOS 26 draws it
(presumably as an icon set's, without glass) has not been checked. Xcode's flattened image is square and opaque, as this
one is.

## App icon sets with an image per size

Each image is a rendition (part 220) of its scale, idiom, subtype and
`dimension2`, a size's rank among all the set's point sizes; per idiom and
subtype a MultiSized Image rendition lists its sizes (in whole points, 83.5
as 83) with those ranks. An iPhone 60pt@3x image is also written as 90pt@2x
for subtype 1792 (the derived size counts in the ranking). Watch icons
take their size's subtype (38mm 320, 42mm 384) as notification-centre and
quick-look icons; the app launcher's and companion settings' are subtype 0.
In BITMAPKEYS a subtype is a bit from a list (0: 0, 320: 8, 384: 9, 1792:
16); size and layer are tracked too.

For the simulators the bundle's 60pt@2x and 76pt@2x icons are the set's
files, copied. A watch app's partial Info.plist names the icon only. A
macOS catalog has no appearance table, key semantics 1, the key format
(appearance, localization, element, part, size, identifier, dimension2,
layer, scale) and no idiom or subtype; `NAME.icns` holds the 16pt and
128pt images (`ic13`, `ic11`, `ic04`, `ic07`, in that order): PNGs as
ImageIO writes them (as for simulators, but EXIF without ColorSpace, and
libpng's smaller window for data of 16 KiB or less) and, for `ic04`, ARGB
planes run-length coded greedily (runs of 3 to 130).

## Packed images

Small images are packed into atlases
(`ZZZZPackedAsset-<scale>.<1, or 0 for macOS>.0-gamut0`, element 9, part
181, layout 1004); each image is then a rendition of layout 1003 with a
`KLNI` TLV (0x3f2): its x, y, width and height in the atlas, then `0c00`, a
32-bit length, and the atlas's key as (attribute, value) pairs of 16 bits in
attribute order, ending with four zero bytes. Measured over 70 random sets
and every icon set in the examples:

- Per (scale, idiom, subtype), images are taken largest area first, equal
  areas by name identifier. One of more than 18,050 px² × scale is not
  packed; neither is the only packable image of a group.
- Atlases fill next-fit: an atlas takes images while the area it already
  holds is below the cap, then the next atlas (`dimension1` 1, 2, ...)
  begins; the key format then includes `dimension1`.
- Within an atlas, Jake Gordon's growing binary-tree packer, with blocks of
  (w + 2, h + 2) placed 2 px in. Each side of the atlas is the root's, plus
  one, rounded up to even.
- The atlas's rows are stored bottom-up (the rectangles' y counts from the
  top); each image is surrounded by a copy of its edge pixels one pixel
  wide, the rest transparent.

## dmp2

Image sets, and tvOS's and visionOS's image layers, are stored as `MLEC`
(version 2 when opaque, otherwise 0; method 11) wrapping `01 00 00 00`,
`04 00 00 00`, a length and a zero word, then a `dmp2` stream: `dmp2`, a
mode, `01 0a 04`, the width and the rows per strip (16 bits each), and by
mode the strips' data. Image sets and a stack's back layer are opaque when
every pixel is; a stack's other layers when their PNG has no alpha (no
colour type with it, no `tRNS`), whatever their pixels. Modes 2 and 3
compress strips of as many rows as fit 2 MiB (their data per row, rounded
up to 16 bytes), each on its own: a 1024 px image is four strips in mode
2 (291 rows each, the last 151). Each mode-2 strip is transformed as an
image of its own. Modes:

- 1: the premultiplied BGRA pixels as they are;
- 3: a 32-bit length and those pixels as the reference LZFSE encoder
  compresses them (`lzfse_encode_buffer`, LZVN for small inputs);
- 4: a palette (count, at most 256, and entry size 4, 16 bits each;
  premultiplied BGRA entries), then a 32-bit length and the 8-bit indices
  compressed the same way;
- 2: a 32-bit length and, compressed the same way, a lossy predictive
  transform padded with zeros to a multiple of 16 bytes: the alpha plane
  (one byte per pixel), one predictor code per row, then the high bytes
  and then the low bytes of three 16-bit values per pixel.

Xcode writes whichever mode is smallest (modes 2 and 3 always; 4 when
there are at most 256 colours; 1 never smaller in practice).

The palette starts in ascending order of the 32-bit little-endian entries
and is then sorted with macOS's `qsort` (FreeBSD's, whose order among
equal keys is part of the output: insertion sort up to 7 elements, a
median-of-three or ninther pivot, a bounded insertion fallback when a pass
swaps nothing, heapsort past a depth of 2 log2 n) by the XOR of each
entry's four bytes. Both example palettes and every probe agree.

Mode 2 converts each premultiplied pixel to YCoCg with truncating (C)
division, losing a bit of G and B: `Co = (B - R) / 2`, `t = R + Co`,
`Cg = (G - t) / 2`, `Y = t + Cg`. Each row picks a predictor, applied to
all three values: 0 zero, 1 left or up (left when |U - UL| <= |L - UL| in
Y), 2 left, 3 up, 4 `(L + U + 1) / 2`. At x = 0 the prediction is the
pixel above (0 for codes 0 and 2, and in row 0); in row 0 it is the pixel
to the left. Residuals are sign-magnitude, `2|r| + (r < 0)`. Row 0 weighs
codes 0 and 2, later rows all five, choosing the smallest sum of
log2(1 + |r|) over the row's residuals, ties going to 0, 2, 3, 1, 4 in
that order. That matches 6,667 of 6,670 probe rows; the other three are
ties broken the other way, two between different residuals with equal
integer costs and one between identical residuals, so the tie rule is not
fully established.

Pixels are premultiplied as `(c * a + 127) / 255`. A palette that would
need strips (over 2 MiB of indices) is not established and is refused. Every layer in the
examples uses mode 4. Where a row's predictor isn't established and mode 2
could be the smallest, the image is refused; so are images that would be
stored uncompressed (mode 1, never seen) and ties between modes.

## Image stacks

tvOS brand assets' primary app icon (an `.imagestack` of
`.imagestacklayer`s, one 1x tv image each, on a 400x240 stack) and
visionOS's `.solidimagestack` (`.solidimagestacklayer`s, one 1024 px
reality image at 2x each). The partial Info.plist's
`CFBundleIcons.CFBundlePrimaryIcon` is the stack's name, a string.

Facets: the stack (element 85, part 181, its identifier) and each layer as
`Stack/Layer/Content` (tvOS: element 85, part 181; visionOS: element 41,
part 181 and idiom 8), sorted by name. Key format: tvOS's is iOS's without
dimension2; visionOS's has dimension1 instead. `CARHEADER`'s schema version
and key semantics are 2 and 2 for tvOS, 1 and 4 for visionOS; platforms
`atv` and `vision`, CMPhoto 218.

Renditions:

- the stack (scale 1, no idiom, part 181, `ATAD`, layout 1002 for tvOS and
  1018 for visionOS, the stack's size in points, named `Contents.json`,
  payload `DWAR` and 8 zero bytes). TLV `0x3f4` lists the layers back to
  front: a count and a zero word, then per layer a zero word, x, y, width
  and height in points (layers centred), a zero word, opacity 1.0, 16 and
  its image's key ((attribute, value) element 85, part 181, identifier,
  then 4 zero bytes). `0x3fc`, `0x3fd` and `0x3fe` are a count and a zero
  word, then per layer 8 zero bytes, `01 00 00 00` and a zero byte; 20
  zero bytes; and 12.
  `0x3ed` is the UTI `public.layeredimage` (its length with the NUL, a
  zero word, the string);
- each layer's image (layout 12, flags 16, `dmp2`, named after its file);
- the flattened composite (part 208, layout 0,
  `ZZZZFlattenedImage-{scale}.1.0-gamut0`, LZFSE chunks, `MLEC` version
  3): the layers composited back to front, source over in premultiplied
  8-bit with `(d * (255 - a) + 127) / 255`;
- tvOS: the radiosity image (part 209, layout 0,
  `ZZZZRadiosityImage-1.0.0`, 480x320, `MLEC` version 0 method 6: width,
  height, 32, the float 0.7255511, then 15x10 cells of 16-bit signed
  BGRA);
- visionOS: per layer two textures, dimension1 1 and 2, each a descriptor
  (element 41, no part, layout 1007, named after the layer's facet:
  `RTXT`, 0, a Metal pixel format, 1, 1, 1 and 1 (16-bit), the key's
  length and a zero word, then the key element 41, part 181, dimension1,
  scale, identifier, idiom, in that order, and 4 zero bytes) and its data
  (part 181, layout 1008, colour space 0, LZFSE chunks, `MLEC` version 1).
  Level 1 is R32Float (55): each pixel centre's distance to the layer's
  edge, for an opaque layer. Level 2 is RG8Snorm (32): a direction towards
  the nearest edge.

The direction texture of an opaque 1024 px layer is the same for every
such layer, whatever its pixels. It is the unit directions to the nearest
edges (45 degrees where two are equally near; each component zero within
2 px of the top and left edges and 1 px of the bottom and right edges
across it) smoothed by a nearly Gaussian kernel (sigma about 2). Fitted
kernels leave a few thousand values one unit off, so actool rebuilds it
from tables measured from Xcode's output instead: the ramp across an edge
(7 values, a pixel narrower at the bottom and right), the profile across
a diagonal (19 values), and the 233 pixels at the corners and the centre
where they don't compose (using the texture's symmetries). That
reproduces it byte for byte.

The radiosity image: on a 480x320 canvas, the composite's lower two
thirds (rows from a third of its height) 40 px in from the left and the
top, each premultiplied channel blurred across and then down by a Gaussian
(`exp(-d^2 / 800)` for d from -40 to 40, as weights of `16384 * g / sum`
rounded down, zero beyond the canvas, each pass `(sum + 8192) >> 14` and
at most 255: what `vImageConvolve_Planar8` does with a divisor of 16384).
Then each 32 px cell is 4x4 sums of its 8 px blocks weighed -1331, 1843,
1843, -1331 (1/1024: the half floats of -1.3 and 1.8) each way, divided
by 2^20 and truncated. Xcode 27.0 on an M4 Max and 27.1 on an M6 write
the same radiosity image, so it doesn't depend on the GPU. That is exact
but for Xcode's arithmetic, which is not established: a flat cell (16
equal blocks, of a level u) comes out 64u - 1 for 36 of the 255 levels
(a set closed under doubling, so it depends on the level's mantissa;
actool has the measured set), and otherwise about one value in 1,700, one
within single-precision rounding of the terms' magnitudes of a whole
number, comes out one more or one less. Single- and half-precision sums
over blocks or pixels in the usual orders, with or without fused
multiply-adds, reproduce neither. The examples' icon is byte-identical; of
20 random three-layer stacks 13 were, the others one value off.
Grey layers (Xcode keeps them GA8), translucent visionOS layers,
other brand asset roles and sizes, and layers small enough to pack into an
atlas are refused.
