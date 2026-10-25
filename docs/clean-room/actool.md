# actool

Reproduced: `actool --compile DIR --platform iphoneos
--minimum-deployment-target V --app-icon NAME --target-device
iphone|ipad --output-partial-info-plist P CATALOG` for a catalog holding one
`.appiconset` with a single universal 1024x1024 iOS image.

Also reproduced, for every catalog in the examples except tvOS's and
visionOS's: `--platform iphonesimulator`, `watchsimulator` and `macosx`,
app icon sets with an image per size (below), iOS image sets of PNG images
(below), and catalogs that compile to nothing. visionOS's (`xrsimulator`: image stacks, below) is byte-identical
too. tvOS's (`appletvsimulator`) is refused when strict, the default: its
radiosity image can't be guaranteed to be Xcode's (below). Relaxed
(`--config=relaxed`), the output is byte-identical to Xcode's.

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
which weighs 1; normalized over the taps; each rounded to 14 bits, and then
a tap beyond an edge added to the edge pixel's; the rounding residual of
the 14-bit weights on that nearest tap. This reproduces both icons with no
differing sample. Merging the taps beyond an edge before rounding instead
differs for noisy images near the edges (up to 12 samples, rows and
columns 1 and 118 of the 120 px icon; 25 random images through the public
API agree with rounding first).

## CgBI PNGs

A device build's bundle icons are CgBI PNGs with `--compress-pngs` (which
rules_apple always passes); without it they are the simulator's PNGs
(below), byte for byte, and Assets.car is the same either way. Observed
chunk sequence: `CgBI` (0x50002006), `IHDR` (8-bit RGBA),
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
bits) when filled with the identifiers in the order their renditions are
first allocated (atlases, which have no identifier, not among them). That
reproduces every catalog examined, including image set catalogs whose
identifiers collide in a three-entry table, which an atlas's 0 would have
grown.

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

