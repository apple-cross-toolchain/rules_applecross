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
radiosity image can't be guaranteed to be Xcode's (below). Accepting that
(`--config=relax_actool_tvos_radiosity`), the output is byte-identical to Xcode's.

Outputs, all byte-identical to Xcode 27.0's for `examples/ios/CleanRoom`:
`NAME60x60@2x.png` (iPhone), `NAME76x76@2x~ipad.png` (iPad), `Assets.car`
and the partial Info.plist (`CFBundleIcons`, `CFBundleIcons~ipad`). For this
catalog Xcode's output is the same on every run, and the timestamp
`assetutil --info` prints is the file's modification time, not content.

Xcode's `Assets.car` varies from run to run when a catalog has more than one
appearance (one dark colour is enough): it writes the blocks of the
`APPEARANCEKEYS` tree, each appearance's name and id, in a random order. The
entries and every other block stay the same. This actool always writes them
in one order. Comparisons with Xcode's output allow exactly that variation
([README.md](README.md#determinism)).

## Deployment targets before 2019's systems

For a deployment target before iOS 13, tvOS 13, watchOS 6 or macOS 10.15,
Xcode stores each image twice: a legacy rendition (palette-image
compression, no deployment target in its key) beside the usual one keyed
by deployment target 2019, the key format gaining
`kCRThemeDeploymentTargetName`. Colours and data sets are unchanged (an
image set, and a colour and a data set, at each side of each threshold).
This actool refuses images there; catalogs of colours and data compile.

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
differing sample. The image is premultiplied; each resampled pixel's alpha
is then raised to its largest colour component, so that it stays
premultiplied (a translucent icon's faint edges: five random icons, every
pixel). Merging the taps beyond an edge before rounding instead
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

A symbol set without a symbol (no `filename` in its `symbols`, or no
`symbols` list: Xcode's Accessory Setup Extension template has one)
compiles to nothing, beside other assets too; the catalog's `Assets.car` is
the same without it (`tests/actool_catalog` SY001). A symbol set with an SF
Symbols template is compiled only relaxed ([Symbol sets](#symbol-sets)).

A sticker pack compiles to nothing, its stickers too, with the arguments
rules_apple passes for a Messages extension (`--include-sticker-content`,
`--stickers-icon-role extension`, `--sticker-pack-identifier-prefix`): in a
`.xcstickers` or `.xcassets` catalog, beside its sticker icon set or not,
for the simulator or a device, from iOS 10 (probes of Xcode's Sticker Pack
Extension template and of a pack of two stickers; `tests/actool_catalog`
ST001). Xcode reads its stickers' `Contents.json` all the same, warning
when one isn't JSON, which this actool refuses; and anything but stickers
in a pack (sequences among them) is refused, not established.

## Assets.car

A BOM store (big-endian): 512-byte header area, blocks 16-byte aligned in
allocation order, the variables table, then the index (at least 256 entries,
else the block count plus one rounded up to 8) and an empty free list.
Variables, in order: `CARHEADER`, `RENDITIONS`, `FACETKEYS`,
`APPEARANCEKEYS`, `KEYFORMAT`, `EXTENDED_METADATA`, `BITMAPKEYS`.

Trees are a 29-byte header (`tree`, version 1, root page block, block size,
entry count, an inline-keys flag, the longest key) and leaf pages: entries
(value block, key block), four zero bytes, then a copy of every key; the
page's length is the block size plus the keys' lengths. When the keys
differ in length the header records 0xffffffff and the page holds no
copies (FACETKEYS with names of different lengths). `BITMAPKEYS` stores
its keys (name identifiers) inline in 1024-byte pages, the others in
4096-byte ones (see "Paged trees" below for trees of more than one page); their value blocks
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

Versions recorded in the car are the Xcode's whose SDK the build targets,
by its version in the SDK's `version.plist` (which gives the Xcode name and
build too): Xcode 27.0's CoreUI 1010, ImageIO 2851, CMPhoto 219 for iOS and
218 for the others; Xcode 27.2's CoreUI 1011 (`CoreUI-1011.4`), ImageIO
2851.1.8 for iOS and macOS and 2851.1.7 for watchOS, tvOS and visionOS,
CMPhoto 222 for all. Both write storage 17 and CoreThemeDefinition 664. Any
other Xcode is refused, whose versions would be recorded wrongly, unless
`--config=relax_xcode_version` records the nearest one's (27.0's before 27.2,
27.2's after). Those
versions are all that differs between the two Xcodes' catalogs: every
golden case, recorded again from Xcode 27.2 (2026-11-01), matched once they
were Xcode 27.2's.

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

This actool writes all of it, and everything but the flattened images'
pixels is Xcode's, byte for byte:
- the document's SVGs as Vector renditions (its PNGs as bitmaps), its
  colours (named `Color-N` in the order Xcode first uses them, equal ones
  shared, `Color-1` white) and gradients (`Gradient-N`, `system-light` and
  `system-dark`);
- for each appearance (light, dark and tintable; `UIAppearance…` or
  `NSAppearanceName…`), an IconImageStack of the background and the groups,
  and an IconGroup of each group's layers: frame, blend mode, opacity,
  glass, blur, specular, translucency, shadow and fill;
- the facets, the appearance keys, the MultiSized Image, the partial
  Info.plist, and the bundle PNGs and `.icns` but for their pixels.

Beside an iOS app's asset catalogs (rules_apple passes them in the same run)
the document's renditions join the catalog's, as Xcode's do: rules_apple's
test catalog, with dark variants, beside Visor's and Isomer's documents, for
the simulator and devices. On macOS too (below, macOS catalogs): the
catalog's atlases at 2x are numbered after the document's own, and images
that would share it (2x, no appearance, translucent colour, small enough to
pack) are refused, its pixels being this actool's.

The rules came from documents written to vary one thing at a time, Xcode's
output for each compared rendition by rendition. Among them, for dark: a
light layer with no fill or opacity of its own for dark takes the
background's light fill (as it is, or with the alpha of its fill's first
colour; a translucent `system-light` becomes a gradient from white to 0.961
grey); a layer is left out only when every appearance hides it or makes it
transparent without glass; and a layer's frame is its image's size times
its group's and its own scale, about the canvas centre, its corner rounded
down. Random documents made after the rules were written then caught what
the targeted ones missed, until a batch caught nothing: five batches, of 60,
100, 100, 150 and 150 documents for iOS and macOS, with no difference left
in the last two. With Visor's, Isomer's and rules_apple's documents, the
probes and the batches, 2,737 compiles are Xcode's.

Xcode's own runs differ in the flattened images' names (a fresh UUID,
process id and time) and in the icon facet's part (220 or 245, at random);
`tools/xcode_equivalence.py` allows exactly those, and, for a document, the
flattened images' pixels. The flattened images (1024 px per appearance for
iOS, 16 to 1024 px for macOS, the smallest packed together) are this actool's own: the
document's appearance rendered flat (the fill, then each group's layers back
to front with a blurred, downward-offset shadow; no glass, specular or
translucency), stored as Xcode stores its. An app built that way shows the
document's light, dark, tinted and clear icons as Xcode's does on iOS and
macOS 26 and later, which draw them from the layers.

Xcode's own flattened images make the output Xcode's, byte for byte:
`tools/icon_flattened.py DOC.icon`, run on a Mac with Xcode, compiles the
document with Xcode's actool for iOS and macOS and writes them into the
document's `Flattened/` folder (an `index.json` and lossless PNGs): the
1024 px Any, Dark and Tintable images and the bundle's `60x60@2x` and
`76x76@2x~ipad` icons, which Xcode draws from its 16-bit rendering; and for
macOS the System appearance at 16 to 512 pt @2x and 1024 pt @1x (each size
drawn anew, premultiplied as the catalog stores them) and the icon family's
members. The index names a SHA-256 of the document's other files (each
path, a NUL, its length and its bytes, in byte order of the paths), and
this actool uses the images only while the document still has it: strict
(the default), it compiles a document with a current `Flattened/` folder
and refuses one without, or with a stale one (run the script again after
editing the document). Xcode ignores the folder (its output is the same
with it), and the folder's images are the document's own output, made by
Xcode, not anything of Xcode's. `--config=relax_actool_icon_composer`
accepts a document without them, its rasters this actool's own.

