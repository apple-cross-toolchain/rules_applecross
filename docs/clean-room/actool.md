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
(`examples/ios/HelloWorldSwiftUI`). An image set or app icon set whose
`Contents.json` has no `images` list at all is as one with an empty list
(two probes, one of them a Flutter catalog with both).

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

## Dark and tinted app icons

A single 1024 px iOS icon may have dark and tinted images (`appearances`
luminosity dark or tinted): per idiom, each is a rendition of the 1024 px
image keyed by its appearance (dark 1, tinted 10, named
`ISAppearanceTintable` in the appearance table), its MLEC versions saying
whether it is opaque (2 and 3, or 0 and 1, as other iOS icons'), and stored
as GA8 when its every pixel is grey (2 bytes a pixel, rows 32-byte aligned,
coded as BGRA ones are; a coloured tinted image stays BGRA). The bundle's
icons and the partial Info.plist are the plain icon's. Other appearances are
refused (four probes and Chromium's legacy icons, equivalent but for the
appearance blocks' order).

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
run-length coding, opaque or not: after `MLEC`, the version (2 opaque, 0
not), 1 and the length, the pixel format (4, BGRA; 3 for GA8), width,
height and each row's offset from the format, then each row's runs, a count
with the top bit set followed by one premultiplied pixel, or a count
followed by that many pixels. A row equal to the one above takes its
offset (an equal row further up doesn't). A run is taken, greedily from the
left, where it spans more bytes than its pixel and two counts: four pixels
and up in BGRA, six in GA8 (a run at a row's start, middle and end, two
runs side by side, rows of runs from 1 to 20 pixels; 13 probes). Xcode ignores entries that aren't iOS icon slots (Flutter's
iPhone 40 and 60 pt at 1x, iPad 76 pt at 3x, which this actool leaves out
too: one probe of a Flutter set, byte-identical); this actool compiles the
current and legacy slots (iPhone 20, 40 and 60 pt at 2x and 3x, 29 pt at
1x to 3x, 57 pt at 1x and 2x; iPad 20, 29, 40, 50, 72 and 76 pt at 1x and
2x, 83.5 pt at 2x) and the App Store's 1024 pt, and refuses the rest.
CarPlay's 60 pt icons at 2x and 3x (idiom 4, its own multi-size index and
images, as an iPhone's) go with an iPhone build (without one they are
refused). An image the set has but no slot uses (only a file name, perhaps
`unassigned`) is ignored (five probes and four corpus catalogs).

## Packed images

Small images are packed into atlases
(`ZZZZPackedAsset-<scale>.<1, or 0 for macOS>.0-gamut0`, element 9, part
181, layout 1004); each image is then a rendition of layout 1003 with a
`KLNI` TLV (0x3f2): its x, y, width and height in the atlas, then `0c00`, a
32-bit length, and the atlas's key as (attribute, value) pairs of 16 bits in
attribute order, ending with four zero bytes. Measured over 70 random sets
and every icon set in the examples:

- BITMAPKEYS' blocks follow a table of the identifiers, filled in the
  order their renditions were first allocated: CoreFoundation's number
  hash, bucket counts and linear probing (as CFBasicHash.c publishes), but
  growing as soon as an insertion takes it past its capacity, not before
  one would. 7, 13, 23 and 41 buckets grow on the 7th, 12th, 20th and 33rd
  entry, and a full 3-bucket table before the 4th. That reproduces all 258
  catalogs of the 350-catalog comparison with BITMAPKEYS. The 14 of them
  that CoreFoundation's own growth missed all have atlases, identifiers
  sharing a bucket, and a count at or past a capacity, so where the table
  grew shows. 52 entries,
  85 to 88 and 119 or more aren't pinned (the 71- and 127-bucket tables'
  growth), and catalogs with those counts are refused.
- Per (scale, idiom, subtype), images are taken largest area first, equal
  areas by name identifier. One of more than 18,050 px² × scale is not
  packed; neither is the only packable image of a group.