Not reproducible, then. Strict (the default), this actool refuses Icon
Composer documents. Relaxed (`--config=relaxed`), it approximates: it renders the
document's default appearance flat (the fill, then each group's layers back
to front with a blurred, downward-offset shadow of the group's opacity, no
glass, specular or translucency; SVG layers through resvg, PNG layers
scaled about the centre) and compiles that image exactly as an icon set's
1024 px image, named `NAME1024x1024.png`. For macOS (`--target-device
mac`) it compiles that image, scaled to each size, as a macOS icon set with
an image per size (`icon_16x16.png` to `icon_512x512@2x.png`): Assets.car,
`NAME.icns` and a partial Info.plist naming both, `CFBundleIconFile` and
`CFBundleIconName`, which are the outputs and keys Xcode writes for the
document, its `.icns` with the same four entries. The result has the renditions an
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
files, copied. In the partial Info.plist each target device's entry
(`CFBundleIcons`, `CFBundleIcons~ipad`) names the icon and lists the
bundle icons it has: iPad's lists the iPhone one first when both are
targets and only when it has its own; a set without one lists none for
that device. For iPad alone an App Store (`ios-marketing`) image also
gives `CFBundleIcons` with the name only. The App Store's 1024pt image is
idiom 6 with a MultiSized Image of its own, ranked among the sizes;
legacy slots (iPhone 57pt at 1x and 2x, iPad 50pt and 72pt at 1x and 2x)
are compiled as any other (20 probes, missing and legacy slots, App Store
images, one device or both). A single-size set's bundle icons are both
written whatever the target devices, each device's entry listing them as
for both and naming the icon only for a target; building for iOS, a set's
`mac` images are ignored (mypod's), and a set of only those compiles to
nothing (no Assets.car, an empty partial Info.plist; Flutter's macOS
template). With no image for any target device, Xcode compiles another
device's (an iPad set for iPhone alone); that is refused. A grey icon (every
pixel R = G = B, whatever the PNG's colour type) is stored as GA8, as
image sets' are, in atlases named `S.O.1`; this actool refuses those for
icons. A watch app's partial Info.plist names the icon only. A
macOS catalog has no appearance table, key semantics 1, the key format
(appearance, localization, element, part, size, identifier, dimension2,
layer, scale) and no idiom or subtype; `NAME.icns` holds the 16pt and
128pt images (`ic13`, `ic11`, `ic04`, `ic07`, in that order): PNGs as
ImageIO writes them (as for simulators, but EXIF without ColorSpace, and
libpng's smaller window for data of 16 KiB or less) and, for `ic04`, ARGB
planes run-length coded greedily (runs of 3 to 130), each from the image
premultiplied and back, rounding to nearest both ways (a fully transparent
pixel's colour is lost: Flutter's macOS icons, every pixel of all four).

An iOS icon's `MLEC` version says whether it is opaque: 2 (one stream) or
3 (chunks) when it is, 0 or 1 when it isn't, per standalone image and per
atlas, whose name's middle number is 1 when opaque and 0 when not
(Google Maps' and Firebase's example icon sets, and probes at 20 to 167
pixels). An atlas of opaque and translucent icons, which those don't have,
is refused. An image under 40 pixels wide is stored with method 1, a
run-length coding (per-row offsets, then runs), opaque or not; this actool
refuses those. Xcode ignores entries that aren't iOS icon slots (Flutter's
iPhone 40 and 60 pt at 1x, iPad 76 pt at 3x); this actool compiles the
current and legacy slots (iPhone 20, 40 and 60 pt at 2x and 3x, 29 pt at
1x to 3x, 57 pt at 1x and 2x; iPad 20, 29, 40, 50, 72 and 76 pt at 1x and
2x, 83.5 pt at 2x) and the App Store's 1024 pt, and refuses the rest.

## Packed images

Small images are packed into atlases
(`ZZZZPackedAsset-<scale>.<1, or 0 for macOS>.0-gamut0`, element 9, part
181, layout 1004); each image is then a rendition of layout 1003 with a
`KLNI` TLV (0x3f2): its x, y, width and height in the atlas, then `0c00`, a
32-bit length, and the atlas's key as (attribute, value) pairs of 16 bits in
attribute order, ending with four zero bytes. Measured over 70 random sets
and every icon set in the examples:

- BITMAPKEYS' blocks follow a CoreFoundation table of the identifiers,
  filled in the order their renditions were first allocated (linear
  probing, as CFBasicHash.c publishes). That reproduces 547 of 570 catalogs
  examined; all 23 others have atlases and identifiers that share a
  bucket, where Xcode's order differs in a way not established (rendition
  key, facet, name and identifier orders, NSMutableSet growth, pre-sized
  tables and hidden atlas entries all fit fewer). Catalogs with atlases
  and colliding identifiers are refused.
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
- 3: a 32-bit length and those pixels compressed: from 4096 bytes on as the
  reference LZFSE encoder compresses them (`lzfse_encode_buffer`), and below
  that as raw LZVN (libcompression's `COMPRESSION_LZVN`, the reference
  `lzvn_encode_buffer`), even where that is larger than its input, which
  LZFSE would store uncompressed instead (48 palette indices as 52 bytes;
  a single-colour 100x100 image's indices are LZFSE although raw LZVN would
  be smaller);
- 4: a palette (count, at most 256, and entry size 4, 16 bits each;
  premultiplied BGRA entries), then a 32-bit length and the 8-bit indices
  compressed the same way;
- 2: a 32-bit length and, compressed the same way, a lossy predictive
  transform padded with zeros to a multiple of 16 bytes: the alpha plane
  (one byte per pixel), one predictor code per row, then the high bytes
  and then the low bytes of three 16-bit values per pixel.

Xcode writes whichever of modes 2, 3 and 4 (when there are at most 256
colours) is smallest, unless mode 1 is chosen instead (below).

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