Checked on Xcode 27.2: Visor's, Shireworks', Applications', Monitor's,
Overcast's, Convo's, Nimbus' three and VisorIsomer's documents and a solid
fill, each with its `Flattened/` folder, compiled for the simulator, for
devices (CgBI icons) and for macOS give Xcode's output byte for byte: every
file, and every rendition with its pixels. `tests/actool_icon` checks, off
a Mac, that a folder's images (made up there) are what the output holds and
that a stale one is refused.

`tests/actool_icon` compiles 55 of these documents (Visor's, Isomer's,
rules_apple's, and a sample of the probes and random batches) for the
simulator and macOS, three for devices, and one beside rules_apple's test
catalog, and compares the output with
digests of Xcode's (`tools/xcode_equivalence.py`'s `icon_output_digest`,
made by `update_expected.py` from two runs of Xcode each). CI runs it with
the actool it builds.

Refused, as not established:
- a scaled layer larger than half the icon, from an image other than 1024
  points at a scale not a multiple of 1/1024, with an edge exactly on a
  whole point: Xcode puts some of those edges a point further out (in bands
  of scale; 101 of 1,934 edges of a sweep of image sizes and scales), by a
  rule not found;
- a background or colour too near Xcode's threshold of light to tell which
  side it falls on;
- colour-managed PNG layers, SVG CSS (`<style>`), `currentColor`, paints
  with fallbacks or missing references, and keys Xcode's documents don't
  have.

What the system draws from (an experiment on the iOS 27 simulator and a
macOS 27 VM, with Assets.car files Xcode compiled from two different
documents and then mixed): every appearance (light, dark, and the Dark,
Tinted and Clear icon styles, on the home screen, Spotlight, the App
Library, Settings, the Dock and Finder) is drawn from the document's layers,
the Vector, IconGroup, IconImageStack, Color and Named Gradient renditions,
about 7 KB for Visor's. The rasters (the flattened Icon Images, 97–99% of
the file, the bundle PNGs and the `.icns`) were never shown; a car without
them drew correctly. They presumably serve systems before 26 and places not
checked (notifications, share sheets, the App Store). Xcode writes them for
a minimum of iOS 26 too.

The layers are structured data, to be reproduced as the rest of the catalog
is. The rasters are Apple's Liquid Glass rendering (rasterized SVGs,
compositing, blur, specular highlights, translucency), which can't be fitted
pixel for pixel from outputs alone. Xcode's own are deterministic on a Mac
(seven runs gave the same pixels; only their names change), but the
Tintable raster came out different on a macOS VM with the same Xcode, while
the Any and Dark ones and the bundle PNGs matched: perhaps because the VM's
graphics are virtualized, so which is "Xcode's" may depend on the machine.
Reproducing them exactly is left for later (black-box research has
reproduced the stages after the GPU and a solid fill's three appearances;
layers, glass and macOS's sizes not yet).
Until then a document's rasters are Xcode's only from its `Flattened/`
folder, else this actool's own, which `--config=relax_actool_icon_composer`
accepts.

## Dark and tinted app icons

`--alternate-app-icon` names an alternate icon (repeatable). Beside a
single 1024 px iOS primary icon, each single-size alternate is compiled as
the primary is (its renditions under its own name, dark and tinted images
included) but gets no bundle icons, and each target device's entry in the
partial Info.plist gains `CFBundleAlternateIcons`, before
`CFBundlePrimaryIcon`: each alternate by name, a dictionary of its
`CFBundleIconName` (keys sorted; a device the build doesn't target gets
none, as it gets no icon name). Six probes and rules_apple's test catalog.
Alternates beside a primary of several sizes, alternates of several sizes,
and other platforms are refused.

A single 1024 px iOS icon may have dark and tinted images (`appearances`
luminosity dark or tinted): per idiom, each is a rendition of the 1024 px
image keyed by its appearance (dark 1, tinted 10, named
`ISAppearanceTintable` in the appearance table), its MLEC versions saying
whether it is opaque (2 and 3, or 0 and 1, as other iOS icons'), and stored
as GA8 when its every pixel is grey (2 bytes a pixel, rows 32-byte aligned,
coded as BGRA ones are; a coloured tinted image stays BGRA). So is a grey
1024 px image itself, its bundle PNGs RGBA as any other's (two probes,
simulator and device). The bundle's
icons and the partial Info.plist are the plain icon's. Other appearances are
refused (four probes and Chromium's legacy icons, equivalent but for the
appearance blocks' order).

An icon set entry's `expected-size` and `folder`, an icon generator's,
are ignored (one probe, Valdi's).

Building for iOS without `--include-all-app-icons` (refused, as is
`--alternate-app-icon`), Xcode ignores the app icon sets `--app-icon`
doesn't name, and all of them without `--app-icon`: the outputs are as if
those sets weren't there (three probes, rules_apple's catalog with an
alternate icon). Other platforms' are refused.

## App icon sets with an image per size

Each image is a rendition (part 220) of its scale, idiom, subtype and
`dimension2`, a size's rank among all the set's point sizes; per idiom and
subtype a MultiSized Image rendition lists its sizes (in whole points, 83.5
as 83) with those ranks. An iPhone 60pt@3x image is also written as 90pt@2x
for subtype 1792 (the derived size counts in the ranking). Watch icons
take their size's subtype (38mm 320, 42mm 384) as notification-centre and
quick-look icons; the app launcher's and companion settings' are subtype 0.
A single-size watch icon (one universal image for watchOS, 1024 pt, as
Xcode makes it from watchOS 7) compiles as a watch icon of that size and
scale 1, subtype 0: one icon image and a MultiSized Image of the one size,
and the partial Info.plist naming it (three probes and one of the corpus's
catalogs, watchOS 7 and 10).
In BITMAPKEYS a subtype is a bit from a list (0: 0, 320: 8, 384: 9, 1792:
16); size and layer are tracked too.

An icon set's atlases share their scale and idiom with the catalog's
image sets: Xcode indexes them together in the image sets' group order
(translucent colour, translucent grey, opaque colour, opaque grey), the
icons being opaque colour images, and packs the icons together with that
scale and idiom's opaque colour images (12 probes of iPhone image sets
beside an iPhone icon set, and one corpus catalog). This actool does both,
for an opaque colour icon set: where image sets pack images at one of the
icon set's scales and idioms, its icons there join those images' opaque
colour group, next-fit as the icon set's own atlases are (the refusal for
an image fitting a closed atlas doesn't apply), each keeping its icon key;
an atlas of icons alone is encoded as the icon set's own are. Nine
catalogs (opaque, translucent, grey and mixed images, at 2x and 3x, nine
images, iPad and universal ones) are byte-identical to Xcode 27.2's
(tests/actool_catalog, `icon-imagesets-*`). Translucent or grey icons
packed with image sets are refused. Universal image sets are indexed apart
(seven probes).

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
device's (an iPad set for iPhone alone); that is refused.

An iOS icon of all sizes (Xcode's "All Sizes": entries of idiom
`universal` and platform `ios`, any sizes, the 1024 pt one without a
scale) is compiled as an iPhone set and an iPad set of the same entries:
each size for both idioms, the 1024 pt image at 1x among them, and the
iPhone XR's 90 pt @2x from 60pt@3x as above. Its bundle icons aren't
copied: with a 1024 pt image both are resampled from it as a single-size
icon's (Resampling), whatever its other sizes and target devices, each
device's entry listing them as a single-size icon's; without one there
are none. 25 random sets of 3 to 16 sizes, opaque and translucent, with
and without a 1024 pt image, for both devices and (six) each alone, and a
corpus set (`tests/actool_catalog/cases/universal-icon`). A grey icon (every
pixel R = G = B, whatever the PNG's colour type) is stored as GA8, as
image sets' are: GA8 atlases named `S.O.1` (rows 32-byte aligned; an
opaque one's padding is alpha 255, but not the bytes aligning its rows)
and links of pixel format GA8. Icons under 40 pixels are packed too. An
opaque GA8 bitmap is chunked from 8 KiB, one chunk more for every 8 KiB
up to three (one stream at 4,224 bytes, two from 8,192 to 13,888, three
above); a translucent one as a colour one is (one stream at 11,648 bytes).
That is the corpus's two grey icon sets, Lynx's (RGB files) and QReader's
(greyscale PNGs and a colour-mapped one), byte-identical. Sets mixing grey
and colour icons, grey ones for macOS, and a grey icon under 40 pixels
stored on its own are refused. A watch app's partial Info.plist names the icon only. A
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
pixels). Translucent and opaque icons are packed apart, as image sets'
are: at a scale, each opacity's icons fill their own atlases, and an
opacity's only packable icon is stored on its own (Flutter's gallery, its
one opaque iPad icon at 1x standalone beside an atlas of translucent ones).
Atlases of both opacities at one scale aren't established and are refused. An image under 40 pixels wide is stored with method 1, a
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
  grew shows. The same holds through every growth up to 1,000 entries
  (41 catalogs of random names at 51 to 673 entries, three at each
  growth, the 52 and 85 to 88 this comparison left open among them, and 18
  of 126 to 1,000 colour sets); more than 1,000 aren't pinned and are
  refused.