- Atlases fill next-fit: an atlas takes images while the area it already
  holds is below the cap, then the next atlas begins. That holds where the
  next image would grow the closed atlas; where it would fit in it, Xcode
  sometimes keeps adding a few more images, and how many isn't established
  (23 probes of one shape repeated: 54 x 60 px at 3x takes three more when
  at least three remain and none otherwise, 60 x 54 never more; the
  images' content plays no part). Such catalogs are refused. A second
  atlas's key format includes `dimension1`. A scale's atlases take their `dimension1` in the
  order of their creation indices as decimal strings: with 19 atlases the
  third is 11 and the eleventh 2 ("0", "1", "10", ..., "18", "2", ...). With
  ten or fewer that's their creation order.
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
and sRGB profiles. A wide-gamut profile adds a 16-bit extended-sRGB
rendition beside the 8-bit one, but only for an image with colours well
outside sRGB (below). A PNG with `gAMA` other than 1/2.2 or `cHRM` other
than sRGB's is converted too; an `sRGB` chunk changes nothing.

This actool converts any RGB profile with colorants and one curve for all
three channels, in the forms below, and refuses the image where Xcode
would add a 16-bit rendition (`colour.rs`). The conversion below equals
that drawing for all 16,777,216 colours of Generic RGB, ITU-709, Display
P3, Adobe RGB, ROMM, ITU-2020, DCI-P3 and ACEScg. Translucent pixels are
converted, then premultiplied. Refused:
- profiles with lookup tables (`A2B0` and the like);
- a different curve per channel (below);
- curves not established (below);
- profiles beside `gAMA` or `cHRM`;
- greyscale PNGs with profiles.

What compiled byte-identical against Xcode 27.0:
- Generic RGB and ITU-709: 50 image sets (gradients, noise, few colours
  and patterns, opaque and translucent, 6 to 300 px; 14 more refused for
  their row predictors) and 6 icon sets (a single 1024 pt image, an image
  per size, legacy slots).
- The corpus's display profiles and the wide-gamut system profiles, on
  grey, muted, gradient, noise, pastel (also translucent) and saturated
  images, with their 16-bit renditions (below): 59 of 63 image sets, the
  rest refused for their row predictors.
- Profiles of our own with Generic RGB's colorants and each curve form:
  44 image sets, the rest refused for their row predictors.
- A display profile's icon sets (single size and an image per size).
- RxExample's catalog, an app icon with a display profile whose curves
  are 1024-entry tables, a WebRTC catalog with a Display P3 image, and
  rules_xcodeproj's, whose display profile gives its image a 16-bit
  rendition: the 350-catalog comparison went from 98 byte-identical to 101.

### Curves

- `curv` with no entries, or gamma 256/256: linear.
- `curv` gamma 461/256: 1.8 with the toe (below). Gamma 563/256: the 2.2
  path (below). Other `curv` gammas aren't established.
- A `curv` table that ColorSync takes for sRGB's curve: sRGB's parametric
  curve, evaluated as type 3 is. A table counts as sRGB's when its linear
  interpolation stays near sRGB's curve everywhere. Tables of 256, 1024 and
  4096 entries within 4.78 (of 65535) were taken. A 64-entry table, 6.32
  away, and tables 8 or more away (a single entry included) weren't.
- Other tables aren't established, and are refused. ColorSync reads them
  one of two ways. A power-like table becomes a gamma it estimates from the
  table, evaluated on the input widened to 12 bits, `v12 = (257 * x + 8)
  >> 4` (`[0, 0.25, 1]` gives 2.0). Any
  other table is interpolated linearly at `min(v12, 4095) / 4096` (exact on
  the tables probed). Neither the estimate nor which tables count as
  power-like is established.
- `para` type 0: a gamma of 1.0 to 3.0, with the toe, except exactly 2.2
  (the 2.2 path).
- `para` type 3: in double precision, except BT.709's parameters as
  ITU-709 and ITU-2020 write them, taken as gamma 2.4. Parameters near
  BT.709's are taken the same way by a rule not established, and are
  refused. Other types aren't established.

With a different curve on each channel, ColorSync widens every channel's
input to 12 bits (`v12`): gamma 1.8 with its toe then
matches exactly on the widened input. An sRGB table on one of the channels
misses 4 of 256 there, so such profiles are refused.

### Wide gamut

Xcode adds the 16-bit rendition only for profiles CoreGraphics calls wide
gamut (`CGColorSpace.isWideGamutRGB`), and only for an image with colours
outside sRGB beyond a margin.

A profile is wide when its colorants' triangle in xy chromaticity has an
area of 0.13447 or more (1.2327 times sRGB's). The flip fell between 0.134467
and 0.134471 on every path tried: moving the red primary out, moving the
green primary out, and moving the colorants toward Display P3's or Adobe
RGB's. Moving the blue primary out never reached it. The curves and white
point don't count.
- Wide: Display P3, Adobe RGB, ITU-2020, ROMM, DCI-P3, ACEScg.
- Not wide: Generic RGB, ITU-709, sRGB, and the corpus's display profile.

Xcode follows the API at the flip: a saturated image with profiles just
either side of it got one rendition and two.

From a wide profile, Xcode adds the rendition when any channel of any
pixel, as the 16-bit rendition stores it (premultiplied, in 12-bit
encoded units, below), is -103 or less or 4110 or more. -102 and 4109
didn't, and -103 and 4110 did (Display P3, opaque pixels and translucent
ones), so the edges are exact. A transparent pixel never counts: a fully
transparent saturated red, or one at alpha 1 or 8, doesn't add it; at 64
it does. Opaque values can't fall between -90 and -103 (the encoding near 0
steps by about 13 there), which is why linear probes put the edge between
-0.00181 and -0.00189. No narrow profile probed added the rendition, though
RxExample's icon reaches -0.023 and 1.041 linear under its display profile,
and Generic RGB's primaries fall outside sRGB too.