A greyscale image (GA8, below) is the same with 2-byte pixels: the stream
header's last byte and the body header's second word are 2, mode 3 holds
premultiplied grey and alpha per pixel, and mode 2 the alpha plane, the
row codes and one 16-bit residual per pixel (high bytes, then low), the
predictors working on the grey alone. No palette was seen for one, even
of 16 grey-alpha pairs. In grey rows Xcode breaks ties between 1 and 3, 3
and 4, and 1 and 2 by the order above (4,018 rows of 29 probes; 40 more
probes were byte-identical).

An image of more than 4 MiB (4 bytes a pixel) is `MLEC` with flags 1 (3
when opaque), method 11 and a count of `KCBC` chunks: each `KCBC`, two zero
words, its rows and its length, then the `01`, `04`, length, zero header and
a `dmp2` stream of those rows alone (mode chosen per chunk). A chunk is as
many rows as fit 4 MiB: 1441 px wide, 727, 727 and 706 rows.

Mode 4 has a second form, also a candidate in the competition for the
smallest: at most 256 premultiplied colours without their alpha, the
entry count's companion 3 (not 4) and each entry's fourth byte 0, then the
alpha plane and the index plane compressed together (WebRTC's incognito
banner, 152 such colours where its colours with alpha number 204; 63 of
the examined streams have it).

Pixels are premultiplied as `(c * a + 127) / 255`, after any colour
conversion (below). A palette that would
need strips (over 2 MiB of indices) is not established and is refused. Every layer in the
examples uses mode 4. Where a row's predictor isn't established and mode 2
could be the smallest, the image is refused; so are ties between modes.
Mode 1 stores the pixels as they are after the header, with no length
(BGRA, or grey and alpha for GA8): images of fewer than 8 pixels (1x1 to
1x7 and 2x2, opaque, translucent, black and white), and images the chosen
mode doesn't compress into a buffer 4 bytes smaller than the pixels. LZVN's
encoder gives up 9 or 10 bytes short of its buffer's end, depending on the
data, so an image whose stream would save 9 bytes on mode 1 may be stored
either way, and the test is the encoder's own: mode 1 when
`lzvn_encode_buffer` into the pixels' size less 4 bytes fails. That matches
194 of 194 small mode-2 images within 30 bytes of the margin, and 1,355 of
1,355 small random images (gradients with noise and with copied pixels,
4 to 16 pixels a side) compiled byte-identical; 245 more are refused for
their row predictors. Whether that holds for a stream compressed with LZFSE
(4096 bytes of data on) or a palette's is not established: one within 32
bytes of mode 1 is refused, as is an image that would be stored that way
in strips. Greyscale images are given the same test, unprobed near the
margin.

## Colour-managed PNGs

Xcode converts an image set's or app icon's PNG with an ICC profile other
than sRGB IEC61966-2.1's or GIMP's built-in sRGB to sRGB (each of the
corpus's 20 profiles compiled on an opaque gradient differs from the
gradient alone). ImageIO's image drawn into an 8-bit sRGB `CGContext` gives
Xcode's pixels exactly: compiling those pixels untagged gives Xcode's
output for 64x68 images (a 16-level colour cube and a grey ramp) with
Display P3, Adobe RGB, Generic RGB, ITU-709, ITU-2020, ROMM, DCI-P3, ACEScg
and sRGB profiles. Wide-gamut profiles (Display P3, Adobe RGB, ITU-2020,
ROMM, DCI-P3, ACEScg) add a 16-bit extended-sRGB rendition beside the
8-bit one. A PNG with `gAMA` other than 1/2.2 or `cHRM` other than sRGB's
is converted too; an `sRGB` chunk changes nothing.

Of these, this actool converts the profiles with one 8-bit rendition that
it knows: Generic RGB and ITU-709, matched by their colorants, white point
and curves (`colour.rs`). The conversion below equals that drawing for
all 16,777,216 colours of each, as it does for Display P3, Adobe RGB (with
its curve measured), ROMM, ITU-2020, DCI-P3 and ACEScg with their own
matrices and curves. With the two profiles, 49 image sets (gradients,
noise, few colours and patterns, opaque and translucent, 6 to 300 px) and
6 icon sets (a single 1024 pt image, an image per size, legacy slots)
compiled byte-identical; 15 more image sets were refused for their row
predictors.
Other profiles, profiles with `gAMA` or `cHRM` beside them, and greyscale
PNGs with profiles are refused. Translucent pixels are converted, then
premultiplied.