- Per (scale, idiom, subtype), images are taken largest area first, equal
  areas by name identifier. One of more than 18,050 px² × scale is not
  packed; neither is the only packable image of a group.
- Atlases fill next-fit: an atlas takes images while the area it already
  holds is below the cap, then the next atlas begins. That holds where the
  next image would grow the closed atlas; where it would fit in it, Xcode
  sometimes keeps adding a few more images, and how many isn't established
  (23 probes of one shape repeated: 54 x 60 px at 3x takes three more when
  at least three remain and none otherwise, 60 x 54 never more; the
  images' content plays no part). Such catalogs are refused, unless
  `--config=relax_actool_storage` (below). A second
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

### Storage Xcode's choice isn't established for

`--config=relax_actool_storage` (`RULES_APPLECROSS_RELAX_ACTOOL_STORAGE=1`)
accepts, for every image, the storage this actool chooses where the rules
for Xcode's aren't established, rather than refusing the catalog:

- **Atlas splits:** atlases split next-fit where the next image would fit
  in the one before.
- **dmp2:** streams near mode 1's margin, ties between modes, palettes and
  16-bit images in strips, each taken as for a bitmap this actool draws
  from a vector image.
- **Bitmap keys:** bitmap keys in CoreFoundation's order for an identifier
  count whose table growth isn't pinned.

The images and their pixels are the same; only how they're packed or
encoded may differ from Xcode's. Of 300 probe catalogs of equal images,
from 1x to 3x, 296 compile byte-identical with it, among them 82 of the 86
that are refused without it. The four others, tall images at 1x, are packed
otherwise. tests/actool_catalog's RS219 and RS229 are two of those 82:
refused strictly, and Xcode's with the config.

## Paged trees

A tree whose entries don't fit one page (126 at `BITMAPKEYS`' 1024 bytes,
510 at the others' 4096: the block size less 16, over eight) is a B+tree.
Entries are inserted one at a time in the order their blocks are allocated
(above): each into the leaf whose last key is the first not below it (the
last leaf past them all), in key order. A leaf one entry over its capacity
splits, the left keeping the larger half (64 of 127, 256 of 511); the new
leaf is the next block allocated, right after the entry's own, and at the
first split the index page is the one after that. Leaves link forward and
backward (fields 2 and 3 of a page's header). The index page (its header's
first field 0) lists each leaf but the last as (page, its last key), then
the last leaf's page where a leaf's four zero bytes are, then copies of
those keys when the tree copies keys; the tree's header names it as the
root. Reproduced, page by page and block by block, for every tree of 41
probe catalogs of 127 to 1,000 colour sets (`RENDITIONS` and `FACETKEYS`
paged from 511 entries, `BITMAPKEYS` from 127). More than one index page
isn't established and is refused.

Names whose identifiers (CoreUI's hash, above) are alike: Xcode gives the
later in byte order another identifier (45 such pairs in random catalogs;
`dv` keeps its own, `f4` doesn't), by a rule not yet established. Such a
catalog is refused, rather than written with both identifiers alike.

## Sprite atlases

A sprite atlas (`.spriteatlas`, SpriteKit's texture atlas in a catalog)
packs its image sets' images into atlases of its own, explicitly, whatever
their size. Established over 44 probe catalogs (trimming from each side, a
clear image, grey, colour and opaque images, one to three scales, 40 images
of 300 by 300 and one of 3,000 by 10 (both refused now, being past 2048 a side: below), atlases in plain and namespacing folders,
beside plain image sets, and 30 random atlases) and Xcode's four SpriteKit
templates, byte-identical for iOS, macOS, tvOS and watchOS:

- **Its images.** Each image set is read as any other (its facet, its
  rendering intent). Each image is trimmed to the rectangle of its pixels
  that aren't fully transparent; a fully transparent one to its top-left
  pixel. Its rendition (layout 1003) is a link, as a packed image's, at
  its trimmed size: its whole size in `0x3e9`, the trimmed size in `0x3eb`
  and the header, its place in the atlas in `0x3f2` (the atlas's key
  includes the sprite atlas's identifier), and, when trimmed, `0x3f3` last:
  `0x3f3` itself, a zero, its whole width and height, then the rectangle
  kept, from the top left. It keeps its own pixel format, GA8 for a grey
  image.
- **Its atlas per scale** (`ZZZZExplicitlyPackedAsset-<scale>.0.0-gamut0`,
  element 9, part 181, the sprite atlas's identifier): ARGB and sRGB for
  every image, grey and opaque ones too, the opacity and greyness in its
  name always 0. Images are taken largest trimmed area first, equal areas
  by identifier, and packed as image sets' atlases are (the binary-tree
  packer, 2 px in, each side the root's plus one, rounded up to even), into
  one atlas (up to 2048 pixels a side; below). After `0x3e9` it lists its images as an icon
  stack lists its layers (`0x3f4`, `0x3fc` to `0x3fe`), by identifier: an
  empty rectangle, blend 0, opacity 1 and the image's key; no flag, blur or
  fill; no effects.
- **Its list of names** (element 9, part 127, scale 1, its identifier;
  layout 1005, named `CoreStructuredImage`, no size or pixels): `0x3ec`
  (eight zero bytes), `0x3ee` 1, and `0x3f5`, a count, a zero, then each
  image set's name's length and bytes, ignoring case as ASCII lower case
  does (`_x`, `a b`, `a-b`, `a.b`, `a10`, `A2`, `a9`, `alpha`, `Beta`: one
  probe of 13 names). One list, at 1x, for every scale.
- **Its facet:** its name (a namespacing folder's prefix included), element
  9 and its identifier. Its image sets are named with that folder's prefix
  but not the sprite atlas's.

Refused, not established: images other than PNGs, images for an
appearance or an idiom, resizable images, colours beyond sRGB, image set
names other than ASCII, anything but image sets in a sprite atlas, keys in
its `Contents.json` but `info`, an atlas without images, and an atlas that
packs to more than 2048 pixels a side. Past that Xcode splits it into
several atlases (`dimension1` 1, 2, ...), each holding a run of the images
in their order, laid out as an atlas of that run alone would be (Xcode's
grid tile set template: its largest 291 tiles at the places this packer
gives them, then its other 109). Where it cuts isn't established: not after
the longest run that fits (316 tiles there), nor at an area of the tiles,
trimmed or whole; 15 and 16 images of 512 by 512 split, 17 didn't. With
`--config=relax_actool_storage` this actool cuts after the longest run that
fits, so such an atlas compiles, its images packed perhaps otherwise than
Xcode's.

## dmp2

Image sets, and tvOS's and visionOS's image layers, are stored as `MLEC`
(version 2 when opaque, otherwise 0; method 11) wrapping `01 00 00 00`,
`04 00 00 00`, a length and a zero word, then a `dmp2` stream: `dmp2`, a
mode, `01 0a 04`, the width and the rows per strip (16 bits each), and by
mode the strips' data. Image sets and a stack's back layer are opaque when
every pixel is; a stack's other layers when their PNG has no alpha (no
colour type with it, no `tRNS`), whatever their pixels. Modes 2 and 3
compress strips of as many rows as fit 2 MiB, each on its own: in mode 2
their data per row rounded up to 16 bytes (a 1024 px image is four strips,
291 rows each, the last 151), in mode 3 the rows' pixels unrounded (two
corpus images 750 px wide, 699 rows a strip, and a probe 601 px wide; a
16-bit image's mode 3 is unprobed, and refused where rounding would
matter). Each mode-2 strip is transformed as an image of its own. Modes:

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
codes 0 and 2, later rows all five. Each row's cost is the sum of
`ln(1 + |r|)` over its residuals in single precision (`logf`), added value
by value, except that predictor 1's adds each pixel's three values first
and then the pixel. Codes are tried in the order 0, 2, 3, 4, 1, and a row
takes the first whose cost is lowest (4 goes before 1: 119 grey rows where
the two alone tie exactly, one of them a corpus image's; no colour row was
seen to tie them exactly). The summation order is what
decides between rows that 1 and 2 predict alike (common: the row under an
atlas's empty top row). The rule was fitted on 15 such rows, the only
ties left among 9,564 rows of probe and corpus images; it then held for
all 1,769 more in images made to tie after it was written, and for every
row of every mode-2 image decoded: 16,956 colour rows, 3,781 rows of
16-bit renditions and 3,720 grey rows. In double precision, or with
`log2` in single precision (tried before), those ties go undecided, which
kept such images refused.
Apple's `logf` and `ln` rounded to single precision first differ at
|r| = 73,908, beyond a 16-bit residual.

A greyscale image (GA8, below) is the same with 2-byte pixels: the stream
header's last byte and the body header's second word are 2, mode 3 holds
premultiplied grey and alpha per pixel, and mode 2 the alpha plane, the
row codes and one 16-bit residual per pixel (high bytes, then low), the
predictors working on the grey alone. No palette was seen for one, even
of 16 grey-alpha pairs. Grey rows' costs are the same sums of the one
value, so their ties go by the order above.

An image of more than 4 MiB (4 bytes a pixel) is `MLEC` with flags 1 (3
when opaque), method 11 and a count of `KCBC` chunks: each `KCBC`, two zero
words, its rows and its length, then the `01`, `04`, length, zero header and
a `dmp2` stream of those rows alone (mode chosen per chunk). A chunk is as
many rows as fit 4 MiB: 1441 px wide, 727, 727 and 706 rows. A GA8 image is
chunked as its BGRA rows would be (4 MiB of 4 bytes a pixel), the header's
second word 2; its mode 3 strips are 2 MiB of unrounded 2-byte rows, so a
chunk's rows are one strip, and its mode 2 strips are a colour image's
(three probes of over 4 MiB, 1100 to 1501 pixels wide, and Chrome's
862-pixel menu shadow in strips of 809 rows).

Mode 4 has a second form, also a candidate in the competition for the
smallest: at most 256 premultiplied colours without their alpha, the
entry count's companion 3 (not 4) and each entry's fourth byte 0, then the
alpha plane and the index plane compressed together (WebRTC's incognito
banner, 152 such colours where its colours with alpha number 204; 63 of
the examined streams have it).

Pixels are premultiplied as `(c * a + 127) / 255`, after any colour
conversion (below). A palette that would
need strips (over 2 MiB of indices) is not established and is refused.
A palette with alpha apart needs strips sooner, its alpha and index planes
being 2 bytes a pixel: in a full 4 MiB chunk whenever twice the width
isn't a multiple of 16. Splitting can only make a stream larger (a length
more, and less for the compressor to share). So where it is no smaller
unsplit than another stream it can't be chosen, and isn't: a launch
screen's two 1125 x 932 chunks, which Xcode stores as plain palettes, and
14 probes of chunked images, byte-identical. Where it might be smaller,
how Xcode splits it isn't established, and the image is refused. Every layer in the
examples uses mode 4. Where a row's predictor isn't established and mode 2
could be the smallest, the image is refused; so are ties between modes
(each accepted with `--config=relax_actool_storage`, above).
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
4 to 16 pixels a side) compiled byte-identical; 245 more were refused for
their row predictors before their rule above was found, and were not
compiled again. Whether that holds for a stream compressed with LZFSE
(4096 bytes of data on) or a palette's is not established: one within 32
bytes of mode 1 is refused, as is an image that would be stored that way
in strips. Greyscale images are given the same test, unprobed near the
margin.

## Colour-managed PNGs

Xcode converts an image set's or app icon's PNG with an ICC profile other
than sRGB IEC61966-2.1's, GIMP's built-in sRGB, or a grey profile with
sRGB's tone curve (two variants) to sRGB (each of the
corpus's 20 profiles compiled on an opaque gradient differs from the
gradient alone). ImageIO's image drawn into an 8-bit sRGB `CGContext` gives
Xcode's pixels exactly: compiling those pixels untagged gives Xcode's
output for 64x68 images (a 16-level colour cube and a grey ramp) with
Display P3, Adobe RGB, Generic RGB, ITU-709, ITU-2020, ROMM, DCI-P3, ACEScg
and sRGB profiles. A wide-gamut profile adds a 16-bit extended-sRGB
rendition beside the 8-bit one, but only for an image with colours well
outside sRGB (below). A PNG with `gAMA` other than 1/2.2, or with `cHRM`
other than sRGB's beside a `gAMA`, is converted too. `cHRM` without `gAMA`
changes nothing, whatever its primaries (Photoshop's sRGB rounded to
31269, 32899, ..., P3's and odd ones, each against the same image without
it; `tests/actool_catalog/cases/png-chromaticities-alone`), with or
without a profile; nor does an `sRGB` chunk. In the 809-catalog corpus
this made 20 more catalogs byte-identical (Photoshop exports carry such
a `cHRM`).

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
  and patterns, opaque and translucent, 6 to 300 px; 14 more were refused
  for their row predictors, before their rule) and 6 icon sets (a single
  1024 pt image, an image per size, legacy slots).
- The corpus's display profiles and the wide-gamut system profiles, on
  grey, muted, gradient, noise, pastel (also translucent) and saturated
  images, with their 16-bit renditions (below): 59 of 63 image sets, the
  rest refused for their row predictors before their rule.
- Profiles of our own with Generic RGB's colorants and each curve form:
  44 image sets, the rest refused for their row predictors before their
  rule.
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
  refused. A curve within 4.78 of sRGB's everywhere, as a table is
  measured, is taken for sRGB's: Display P3's profile of 2015 (4.19 away;
  today's is 0.27) compiled 7,555 colours exactly as today's does, where
  its own parameters miss 26 of 1,555 colours of a corpus image by one.
  Curves 4.78 to 6.32 away are refused, as tables are. Other types aren't
  established.

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
8-bit atlases with one 16-bit). Image stacks that would need the rendition
are refused, as are macOS catalogs (their key format isn't
established), images of more than 4 MiB as BGRA, and two 16-bit atlases'
order, which wasn't compared byte for byte. Images whose rows' predictor
ties (1 against 2 or 3 with identical residuals) are refused as at 8 bits;
Xcode breaks those either way (176 such rows in the probes), by a rule not
established.

An iOS app icon set's wide-gamut icons get the rendition too, beside each
8-bit one, keyed with gamut 1: an atlas of their own (`ZZZZPackedAsset-
<scale>.<opaque>.0-gamut1`) for each 8-bit atlas, laid out the same and
linked the same, and a 16-bit icon image for each icon stored on its own,
the 1024 pt App Store icon among them. Both are stored as the 8-bit icons
are, the half floats in rows of eight bytes a pixel rounded up to 32,
LZFSE-compressed in one stream or chunks as an 8-bit icon's bytes would
be, not as `dmp2`. The multi-size entries aren't repeated. Observed with
every iOS size, opaque and translucent, from Display P3 and from a profile
of its colorants and curves alone, and a corpus icon set. A set mixing
wide-gamut icons with sRGB, grey or macOS ones, icons packed with image
sets' images, and one stored on its own under 40 pixels (run-length
coded at 8 bits) are refused.

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

For iOS built for both iPhone and iPad, iPhone, iPad and CarPlay colours
keep their idioms beside universal ones (each appearance's of each idiom a
rendition) and watch, Mac, TV and visionOS colours are left out, as image
sets' are (40 random catalogs of one to four sets of one to five idioms
and appearances; a corpus catalog; `tests/actool_catalog/cases/
colour-idioms`). Refused: idioms other than universal for iOS built for
one device only (Xcode thins them by target device: an iPhone build kept
iphone and car and dropped universal), and keys not above. For macOS, see macOS catalogs. In the 350-catalog comparison 13 more catalogs matched:
8 byte-identical and 5 matching but for the appearance blocks' order.