### The 16-bit rendition

A second rendition of the image, keyed with display gamut (attribute 24)
1 where the 8-bit one has 0:
- **Pixels:** half floats, R, G, B, A, little-endian, extended sRGB.
  Each channel is the 12-bit encoded value of the conversion below, not
  clamped and mirrored for negatives: `e12 = sign(k) * round(enc(|k| /
  4096) * 4096)`, with `k = ((sum M q >> 15) + 1) >> 1`. Alpha is widened to
  12 bits, `A12 = (257 * a + 8) >> 4`. Colours are premultiplied in 12 bits,
  `round(e12 * A12 / 4096)` (halves up), and stored as the nearest half
  float of that over 4096 (ties to even); alpha as that of `A12 / 4096`.
  This equals ImageIO's image drawn into an extended-sRGB half-float
  `CGContext`, which equals Xcode's raw pixels. The model equals that
  drawing for all 16,777,216 colours of Display P3 and of Adobe RGB, and
  3,825 translucent values across alpha (from five colours).
- **Header:** pixel format `RGBW` (stored `WBGR`), colour space 4, 0x3ef
  eight bytes a pixel rounded up to 32. The MLEC body's second word and the
  `dmp2` header's fourth byte are 0x14.
- **`dmp2` modes:**
  - mode 1 below 8 pixels, the half floats as they are;
  - mode 3, the half floats compressed;
  - mode 2, lossy: each colour as `round(h * 512)` (halves away from zero),
    with R, G, B in the places B, G, R take at 8 bits, and the alpha plane
    the source's 8-bit alpha.

  The smallest wins, as at 8 bits; no palette was seen, even for four
  colours. Mode 2's strips are the 8-bit ones' (`7 w + 1` bytes a row: a
  1000 px image in strips of 299 rows).
- **Atlases:** small images are packed as at 8 bits, but the 16-bit
  renditions of colour images go in atlases of their own, per scale and
  opacity. They're named `ZZZZPackedAsset-{scale}.{opaque}.0-gamut1`, keyed
  with gamut 1, and their dimension1 is counted apart from the 8-bit
  atlases'. The links carry the gamut in the atlas's key.
- **Key format:** gamut comes after dimension1 when an image has the
  rendition. BITMAPKEYS tracks it as a mask (3 for both gamuts).

102 probe catalogs with the rendition were byte-identical against Xcode
27.0: single images (tiny to 1000x700, opaque and
translucent, Display P3, Adobe RGB, ROMM, ITU-2020, DCI-P3, ACEScg and
display profiles) and atlases (both gamuts, translucent and opaque, two
8-bit atlases with one 16-bit). App icons and image stacks that would need
the rendition are refused, as are macOS catalogs (their key format isn't
established), images of more than 4 MiB as BGRA, and two 16-bit atlases'
order, which wasn't compared byte for byte. Images whose rows' predictor
ties (1 against 2 or 3 with identical residuals) are refused as at 8 bits;
Xcode breaks those either way (176 such rows in the probes), by a rule not
established.

### Conversion

The conversion, for each channel of an 8-bit pixel `x`:

- 12-bit linear `q = round(L(x / 255) * 4096)`, with `L` the profile's
  curve: a parametric curve of type 3 in double precision, except that
  BT.709's parameters (ITU-709, ITU-2020) are taken as a pure gamma 2.4; a
  gamma has a toe, `L = max(x^g, x / 16)` (which matters from about 1.5
  on); a `curv` gamma of 461/256 is snapped to 1.8 (Generic RGB);
- `idx = clamp((sum M[r][c] * q_c) >> 15, 0, 8191)` with `M` the matrix
  to sRGB in units of 2^-16 (below);