The conversion, for each channel of an 8-bit pixel `x`:

- 12-bit linear `q = round(L(x / 255) * 4096)`, with `L` the profile's
  curve: a parametric curve of type 3 in double precision, except that
  BT.709's parameters (ITU-709, ITU-2020) are taken as a pure gamma 2.4; a
  gamma of 1.5 or more has a toe, `L = max(x^g, x / 16)`; a `curv` gamma of
  461/256 is snapped to 1.8 (Generic RGB);
- `idx = clamp((sum M[r][c] * q_c) >> 15, 0, 8191)` with `M` the profile's
  matrix to sRGB in s15Fixed16 integers (always multiples of 4);
- the output is `E[(idx + 1) >> 1]`, with `E[k] = round(enc(k / 4096) *
  255)` for the sRGB encoding `enc`, except at 65 of the 4097 entries,
  all within 0.03 of a half.

Three parts are measured data, not derived: the 65 exceptions in `E`
(an approximate encoding: no LUT size, interpolation or rounding tried
reproduces them), each profile's matrix (read back from the float path:
an extended-linear float context's outputs are `M . q / 4096` exactly; the
matrices are not the Bradford-adapted products of the profiles' colorants,
which miss by up to 1.5 units), and the two curves' 12-bit tables (read
back the same way, with own profiles of sRGB's colorants and one curve).
Someone who derives any of them can replace them in `colour.rs`; a test
checks the encoding table against its checksum. Adobe RGB's gamma
(563/256) and a parametric gamma of exactly 2.2 follow a curve near
`x^2.2` that differs at 75 of 256 inputs and is not implemented. Display
profiles (curves as tables, as in the corpus's three refused catalogs)
aren't established either.

## Image sets

For iOS, an image set's PNG images (universal idiom, at 1x, 2x or 3x, or no
scale meaning 1x) are each a rendition keyed by scale, the set's identifier
(the name hashed as for app icons), element 85 and part 181, with a facet of
the same element, part and identifier. Each is a `dmp2` image (above),
layout 12, named after its file: TLVs 0x3e9 (1, 0, 0, width, height), 0x3eb
(1, 0, 0, 0, 0, width, height), the composition, 0x3ee 1 and 0x3ef the
bytes per row rounded up to 32 (240 is 256). Its CSI flags are the
rendering intent: 16 automatic (no property), 8 template, 0 original. The
key format is the app icons' without `dimension2` when nothing uses it.

An image whose every pixel is grey (R = G = B, whatever its PNG's colour
type) is stored as GA8: pixel format ` 8AG`, colour space 2, a greyscale
`dmp2` payload, and bytes per row of one a pixel when opaque and four
otherwise, rounded up to 32.