A colour may refer to a system colour (`"reference"`, with platform
universal or ios, or osx, which Xcode compiles for iOS the same way, one
probe): its rendition holds the colour as Xcode resolves it for
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
naming its key in attribute order, numbered by `dimension1` in turn as the
universal ones are: three and four for one idiom and scale (opaque,
translucent and grey) in seven random catalogs, and an iPhone icon set's
two atlases after iPhone image sets' grey one in a corpus catalog. Watch,
Mac, TV and vision images are left out, and so are iPad images at 3x and
CarPlay ones at 1x, scales those devices don't have (a set of nothing else
has no facet, a catalog of nothing else compiles to nothing): each
compiled beside a universal 2x image gave the same output as without it,
every other idiom and scale didn't, and seven visionOS test catalogs built
for iOS were the same without their vision images. Other idioms, device
idioms with other target devices, and atlases for both an appearance and
an idiom are refused (13 probes).

Folders are scanned for image sets: a folder's assets are named as at the
top, or with its name and a slash before theirs when its `Contents.json`
says `provides-namespace` (nested ones too: `Buttons/Small/dot`); the
identifier is the hash of that full name. Other assets in folders, and
other folder properties, are refused. Image sets named alike, in two
catalogs or in folders without a namespace, compile to one, each warned of
as "The image set name "X" is used by multiple image sets." (an allowed
warning). Which one Xcode keeps follows no order of their paths: not the
catalogs' order on the command line, not depth first or by depth, and not
bytewise or case-insensitively by path (17 probes). It is likely the file
system's own enumeration. So image sets named alike compile only when their
files are identical, which makes the choice irrelevant (three probes and a
corpus app's catalogs); otherwise they're refused, as are colour and data
sets named alike (not probed). Files that aren't assets (a README,
with or without an extension, at the top or in a folder) are ignored
(three probes). So are image, colour and data sets without a
`Contents.json` (three probes; an app icon set without one is an empty set,
compiling to nothing even when `--app-icon` names it, one probe) and,
without a word, an image or icon whose file is missing (five probes:
an icon set of only missing files compiles to nothing; one whose 1024 px
image is missing but whose dark and tinted ones aren't is refused). An
iPhone or iPad icon image of another size than its slot's (120x121 for
40 pt @3x, say) is left out without a word too: the output is the same as
without its entry (five probes: larger and smaller images, a size's only
image, the 60 pt @3x image the iPhone XR's icon is made from, and the iPad
bundle icon's; rules_apple's `app_icons_ios_with_bad_size`). Other idioms'
are refused, as is a single-size icon's 1024 px image of another size.