- the output is `E[(idx + 1) >> 1]`, where the sRGB encoding `enc` is
  itself rounded to 12 bits: `E[k] = round(round(enc(k / 4096) * 4096) *
  255 / 4096)`.

`M` comes from the colorants alone (not the white point or curves; a
profile of our own with a profile's colorants and linear curves gives the
same matrix). Each row of the source colorants `S` (XYZ by R, G, B) is
scaled to sum as the same row of the sRGB profile's colorants `D` does,
so white maps to white, and `M = D^-1 * S'`, computed in double precision.
`M` is then an int16 matrix over a power of two: 2^14 while every entry is
under 2 in magnitude, else 2^13 (ROMM's 2.03), so its entries are
multiples of 4 or 8 units of 2^-16. Each entry rounds to the nearest.

That derivation gives every entry read back from CoreGraphics:
- Display P3, Adobe RGB, Generic RGB, ROMM, ITU-2020, DCI-P3 and ACEScg;
- a profile with diagonal colorants (every diagonal gives the same matrix);
- 40 profiles sweeping one colorant entry from 0 to 1.

The one exception is Adobe RGB's `M[1][0]`, which can't be read back: 0
and the derivation's -4 convert all 16,777,216 colours identically.

The float path gives those matrices exactly. With an extended-linear
32-bit float context, an 8-bit image's outputs are `round(M . q / 65536)
/ 4096`. Primary ramps and a few thousand random colours leave one
candidate per row, except rows near the identity.

The three parts were first measured: `E`'s 65 departures from
`round(enc(k / 4096) * 255)`, the matrices, and the curves' 12-bit tables.
The derivations above reproduce all of them, and tests check them against
the measured data.

A gamma of exactly 2.2 takes another path. That's a parametric 2.2 (its
s15Fixed16 value; the neighbouring values are ordinary gammas) or a
`curv` gamma of 563/256, as in Adobe RGB. It has no toe, and widens the
8-bit input to 12 bits first: `v12 = (257 * x + 8) >> 4`, then
`q = round((v12 / 4096)^2.2 * 4096)`. That reproduces the 12-bit table read
back for a parametric 2.2 profile of our own and for each of Adobe RGB's
channels, all 256 entries, and is no closer than 0.003 to a rounding
tie. A 16-bit image's float output is `x^2.2` to float precision; only the
8-bit path does this.

## Colour sets

A colour set's colours (iOS) are renditions keyed by appearance, scale 1,
the set's identifier, element 85 and part 217; the set's facet has element,
part and identifier. Each is a CSI with no size, scale or pixel format,
layout 1009, the set's name, TLVs 0x3ec (0 and opacity 0.0) and 0x3ee 1,
and a payload of `RLOC`, version 1, the colour space's code, the
component count and the components as doubles.

- **Colour spaces:** sRGB 1, `gray-gamma-22` 2, Display P3 3 (also when
  no space is named), `extended-srgb` 4, `extended-linear-srgb` 5 and
  `extended-gray` 6. The grey ones have white and alpha.
- **Components** are read as text:
  - with a decimal point, as a single-precision float ("0.333" is
    0.3330000042915344);
  - `0x` and hex digits, over 16^n - 1 ("0x33" is 0.2, "0x1FF" 511/4095);
  - otherwise the leading integer over 255, alpha included ("1" is 1/255,
    "1e-1" too, "256" is 256/255).

  Leading spaces are skipped and nothing is clamped (1.5 and -0.25 stay).
- **Appearances:** none 0 (`UIAppearanceAny`), dark 1 (`UIAppearanceDark`),
  high contrast 2 (`UIAppearanceHighContrastAny`), dark and high contrast 3
  (`UIAppearanceHighContrastDark`), light 4 (`UIAppearanceLight`), and light
  and high contrast 5 (`UIAppearanceHighContrastLight`). The catalog's
  APPEARANCEKEYS lists Any and those used, by name.
- **Xcode's own output varies:** it allocates the appearance table's name and
  value blocks in a different order on each run (six runs of one catalog gave
  six orders), so a catalog with appearances has no one Xcode output. This
  actool allocates them by name, an order Xcode also produced; that run's
  output and this actool's were byte-identical. Other runs differ only in
  where those blocks lie.

Colours take bitmap keys as images do, in the order of the identifier
table under Packed images; 54 of 54 random catalogs of 2 to 39 colour sets
were byte-identical with it (25 of 54 like them had identifiers sharing a
bucket, which the earlier model refused).

Refused: idioms other than universal (Xcode thins them by target device: an
iPhone build kept iphone and car and dropped universal), macOS catalogs,
and keys not above. In the 350-catalog comparison 13 more catalogs matched:
8 byte-identical and 5 matching but for the appearance blocks' order.

A colour may refer to a system colour (`"reference"`, with platform
universal or ios): its rendition holds the colour as Xcode resolves it for
the appearance (a table of the 39 it knows, from one probe of each in each
appearance; light is resolved as dark and high-contrast light as
high-contrast dark, all 39 checked), with a reference flag (1) in the byte
after the space's code, then `RLOC`, version 1, the name's length and the
name. Names Xcode doesn't know (`whiteColor`) resolve to a placeholder
colour; they are refused (ten probes).

## Image sets

For iOS, an image set's PNG images (universal idiom, at 1x, 2x or 3x, or no
scale meaning 1x) are each a rendition keyed by scale, the set's identifier
(the name hashed as for app icons), element 85 and part 181, with a facet of
the same element, part and identifier. Each is a `dmp2` image (above),
layout 12, named after its file: TLVs 0x3e9 (1, 0, 0, width, height), 0x3eb
(1, 0, 0, 0, 0, width, height), the composition, 0x3ee 1 and 0x3ef the
bytes per row rounded up to 32 (240 is 256). Its CSI flags are the
rendering intent: 16 automatic (no property), 8 template, 0 original. Xcode
also takes `template-rendering-intent` from the image set's `info`. The
key format is the app icons' without `dimension2` when nothing uses it.

An image whose every pixel is grey once premultiplied (R = G = B, whatever
its PNG's colour type: a transparent red pixel in a grey image, or (10, 12,
11) at alpha 1, keeps it grey; (200, 0, 0) at alpha 1 doesn't) is stored as
GA8: pixel format ` 8AG`, colour space 2, a greyscale
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
packable image is stored on its own. Atlases are filled next-fit (refused
where the next image would fit in the closed atlas, see above), one
closed once its area reaches the cap and the next taking the following
dimension1 (and the same name), even for a single image. The key format
has dimension1 only when an atlas uses it.

An image may be for an appearance, as a colour set's colour is (luminosity
light or dark, high contrast, or both): its rendition's key has that
appearance's value, and the catalog's appearance table its name. Each
appearance has atlases of its own at each scale, with the same names, keyed
by the appearance too. A link to one names its key by attribute but with the
appearance last, after the scale. A second atlas for an appearance at one
scale is refused (seven synthetic probes and four of real images, dark and high
contrast at each scale; byte-identical or equivalent but for the appearance
blocks' order).

With both `--target-device iphone` and `ipad`, an image may be for the
iPhone, iPad or CarPlay idiom: its key has idiom 1, 2 or 4 (none for
universal), and each idiom has atlases of its own, keyed by it, the link
naming its key in attribute order. Watch, Mac and TV images are left out
(a set of nothing else has no facet, a catalog of nothing else compiles to
nothing). Other idioms, device idioms with other target devices, and
atlases for both an appearance and an idiom are refused (13 probes).

Folders are scanned for image sets: a folder's assets are named as at the
top, or with its name and a slash before theirs when its `Contents.json`
says `provides-namespace` (nested ones too: `Buttons/Small/dot`); the
identifier is the hash of that full name. Other assets in folders, and
other folder properties, are refused. Files that aren't assets (a README,
with or without an extension, at the top or in a folder) are ignored
(three probes). So are image, colour and data sets without a
`Contents.json` (three probes; an app icon set without one is an empty set,
compiling to nothing even when `--app-icon` names it, one probe) and,
without a word, an image or icon whose file is missing (five probes:
an icon set of only missing files compiles to nothing; one whose 1024 px
image is missing but whose dark and tinted ones aren't is refused).

Refused: atlases whose `dmp2` row
predictors aren't established (common: the row under an atlas's empty top
row ties between predictors 1 and 2, which Xcode breaks neither by cost,
summation order, nor compressed size),
vector images (SVG, PDF) and other properties,
and image sets on other platforms. 90 random catalogs of one to four image sets (opaque,
translucent, single-colour, tiled and noisy images, 20 to 420 px, with and
without an app icon, simulator and device, deployment targets 13 to 18,
with and without `--compress-pngs`) and six images of over 4 MiB were
byte-identical.

### JPEG images

A JPEG in an image set is stored as a file, not decoded: a rendition keyed
as a PNG's, layout 12, pixel format `JPEG`, no width, height or colour
space in its header, the same TLVs as a `dmp2` image but 0x3ef, and the
JPEG as a `RAWD` payload (version 0, length, bytes). JPEGs are never packed
into atlases, whatever their size. Xcode keeps the file byte for byte when
it holds JFIF, tables, frames (baseline or progressive), a restart
interval, the scan and the standard 3,144-byte sRGB IEC61966-2.1 profile,
and drops comments (seven probes, and all 24 of the corpus's JPEGs that hold no more).

Other metadata goes through ImageIO. Ducky (APP12) segments are dropped and
Adobe (APP14) segments kept as they are (two probes). Exif of no entries
is dropped (two of the corpus's), and Exif of only an orientation, as
ImageIO writes it (big-endian, its first IFD at 8), kept as it is, the
orientation also recorded in the rendition's 0x3ee TLV (1 otherwise; two
probes). A Photoshop (APP13) segment of only an empty IPTC record, its
digest or both is dropped (one probe, one of the corpus's), and so is XMP
without properties (two probes). Other Exif is rewritten (little-endian
Exif comes out big-endian, some entries dropped and values changed), and
XMP's properties become Exif and IPTC (an XMP orientation becomes Exif's;
a title and byline, IPTC's): those are refused, as are JPEGs with Adobe
RGB or other profiles, which were re-encoded.

### PDF images

Xcode stores a PDF as it is (layout 9, pixel format `PDF `, `RAWD`) and
also rasterizes it at 1x, 2x and 3x into `dmp2` renditions (grey and alpha
for a black drawing). Matching those needs CoreGraphics' rasterizer and
antialiasing, which this actool doesn't reproduce: PDF image sets are
refused, with or without `preserves-vector-representation`.

## Data sets

A data set's file (one, universal, with its `universal-type-identifier`) is
a rendition keyed as an image set's at scale 1, with a facet as an image
set's: layout 1000, flags 0, no size, scale 100, pixel format `DATA`
(stored `ATAD`), colour space 0, named `CoreStructuredImage`. Its TLVs are
the composition, 0x3ed (the identifier's length with its NUL, 0, then the
identifier and a NUL) and 0x3ee 1, and its payload the file as `RAWD`
(version 0, length, bytes), whatever its size. A catalog of data alone has
no appearance table. Without a type identifier Xcode looks one up for the
file's extension (`.drawing` is `com.apple.drawing`), which this actool
refuses, as it does several files or other idioms (eight probes).

## Launch images

A launch image set (`.launchimage`) compiles to nothing unless
`--launch-image` names it: a catalog with one and an image set gives the
same Assets.car and partial Info.plist as the image set alone. The named
set's images go into the bundle as files, copied byte for byte, and the
partial Info.plist lists them under `UILaunchImages`. Observed with 13
catalogs, four of them from the corpus:

- An image's name is the set's, then `-` and its minimum system version
  without the dot (7.0 is `700`, 8.0 `800`), then what tells it apart: an
  iPhone's 4-inch (`retina4`) image `-568h`, 4.7-inch `-667h`, 5.5-inch
  `-Portrait-736h` or `-Landscape-736h`, the 3.5-inch one nothing; an
  iPad's `-Portrait` or `-Landscape`. The file adds `@2x` or `@3x` and,
  for iPad, `~ipad`: `LaunchImage-700-568h@2x.png`,
  `LaunchImage-700-Landscape@2x~ipad.png`.
- `UILaunchImages` has one entry per name, in the set's order of their
  first image: `UILaunchImageMinimumOSVersion`, `UILaunchImageName`,
  `UILaunchImageOrientation` (`Portrait`, `Landscape`) and
  `UILaunchImageSize`, the portrait size in points (`{414, 736}` for
  both 5.5-inch images, `{768, 1024}` for iPad's).
- Images for devices the build doesn't target are left out; slots without
  a file are Interface Builder's alone.
- A set without a single image gives an iPhone build an image Xcode makes:
  a black 640x1136 `-700-568h@2x`, written as Xcode writes the simulator's
  icons (see Simulator PNGs), with its entry. A set whose only images are
  for iPad gives an iPhone build nothing.

Refused: other system versions and subtypes, tvOS launch images, extents
other than full screen, and images other than PNGs.

## Image stacks

An iOS build leaves tvOS brand assets and visionOS solid image stacks out
unless `--app-icon` names them: the Assets.car is the same as without them
(two probes; a catalog of nothing else compiles to nothing).

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