Images small enough to pack (18,050 px² per scale factor, as for icons)
are packed per scale, translucent colour, translucent grey, opaque colour
and opaque grey images each in atlases of their own, as icons are packed (largest area first, equal areas by identifier,
twelve probes of equal-area shapes): an atlas is a `dmp2` image (layout
1004, flags 0, keyed element 9, part 181, scale and, from the second at a
scale on, dimension1 its index) named `ZZZZPackedAsset-{scale}.{opaque}.{grey}-gamut0`
and flagged opaque as its group is, whatever its padding (an opaque grey
atlas's padding is opaque too: alpha 255, grey 0); each image is a
layout 1003 link to it flagged with its rendering intent. A group's only
packable image is stored on its own. Atlases are filled next-fit, one
closed once its area reaches the cap and the next taking the following
dimension1 (and the same name), even for a single image. The key format
has dimension1 only when an atlas uses it.

Folders are scanned for image sets: a folder's assets are named as at the
top, or with its name and a slash before theirs when its `Contents.json`
says `provides-namespace` (nested ones too: `Buttons/Small/dot`); the
identifier is the hash of that full name. Other assets in folders, and
other folder properties, are refused.

Refused: atlases whose `dmp2` row
predictors aren't established (common: the row under an atlas's empty top
row ties between predictors 1 and 2, which Xcode breaks neither by cost,
summation order, nor compressed size),
vector images (SVG, PDF), other idioms, appearances and other properties,
and image sets on other platforms. 90 random catalogs of one to four image sets (opaque,
translucent, single-colour, tiled and noisy images, 20 to 420 px, with and
without an app icon, simulator and device, deployment targets 13 to 18,
with and without `--compress-pngs`) and six images of over 4 MiB were
byte-identical.

## Colour sets

A colour set's colour is a rendition keyed by scale 1, the set's identifier
(its name hashed as for image sets), element 85 and part 217, with a facet
of the same element, part and identifier: a CSI of layout 1009 named after
the set, no bitmap (flags, size, scale, pixel format and colour space 0),
TLVs 0x3ec (0, 0.0) and 0x3ee (1), and the payload `COLR` (written `RLOC`),
1, the colour space's code, the component count and the components as
doubles. Codes: sRGB 1, gray-gamma-22 2, Display P3 3 (also when no space is
named), extended sRGB 4, extended linear sRGB 5, extended gray 6. A
component with a decimal point is read as a 32-bit float (`0.800` is
0.800000011920929); one without, decimal or `0x` hex, as an integer out of
255 (`1` is 1/255). Colours take bitmap keys as images do. Measured with
twelve probes of every space and form, and colours beside image sets, an
app icon and in a namespace folder.

A dark colour is appearance 1 and adds `UIAppearanceDark` (1) to the
appearance keys, but which of the two keys Xcode allocates first varies
(Any first in 7 of 12 small catalogs, with nothing seen to tell them
apart), so other appearances are refused, as are other idioms. In
catalogs with colours, identifiers that share a bucket of the bitmap key
table come out in an order not established (25 of 54 random colour
catalogs), so those are refused, as with atlases.

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
the same radiosity image, and the same bytes on every run, so it depends
neither on the GPU nor on timing.

That is exact but for Xcode's arithmetic, which is not established. A flat
cell (16 equal blocks, of a level u) comes out 64u - 1 for 36 of the 255
levels (a set closed under doubling, so it depends on the level's
mantissa); actool has the set, measured for every level. Other values come
out one more or one less when they are close to a whole number: of 256,000
measured from about 550 stacks, 6,660 did, none further from it than 0.56 *
2^-24 of the terms' magnitudes (the sum of the absolute weighed blocks)
plus 0.002. Which ones do depends on the blocks, not only on the value:
about half of 12,600 cells constructed to be whole numbers came out one
less, and a pattern and its transpose almost always agree. No model tried
reproduces that: single- and half-precision sums over blocks or pixels in
every order (all 63 million for flat cells), separable passes, fused
multiply-adds, products rounded before an exact sum, separate positive and
negative sums, offsets, vImage's convolutions, Core Image's bicubic scaling,
and Metal shaders with and without fast math.

So when strict (the default), actool refuses a tvOS icon with any value
within 2 * 2^-24 of its terms' magnitude plus 0.005 of a whole number
(values in (-1, 1) excepted, which truncate to 0 either way). That refuses
most icons, including the examples'. Relaxed, `--config=relaxed`
(`RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed`), it writes them anyway:
the examples' icon is then byte-identical, and of 20 random three-layer
stacks 13 were, the others one value off. What remains to find is how Xcode computes the
cells; the measured cells and probes are described above.
Grey layers (Xcode keeps them GA8), translucent visionOS layers,
other brand asset roles and sizes, and layers small enough to pack into an
atlas are refused.