Refused: vector images (SVG, PDF) unless relaxed ([Vector
images](#vector-images)), other properties, and image sets on other
platforms. 90 random catalogs of one to four image sets (opaque,
translucent, single-colour, tiled and noisy images, 20 to 420 px, with and
without an app icon, simulator and device, deployment targets 13 to 18,
with and without `--compress-pngs`) and six images of over 4 MiB were
byte-identical.

### Resizable images

An image's `resizing` (three parts across or down, or nine) crops it to
its caps and centre: the columns of the left cap and the centre's width
after it, then those of the right cap; the rows likewise. Cap insets and
the centre's size are in pixels. The rendition's 0x3e9 TLV then lists
slices in place of the whole image. Three parts across go left to right,
three down from the top, and nine by rows from the top, each row left to
right. Each slice is x, y counted from the bottom, width and height. The
layout is 20 for three parts across, 23 down and 30 for nine, one more
when the centre stretches rather than tiles; the metrics have the
cropped size. A resizable image is packed into an atlas by its cropped
size as any other, its link carrying the slices and, in its `KLNI`, its
layout in place of 12 (16 probes, grey and colour, and Chrome's tab strip catalog, with a
dark appearance). Resizable JPEGs and wide-gamut images, other keys and
caps that don't fit are refused.

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
without properties (two probes).

Other Exif ImageIO writes back in its own form (18 probes, five of the
corpus's JPEGs):
- Big-endian (little-endian Exif is turned round), IFD0 at 8, its entries
  by tag, then its values of more than four bytes in that order, each
  padded to an even length, then the Exif IFD the same way. The input's
  order and layout don't matter.
- YCbCr positioning is dropped, whatever its value. Pixel dimensions
  stored as SHORTs become LONGs. Other entries probed (orientation,
  resolutions and their unit, the Exif and FlashPix versions, components
  configuration, user comment, colour space, pixel dimensions, however
  wrong) are kept as they are. No entries are added: Exif without an
  orientation stays without one.
- Rationals are turned into a value and back into a fraction: 25400/1000
  is 127/5, 144/2 72/1, 2/6 1/3, while 7/3, 1000/3 and 333/1000 stay.
  123456789/1000000 becomes 1355679/10981, a convergent of its continued
  fraction within 1e-9 of it, 72/0 becomes 0/1 and 4294967295/2
  2147483647/1.

This actool writes Exif of those entries so, and the orientation in the
TLV as before. It takes rationals only where every earlier convergent of
the fraction is more than 1e-6 off, so that it is written in lowest
terms. Other entries, a second IFD (thumbnails), zero denominators and
terms past a signed 32-bit integer are refused.

ASCII entries (the image description, make, model, software, artist and
copyright; the modification, original and digitized dates) are written
back as their text up to the first NUL and one NUL: `D90\0\0\0` is
`D90\0`, padded to an even length when stored after the IFD. A date that
isn't one (`ab`) is dropped; this actool refuses it, and text followed by
more than NULs (a copyright's second string).

XMP is dropped, but ImageIO first maps some of its properties into Exif
(about 20 probes): `xmp:CreateDate` becomes DateTimeDigitized (its time
zone dropped), `tiff:Orientation` the orientation, `exif:PixelXDimension`
the pixel width. These are dropped without effect: the creator tool, the
modification and metadata dates, the document and instance ids and
derivation, `dc:format`, Photoshop's colour mode, profile name,
transmission reference, headline, city and credit, and `tiff:Software`.
This actool takes XMP whose mapped properties the Exif already holds with
the same value, and refuses other properties.

ImageIO rewrites a Photoshop (APP13) IPTC record (about 15 probes, two of
the corpus's JPEGs): its character set (1:90) becomes UTF-8 and its
record version (2:00) 2, written first whatever was there; the other
datasets follow in an order of ImageIO's own, not the file's ({2:103,
2:40} as 103 then 40, {2:62, 2:63} as 62 then 63, but all four as 63, 62,
103, 40); the digest (0x425) is recomputed, the MD5 of the new record;
other resources stay in place. A record of one dataset and no header is
kept beside a rewritten one. The record's date and time also become
DateTimeDigitized. This actool takes at most two datasets besides the
header, in the orders established, a date and time only when the Exif
already holds them, only ASCII values, and the record and its digest
alone; it refuses the rest.

ImageIO writes the metadata back in its own order, whatever the file's:
JFIF, Exif, the colour profile, Photoshop's, Adobe's, then the tables (six
probes, two of the corpus's JPEGs). Besides Apple's sRGB profile, the
ICC's 3,048-byte "sRGB IEC61966-2-1 black scaled" and Facebook's 524-byte
compact "c2" are kept as they are (one corpus JPEG each), recognised by
their exact bytes. JPEGs with Adobe RGB or other profiles, which were
re-encoded, are refused, as is XMP that ImageIO merges into a Photoshop
IPTC record already present (a corpus JPEG).

### Vector images

Xcode 27.2, 58 probes of PDF and SVG image sets on iOS and macOS, plus 15
of packing, appearances and rendering intents beside PNG images:

- **No case keeps only the vector.** Each image is stored as a Vector
  rendition (layout 9, flags 4, no size or scale, named after the file,
  pixel format `PDF ` or `SVG `, the composition and 0x3ee TLVs, `DWAR`, a
  flag and the length). It is also drawn into a bitmap at each of the
  platform's scales (1x to 3x on iOS, 1x and 2x on macOS), its size in
  points times the scale:
  - a PDF's sides rounded to the nearest, halves up;
  - an SVG's rounded up, with the bitmap's slice (its 0x3e9 TLV) rounded to
    the nearest, from its top-left corner, so 10.05 points makes an
    11-pixel bitmap with a 10-pixel slice;
  - an SVG under a point wide is drawn as wide as its content, which isn't
    reproduced (fourteen probes, 0.6 to 170.05 points).
- **The bitmaps are a PNG's in all but their flags.** They are packed
  into atlases by the same rules, stored as GA8 for a grey drawing, and
  stored opaque when every pixel is. Their flags are the rendering
  intent's plus 4, and plus 256 when the set preserves the vector
  representation.
- **The Vector rendition's key:** part 181 without a scale; with
  `preserves-vector-representation`, part 42 at scale 1. It takes the
  image's appearance, as the bitmaps do.
- **An SVG** is stored as it is, LZFSE-compressed (flag 1) when that is
  smaller, as an Icon Composer document's SVG layers are.
- **A PDF** is written again through CoreGraphics (flag 0): PDF 1.3, its
  colours in an ICC-based sRGB space, its drawing re-serialized and
  compressed, and a zero document ID and empty info, so it is deterministic.
- **A single image without a scale, or at 1x,** is drawn at every scale.
  Images one per scale are drawn otherwise:
  - a PDF at its own size;
  - an SVG at its size times the scale again, its slice covering its
    size;
  - with only the highest scale's file kept as the vector.

### Exact PDFs

Xcode doesn't store a vector set's PDF: it draws the page into a new one
with CoreGraphics. `src/pdf.rs` replays a page of paths (`m l c v y h re`),
fills, strokes, clips (`W W*`), colours (`g rg k`, their stroking forms
and `cs sc` in Device spaces), line state (`w J j M d`), transforms
(`cm q Q`) and constant alpha (`gs` with `ca` and `CA`), and writes what
CoreGraphics writes for them; its doc comment lists the rules. Against
Xcode 27.2, 495 pages compiled byte-identical: 180 random mixtures of all
of these, 119 written for single rules, and 196 more (numbers near
rounding thresholds, alpha changes, operand wrapping). The other 4 were
values at a formatting threshold, which it refuses.

- Numbers (961 probes): 0 under 1e-6; within 1e-6 of an integer, that
  integer; from 1 up, 7 significant digits without trailing zeros; under
  1, the fewest decimals, from the first significant digit's up to 8, at
  which the value is within 1e-5 of a multiple of their unit, zeros kept
  (`0.0999999999` is `0.10`), else 7 significant digits, zeros kept
  (`0.6939910`). Values within 1% of a threshold are refused, as are
  rounding ties.
- RGB and grey are written in ICC spaces, the Mac's sRGB and Generic Gray
  Gamma 2.2 profiles, which the SDK archive carries
  (`usr/share/colorsync`, from `tools/package-sdks.sh` on the Mac that
  packages it); without them the PDF is stored as it is. CMYK stays
  DeviceCMYK.
- Streams are zlib at level 1 (memLevel 8), as this actool's PNGs are.
- Refused (stored as it is): text, images, shadings, patterns, other
  colour spaces (ICC-based ones included), soft masks and blend modes, a
  page not at the origin or with a crop box, rotation or user unit of its
  own, and more than one page.

Matching the bitmaps needs CoreGraphics' and CoreSVG's drawing, pixel for
pixel, which this actool doesn't reproduce. So vector images are refused
strictly. With `--config=relax_actool_vector` they compile:

- **The bitmaps:** a PDF page or an SVG that only fills paths with
  opaque colours is drawn as CoreGraphics draws it
  (`crates/actool/src/cgraster.rs`; Xcode draws the two kinds the same: a
  PDF and an SVG of one drawing give byte-identical bitmaps). Others are
  drawn by resvg (SVGs) and hayro (PDFs).
  - **Fills:** CoreGraphics' fixed-point scan converter, found black-box
    by drawing paths through CoreGraphics' public API into bitmaps on a Mac
    and reading the pixels back (the research is on the branch
    `coregraphics-fills-research`). It is exact for random edges, wedges
    and rectangles, and ±1 in some rows holding a vertex of triangles and
    curves, so these bitmaps stay relaxed.
  - **Colour and compositing:** a grey or RGB colour becomes sRGB bytes as
    `floor(v · 255 + 0.5)`, and each fill composites over the bitmap as
    `floor((c · a + d · (255 − a)) / 255)` per premultiplied channel, a
    its coverage (384 of 384 channels probed, over clear, opaque and
    translucent pixels).
  - **SVGs in double precision:** CoreSVG keeps an SVG's numbers as
    doubles, so where it can this actool reads the SVG itself
    (`crates/actool/src/svgfills.rs`: groups, transforms, paths of lines
    and cubics, rectangles, circles and ellipses as four cubics, polygons,
    fills and fill rules) rather than from usvg's float32 paths. A circle
    drawn at 2x and 3x then matches Xcode's, as it did at 1x only. Xcode
    draws a circle the same whichever quadrant its cubics start in and
    whichever way they turn (a probe of all eight). Arcs and quadratic
    curves, whose conversion isn't established, use usvg's paths.
  - **Placement:** the page or SVG is drawn from the bitmap's lower left
    corner, so where a side's pixels are rounded up, the slack is at the
    top (pages of 200.5 x 150.75 and 200.25 x 150.5 points, PDFs and SVGs,
    1x to 3x).
  - Of the corpus's 26 vector catalogs, 21 compile byte-identical to
    Xcode's with the config; the others hold strokes, SVG circles, or PDF
    features beyond opaque fills.
- **The PDF:** as CoreGraphics writes it (below, "Exact PDFs") where the
  page is within what this actool replays, else stored as it is, which
  CoreUI reads all the same.
- **The bitmaps' encoding:** where Xcode's choice between valid dmp2
  streams isn't established (tied modes, compressed or not, a palette in
  strips), the encoder takes one rather than refusing, for these bitmaps
  and the atlases holding them alone.
- **Their atlases:** packed next-fit, closed once their area reaches the
  cap, where the next image would fit in the closed atlas. Image sets of
  bitmaps refuse that case, as Xcode sometimes keeps adding a few more
  there. Next-fit gives Xcode's atlases in 296 of 300 catalogs of equal
  images (1x to 3x; the four others tall ones at 1x) and in 158 of 200 of
  mixed sizes at 1x. An app's 23 equal SVG icons, and its whole catalog,
  compile with Xcode's atlases.
- **Every other byte is Xcode's:** each rendition's key, name, flags, size,
  TLVs and storage, and the atlases' layout (tests/actool_vector,
  twenty-two catalogs for the simulator and macOS, and three for tvOS,
  watchOS and visionOS).
- **Supported:** per appearance and idiom (universal, the Mac's on macOS,
  or iPhone's and iPad's on iOS for both devices), one file without a
  scale or one per scale, on iOS, macOS, tvOS, watchOS and visionOS.
  PNGs at 2x or 3x beside one universal 1x vector image take their scales'
  places as bitmaps of their own; beside one without a scale, Xcode
  reports its error for bitmaps beside an image without a scale, and so
  does this actool. Other mixes of vector and bitmap images are refused,
  as are PDFs of more than one page and images under a point.

#### Images one per scale and other platforms

Eighteen probes on iOS, and their sets again on macOS, tvOS, watchOS and
visionOS:

- **The platform's scales:** 1x to 3x on iOS, 1x and 2x on macOS and tvOS,
  2x on watchOS and visionOS. A file at a scale the platform doesn't draw
  is ignored.
- **The Vector rendition** is the file at the highest of those scales
  (part 181, no scale). Preserved, each file is stored at its own scale
  (part 42), the 1x file even where the platform draws no 1x.
- **Each scale is drawn** from its own file, else from the 1x file, else
  from the highest scale's (a set of 2x and 3x SVGs draws its 1x from the
  3x one).
- **A file at scale `f` drawn at scale `t`:** a PDF at its size over `f`
  times `t` (a 32-point PDF at 3x drawn at 1x is 11 pixels); an SVG at its
  size times `t`, Xcode not dividing by `f` (a 64-point SVG at 2x is a
  128-pixel bitmap), with a slice of its size over `f` times `t`.
- **Device idioms** (`iphone`, `ipad`) are keyed by idiom, the Vector
  renditions too; a universal file beside an iPad one is kept for the
  universal idiom.

### Symbol sets

A symbol set's SF Symbols template (`.symbolset`, an SVG made by the SF
Symbols app, template v.3.0), a glyph for each weight and size it has,
Regular-M among them, on iOS and macOS. Xcode 27.2, 19 probe catalogs
(rectangles, tall and wide glyphs, a
glyph off its margin, absolute and relative path commands, curves and arcs,
two paths, two symbol sets, a symbol beside an image set, one 10 units
square, three to five glyphs of six weights and all three sizes), a real
one and the tests' four, deployment targets iOS 13 to 18
and 26 and macOS 14 and 26. The SVG and its metrics are byte-identical; the bitmaps
are drawn with resvg rather than CoreGraphics, so a symbol set is compiled
only with `--config=relax_actool_vector`, as vector images are
(`tests/actool_vector` symbol, symbol-small, symbols-and-image,
symbol-weights):

- **The glyphs.** The template's `Symbols` group holds a group for each
  glyph, named by its weight (Ultralight, Thin, Light, Regular, Medium,
  Semibold, Bold, Heavy, Black: glyph weight 1 to 9) and size (S, M, L:
  glyph size 1 to 3), such as `Black-S`. Xcode requires Regular-M and
  stores every glyph there is as it is: none is interpolated or derived
  from another, and only Regular-M is drawn. The guides are the SF
  Symbols app's: each size's Capline above its Baseline, 70.459 units for
  S and 70.46 for M and L. Regular-M is drawn at the platform's small,
  medium and large sizes: 15, 17 and 20 points at 1x to 3x on iOS; 11, 13
  and 17 at 1x and 2x on macOS. A unit is the medium size over 100 points,
  for every glyph whatever its size.
- **Each glyph's SVG** (element 85, part 59, scale 1, layout 1017, flags 4, scale
  100, pixel format `SVG `): `<?xml version="1.0" encoding="UTF-8"?>`, a
  line feed, then `<svg version="1.1" viewBox="0 0 W H"><g id="s"
  transform="matrix(s 0 0 s tx ty)">` and the glyph's paths, `<path
  d=".."/>` each, then `</g></svg>` and a line feed. W and H are the paths'
  bounds times the unit s, and (tx, ty) moves the bounds' top-left corner
  to the origin. The bounds are of the paths' points and their curves'
  control points (a quadratic's own, and a smooth curve's reflected one),
  not of the curves themselves; an arc counts as the cubics it is drawn
  with, one for each 90 degrees from its start and one for the rest, their
  control points 4/3 tan(a/4) of a radius along its tangents (probes of
  each command, and of arcs of 90 to 200 degrees starting off an axis,
  circular and elliptical). Each command keeps its letter, absolute or
  relative, and its numbers are written as C's `%g` writes them. Stored with `DWAR` as an SVG layer is, LZFSE-compressed when
  that is smaller. Its TLVs: `0x3ec`, `0x3ee` 1, `0x3fa` (its metrics) and
  `0x3fb` (1, 0, 0). The metrics: 3, the medium size, then as floats the
  depth below the baseline times s, its size's guides' distance times s,
  0, then twice 0.5 times s and the medium size times 35/512; then, for
  Regular-M, 3 and each size's index and points (0 small, 1 medium, 2
  large), and for the others 0.
- **Regular-M's bitmaps:** one per size and scale (part 181, `dimension2` 0 to 2 for
  small to large), each `ceil(W x points x scale / medium)` by the same of
  H, computed in doubles. They are grey with alpha (GA8) whatever the
  glyph, flags 4, and packed as image sets' images are, into atlases of
  their own keyed with the deployment attribute 5, beside other symbols'
  bitmaps. Equal areas are taken by identifier, then the smaller size
  first. Their links name the SVG's file and have a layout of 0 as their
  own.
- **Their keys** add the deployment (5), glyph weight and glyph size
  attributes (the bitmaps' 4 and 2: Regular-M's), and the key format orders them as the
  platform's does: on iOS 7, 13, 12, 15, 16, 26, 27, 9, 25, 17, 1, 2; on
  macOS 7, 13, 1, 2, 3, 17, 9, 11, 12, 25, 26, 27. Its facet is element 85,
  part 181, its identifier and the deployment.

Refused, not established: a template whose guides are another distance
apart, two glyphs of one weight and size, glyph elements other than paths
with only their data, attributes on the glyph group beyond a translation,
more than one symbol or an idiom other than universal in a set, keys other
than `info` and `symbols`, tvOS, watchOS and visionOS, symbols beside images
with a `dimension1` or a gamut, and an atlas of symbols past the first.

### Texture sets

A texture set (`.textureset`, a Metal texture: Xcode's Metal Game template
has one) of a single universal mipmap set with only its base level, a PNG,
on iOS and macOS. Xcode 27.2, the template and 10 probe catalogs (16 by 16,
20 by 12, 8 by 8 and 1024 by 1024; each origin and interpretation, no
properties, opaque and translucent) and the tests' two. The description,
keys and storage are byte-identical; the smaller levels and the ASTC levels
are this actool's, so a texture set is compiled only with
`--config=relax_actool_texture` (`tests/actool_vector` texture,
texture-kinds):

- **For each GPU family:** on iOS, for none and for `graphics` 1 and 2 (an
  attribute that follows `dimension2` in the key format and that BITMAPKEYS
  tracks); on macOS, for none. Family 2's levels are ASTC 4x4, the others'
  RGBA8.
- **Its description** (element 41, scale 1, its identifier and family;
  layout 1007, flags 0, the base level's size, scale 0, `BGRA`, colour space
  1, named after the set): `0x3ec`, `0x3ee` 1 and `0x3f6` (0, the
  interpretation: data 0, non-premultiplied colours 1, colours (the default)
  2; then 1 when every pixel is opaque, else 0). Its payload: `RTXT`, 0,
  Metal's pixel format (RGBA8Unorm 70 for data, RGBA8Unorm_sRGB 71 for
  colours; ASTC_4x4_LDR 204 and ASTC_4x4_sRGB 186 for family 2), 1, 1, a
  16-bit 1 and the number of levels, then each level's key: its length, 0,
  the key's attribute and value pairs by attribute, and 0.
- **Its levels** (part 181, `dimension2` the level after the base; layout
  1008, flags 0, the level's size, scale 100, `BGRA`, colour space 0, named
  after the PNG): `0x3ec`, `0x3ee` 1 and `0x3ef`, the bytes in a row (4 a
  pixel, or 16 a block of 4). Each side is halved, rounded down, to 1 by 1.
  Rows are in the PNG's order for a bottom-left origin (the template's) and
  flipped for a top-left one; data is stored as it is, non-premultiplied
  colours as they are but for a colour conversion, colours premultiplied.
  Under 1024 pixels a level is stored raw (MLEC method 0); from there an
  RGBA level as other bitmaps are (one LZFSE stream under 16 KiB, version
  0; chunked from there, version 1), and an ASTC level in one stream.
- **Its facet:** element 41 and its identifier. It doesn't count toward the
  catalog's appearances.

What this actool does instead of Xcode: its smaller levels average the area
each pixel covers, where Xcode's filter isn't a plain average (0, 30, 0 and
30 make 13; opaque alpha comes out 254) and its colours go through a
conversion; its ASTC is a plain encoder (one partition, RGBA endpoints at 8
bits along the block's principal axis, 2-bit weights on a 4x4 grid), where
Xcode's encoder chooses otherwise.

Refused, not established: textures for an idiom or a device, more than one
texture, mipmap levels given rather than generated, cube and array
textures, images other than PNGs, other properties, tvOS, watchOS and
visionOS, and textures beside an atlas's `dimension1`, a gamut or symbols.

## Data sets

A data set's file (one, universal, with its `universal-type-identifier`) is
a rendition keyed as an image set's at scale 1, with a facet as an image
set's: layout 1000, flags 0, no size, scale 100, pixel format `DATA`
(stored `ATAD`), colour space 0, named `CoreStructuredImage`. Its TLVs are
the composition, 0x3ed (the identifier's length with its NUL, 0, then the
identifier and a NUL) and 0x3ee 1, and its payload the file as `RAWD`
(version 0, length, bytes), whatever its size. A catalog of data alone has
no appearance table. Several files or other idioms are refused (eight
probes).

Without a type identifier Xcode records the one the Mac's type database
gives the file's last extension, whatever its case, lowercased (`.drawing`
is `com.apple.drawing`, `.tar.gz` gzip's, `.PNG` `public.png`). That
database holds what macOS and Xcode declare, but also what installed apps
do: on one Mac `.dat` was a game's type. This actool knows the 705
extensions macOS 27 and Xcode 27.2 declare (CoreTypes' and their
frameworks' declarations, each compiled by Xcode and kept where its
answer is that declaration; `data_utis.rs`) and refuses others. A file
without an extension is `dyn.age8u`, the dynamic identifier for an empty
extension: `dyn.a` and the bytes of `1=<extension>` five bits at a time,
most significant first, in `abcdefghkmnpqrstuvwxyz0123456789`
(`dyn.age81u8p4` for an unclaimed `xyz`). `tests/actool_catalog/cases/
data-set-types`.

## macOS catalogs

For macOS (`--platform macosx`), image, colour and data sets compile as for
iOS, renditions and all, but for:
- the key format: appearance, localization, element, part, size,
  identifier, then dimension1 and dimension2 when something uses them,
  layer and scale (no idiom or subtype);
- the appearances, numbered and named as macOS has them: Any 0
  (`NSAppearanceNameSystem`), Dark 1 (`NSAppearanceNameDarkAqua`), high
  contrast 3 (`NSAppearanceNameAccessibilitySystem`), dark high contrast 6
  (`…AccessibilityDarkAqua`), Light 8 (`NSAppearanceNameAqua`) and light
  high contrast 9 (`…AccessibilityAqua`), atlases' links included. The
  table holds those used, System only when an image or colour has no
  appearance (data never has one), and is left out when nothing has one;
  an Icon Composer document's always holds System, Dark, Aqua and
  `ISAppearanceTintable`;
- the idioms: a `mac` image or colour takes its scale and appearance from a
  universal one, and the other idioms' are left out (a set of iPhone images
  alone compiles to nothing).

Xcode takes either of a `mac` and a universal image for one scale and
appearance when one names its scale and the other doesn't, from run to run;
this actool refuses them. An app icon set `--app-icon` names whose images
are all for other platforms is an error, as it is for Xcode ("did not have
any applicable content"); one without images compiles to nothing.

Targeted catalogs (colours in every space and appearance, data, images at
each scale, translucent, grey, template, packed, several atlases, by idiom)
and random ones made after the rules were written, of up to six image,
colour and data sets each: every one of them matches Xcode's (byte for
byte, or but for the appearance blocks' order) or is refused, the most for
Xcode's own error below (Errors Xcode reports), the others for the reasons
above or elsewhere here (dmp2 encodings, data without a type identifier);
the last 150 random ones needed no change. Beside Visor's Icon Composer
document, 112 catalogs: 102 Xcode's (its flattened images aside), 10
refused as above. `tests/actool_catalog` keeps a sample with Xcode's output
(or its error), and `tests/actool_icon` two beside the document.

## watchOS, tvOS and visionOS catalogs

Image, colour and data sets compile for watchOS, tvOS and visionOS as for
iOS, keys and appearance numbers alike, but for:
- the idioms: `watch`, `tv` and `vision` images and colours take their
  scale and appearance from universal ones, as macOS's `mac` ones do, and
  keep their idiom in their key (5, 3 and 8); other idioms are left out;
- the scales: 3x images are left out, whatever their idiom (macOS's too),
  and watchOS's and visionOS's own images at 1x (universal ones at 1x, and
  their own without a scale, are kept). A catalog whose only images were 3x
  ones is refused: Xcode writes an empty Assets.car of a key format of its
  own;
- the appearances: watchOS keeps only images and colours without one;
  visionOS leaves out its own `vision` ones for dark or light (high
  contrast kept), universal ones keeping theirs;
- visionOS's CARHEADER: schema version 1, key semantics 4, as for its
  image stacks.

The appearance table, for these and iOS, holds the appearances images and
colours use, Any only when one has none (a dark image alone gives Dark
alone; this corrects an earlier rule that always added Any).

Probes of each rule, then 220 random catalogs per platform made after them
(the last 120 per platform with 3x images among them): every one Xcode's
or refused, none differing. `tests/actool_catalog` keeps samples for each.

## Errors Xcode reports

rules_apple fails a build on every error, warning or notice actool reports
but for a few it lets through (a 1024x1024 icon for App Store, an image
used by several sets, launch images' deprecation). So where Xcode reports
one, this actool must refuse, or a Linux build would pass where a Mac's
fails. The one found among every probe and random catalog here: an image
set with an image without a scale (Any) and images with one, which Xcode
reports for iOS, macOS and tvOS whatever their idioms and appearances, and
for watchOS and visionOS for some sets only (by a rule not established);
this actool refuses every such set, with Xcode's message. For iOS and tvOS
an iPad or TV image at 3x and a vision image at 1x don't count as images
with a scale: of 80 random sets of two to four images (idioms, scales,
dark or not), Xcode reported the error for exactly the 50 with an image
without a scale and another counting, on both platforms. Other exemptions
the data allows (a Mac or watch image at 3x, a watch one at 1x) didn't
decide any set and aren't assumed. The comparisons
check Xcode's diagnostics (`tools/xcode_equivalence.py`'s
`actool_diagnostics`), and `tests/actool_catalog` keeps Xcode's error for
the cases it reports one.

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

## Sticker icons

A Messages extension's icon set (`.stickersiconset`) compiles only when
`--app-icon` names it; otherwise the catalog is as without it. Named, for
iOS 14 on, Xcode keeps only its 1024x768 App Store image (`ios-marketing`):
an image rendition of the `marketing` idiom (6), its file's name, flags 0
(no template rendering intent, unlike an image set's), stored on its own,
the set's name as an image's facet. The partial Info.plist gets
`MSMessagesExtensionStoreIconName`, the set's name. A catalog of only
the icon has no appearance table; an image set beside it brings one. A set
naming no files, as Xcode's template makes it, compiles to nothing, the
Info.plist key included. iOS 11 keeps three of the images, which this
actool refuses, as it does targets below 14 (four probes, three of the
corpus's catalogs).

## Complications

A watch complication set (`.complicationset`) compiles to nothing for
every platform, its image sets' images too: Xcode 27.2 leaves out
rules_apple's complication set of three circular images for iOS, tvOS,
watchOS and macOS, beside an image set compiled as alone; one whose image
sets name no files, as Xcode's watch app template makes it, gives no
Assets.car and an empty partial Info.plist for a catalog of nothing else
(two of the corpus's catalogs).

## Image stacks

An iOS build leaves tvOS brand assets and visionOS solid image stacks out
unless `--app-icon` names them: the Assets.car is the same as without them
(two probes; a catalog of nothing else compiles to nothing).

Xcode checks every stack first, though, on every platform and whether or
not `--app-icon` names it: a stack listing fewer than two layers is an
error, "The image stack “Icon” must have at least 2 layers." (for visionOS,
"The visionOS App Icon “Solid” must have at least 2 layers."), which fails
rules_apple's build. That covers a stack at a catalog's top level and one in
brand assets that their Contents.json doesn't list. It counts the layers
listed: two without images are no error. This actool reports the same
error (16 probes on iOS, tvOS and visionOS; tests/actool_catalog's
brand-stack-one-layer and solid-stack-one-layer). Xcode also reports an
error when the last layer with content doesn't fill the stack exactly,
which this actool doesn't yet. A stack at the top level with two layers or
more is refused.

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
most icons, including the examples'. Relaxed,
`--config=relax_actool_tvos_radiosity`
(`RULES_APPLECROSS_RELAX_ACTOOL_TVOS_RADIOSITY=1`), it writes them anyway:
the examples' icon is then byte-identical, and of 20 random three-layer
stacks 13 were, the others one value off. What remains to find is how Xcode computes the
cells; the measured cells and probes are described above.
Grey layers (Xcode keeps them GA8), translucent visionOS layers,
other brand asset roles and sizes, and layers small enough to pack into an
atlas are refused.
