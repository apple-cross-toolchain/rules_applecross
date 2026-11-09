# ibtool

Reproduced: `ibtool --compilation-directory DIR ... X.storyboard` and
`ibtool --link DIR ... X.storyboardc`, as rules_apple runs them, for every
storyboard in the examples: CleanRoom's launch screen, `resources/Launch`,
HelloWorldObjC's `Main` (iOS), tvOS's `Main`, the watchOS host's `Main` and
the watch app's WatchKit `Interface`. Compiled and linked, every file is
byte-identical to Xcode 27.0's (toolchain compiler mode, the default).
Linking copies an iOS or tvOS storyboard; a WatchKit one is rewritten
(below). `ibtool --compile X.nib ... X.xib`, as rules_apple's compile_xib
runs it, is reproduced for iOS xibs holding a view (and table view cells
or other views beside it), a lone table, collection or web view, a table or
collection view cell, or a reusable view (below, "Xibs").

Byte-identical is the measure wherever Xcode's own output is. Where it
isn't, Xcode differing from run to run on the same document, this ibtool
makes one fixed output with Xcode's shape: the same objects, classes and
keys, differing only in values Xcode's own runs disagree on (a
compile-time date), or only in the order of arrays Xcode's own runs order
differently (a hash table's order). Two corpus storyboards are matched
this way; the cases are under date pickers and outlet collections.

## Xibs

Xcode compiles an iOS xib to one nib file, whatever the deployment target
(iOS 12 to 26, one probe each). Of 328 distinct iOS xibs found on one
machine that Xcode 27.2 compiles (old projects, examples and libraries,
Interface Builder 3 to 27), 310 compile byte-identical to Xcode's and the
rest are refused; none compile differently. `tests/ibtool_xib` holds 165 cases written for this, checked
against Xcode's digests. An iPad xib (`CocoaTouch.iPad.XIB`) compiles as
any other (one probe).

- **A view.** The xib's File's Owner, first responder and one top-level
  `<view>`: its nib is the view nib Xcode compiles for a storyboard scene
  whose controller is the owner (its class, its outlets and the actions
  aimed at it, and the actions sent to the first responder), with two
  differences. The owner is the nib's own
  `IBFilesOwner` proxy, where a scene's controller is an
  `UpstreamPlaceholder` added after the first responder, so every outlet
  and action comes from or goes to the owner; and the owner's `view` outlet
  is there only when the xib has it, sorted among the others by name. So
  this ibtool rewrites the xib as that one-scene storyboard and compiles
  it, leaving the placeholder out.
- Unlike a scene's view, the view keeps its own frame whatever the device
  from iOS 17.0 on (nine of the corpus's). Below 17.0, a top-level plain
  view (custom class or not), scroll, table, collection or text view
  without a freeform size (`freeformSimulatedSizeMetrics`) keeps its
  frame's origin but takes the device's size, or without a device the
  generic canvas's (600x600, 393x852 without `useTraitCollections`), the
  root view's and the views beside it alike, whatever its frame's size;
  labels, buttons, text fields, image, stack, map, web, picker, activity
  and visual effect views, bars, segmented controls, sliders, cells and
  reusable views keep their frames (eleven probes at iOS 16 and 18, offset
  or not, in either orientation; 30 element types probed at iOS 16; a
  corpus sample at iOS 12 to 16). Other elements, and devices Xcode
  doesn't know, are refused there. The view gets no legacy top and bottom
  layout guides when
  it has no safe area (36), and has a safe-area guide only when it lists
  one, not whenever the document uses safe areas (two).
- Colour components are kept as the document writes them, where a
  storyboard's are rounded to 10 significant digits:
  `0.66666666666666663` is `UIRed-Double` 0.6666666666666666 in a xib's nib
  and 0.6666666667 in a storyboard's (one probe of each; nine of the
  corpus's).
- Where the view sits on the canvas (`canvasLocation`) and the simulated
  bars and screen (`simulated…Metrics`) are the editor's and aren't
  compiled; so are a view's simulated orientation and a split view
  master's simulated size (`splitViewMasterSimulatedSizeMetrics`) anywhere
  (corpus storyboards).
- **A table view cell.** With an owner without connections, a top-level
  `<tableViewCell>` compiles to the nib of a storyboard's prototype cell
  (Table view controllers, below), but for the cell's safe-area guide, an
  unowned `UILayoutGuide` (no owning view, an empty identifier, none of it
  archived) listed after the content view, and after the style's detail
  label when it has one (five probes: plain, default, subtitle and right
  detail cells, with and without the guide). A cell without a reuse
  identifier has no `UIReuseIdentifier` (one probe). The owner's outlets,
  to the cell or its content's views, are the nib's connections, from the
  `IBFilesOwner` proxy, by name (two probes); beside the cell's own
  connections they are refused. The cell's outlets may go to its views'
  constraints too, as a view's (one probe), a collection view cell's
  likewise (two corpus xibs).
- **A collection view cell.** A top-level `<collectionViewCell>` compiles
  as a collection view's prototype cell does (Collection views, below),
  with three differences (and a subview's own safe-area guide is a
  placeholder first in its turn among the objects, as in a view's nib;
  its views' outlets to one another are among the cell's, by name, the
  cell's first where names are equal: two of the corpus's; its
  `fixedFrame` changes nothing, and `translatesAutoresizingMaskIntoConstraints="NO"`
  is `UIViewDoesNotTranslateAutoresizingMaskIntoConstraints` after its
  clipping: one of the corpus's):
  - Its content view is always pinned to the cell by the four
    `UIIBSystemGenerated` constraints from iOS 17.0 on, as a prototype's is
    only when it holds constraints; below 17.0, a xib's is pinned as a
    prototype's is (six corpus xibs at iOS 12 and 16, the 22 with
    constraints alike, and probes at 11 to 18).
  - Its safe-area guide, unowned as a table cell's, follows its content
    view.
  - In the old format (a `<view key="contentView">` without an id), a cell
    with a guide and constraints of its own also archives its content
    view's safe-area guide, with its system constraints, after the content
    view's constraints. The cell's constraints may use the guide: it is
    the content view's, defined where a constraint first uses it, at its
    origin for their order, and as the second item ranked as a view
    deeper than the subviews is, first in its group (five probes and a
    corpus xib).

  A cell's layout margins, then its `insetsLayoutMarginsFromSafeArea="NO"`,
  follow its constraints (two probes), its restoration identifier comes
  just before its semantic content (one probe),
  and its own runtime attributes are key-value pairs, as its views' are
  (eleven probes, and 17 more of the corpus's: every collection view cell
  and transparent table view cell there).
- A table view cell's tint colour is `UITintColor` after its mask and
  clipping (two probes).
- A table view cell's `opaque="NO"` leaves out `UIOpaque` (one probe), and
  its `userInteractionEnabled="NO"` is `UIUserInteractionDisabled` after
  `UIOpaque` (one probe).
- **Cells and views beside the view.** Table view cells and other views
  after a top-level view are in the view's nib, the owner's outlets
  reaching them and their views. They are top-level after the view, in
  document order, the first responder in its own place among them. The
  objects list each root's first listing in turn, then each root's rest in
  turn. The view's first listing runs through its own turn in the
  breadth-first order (its guide, its constraints, its subviews); a cell's
  is its content view, its safe-area guide (after a style's detail label)
  and its content subviews; another view's is its constraints and its
  subviews (a collection view's layout before its guide). Another view, of
  any kind a storyboard compiles, compiles as a subview does, its own
  safe-area guide unowned and first. A custom object there is a class
  swapper for `UICustomObject` listing nothing, the owner's outlets and a
  table's data source and delegate reaching it. A cell's controls'
  outlets and actions on the owner are sorted with the view's (outlets by
  name, actions by mask and selector), and so are the cell's own outlets
  to its views. Its default variations have a trait storage list of their
  own after the view's, one per cell in document order, the cell its
  top-level object, its views in the document's order and its content
  view's subviews' storages twice, as a visual effect view's content's; a
  record whose ranks cross ten is refused. The cell's segues, runtime
  attributes and accessibility there aren't established and are refused
  (eleven probes, 80 of one or two cells with outlets and variations,
  four corpus xibs).
- **A lone table, collection or web view.** Alone in the xib, its owner's
  `view` outlet its own, it compiles as beside a view (the table view and
  collection view controller templates, three probes): this ibtool
  compiles it beside a placeholder view the nib then leaves out, with
  any views and cells after it beside that placeholder too (a corpus xib).
  An empty xib (the Empty template) is its owner and first responder
  alone (one probe).
- The nib's top-level objects and its objects list follow the document's
  order: a first responder placed after the view comes after it there
  too (one probe, two corpus xibs).
- A view keeps its document mask when it doesn't translate it into
  constraints, where a storyboard's has 36 (four probes: labels and plain
  views, in a view and a table view cell, against one storyboard).
- **A collection reusable view.** A top-level `<collectionReusableView>`
  compiles as a top-level view does, a `UICollectionReusableView` (or
  swapped in for one), with its `reuseIdentifier`, even an empty one, as
  `UIReuseIdentifier` after its large content (three probes).
- **Gesture recognizers beside the view.** Top-level recognizers after a
  view are its nib's own top-level objects, after the view in document
  order and listed right after it, with no placeholders: the views'
  `gestureRecognizers` collections hold them, and their actions on the
  owner are event connections without a mask after the outlets, by
  selector (a corpus xib's three, one probe). Beside segues, with outlets
  or segues of their own, or beside controls' actions, they are refused.
- **A lone view controller.** A plain `<viewController>` alone (a launch
  screen's) is a `UIViewController` holding its view as `UIView`, top-level
  after the placeholders and listed right before its view, which compiles
  as a top-level view does; its unused legacy layout guides are the
  editor's (a corpus xib, one probe). One with attributes or connections,
  or beside other objects, is refused.
- **A main window.** A xib with a `<window>` (an old MainWindow.xib) holds
  its window, its custom objects (the app delegate, each a class swapper
  for `UICustomObject`) and one navigation, tab bar or split view
  controller, all top-level after the placeholders in document order and
  listed so. The window is a `UIWindow`: its geometry and background, then
  `UIDeepDrawRect`, `UIAutoresizeSubviews`, its mask,
  `UIShouldNotAdaptToMaterials` true, `UIClearsContextBeforeDrawing` false
  when the document says so, its semantic content and large content
  properties, and `UIResizesToFullScreen` true. The container holds its
  children inline, as a storyboard's container nib does, but each child
  loads its view from its `nibName` (`UINibName`) with no external objects
  or storyboard placeholders: a child's title, tab bar item and navigation
  item (with the bar it is on) come first, then `UINibName`,
  `UIParentViewController`, `UIKeyEdgesForExtendedLayout` (0 for an empty
  `<extendedEdge>`, as on the containers, after their children), and a
  table view controller's `UIClearsSelectionOnViewWillAppear`. The objects
  list each container's parts breadth first as a storyboard's (a split view
  controller's children by relationship, detail first). The connections
  are the controllers' outlets first, then the other outlets in byte order
  of their labels, whatever the labels (four corpus xibs: navigation on
  iPhone and iPad, tab bar, split view; three probes renaming outlets; two
  cases of this repository's). A tab bar's `clearsContextBeforeDrawing="NO"`
  is `UIClearsContextBeforeDrawing` false after its mask (a corpus xib).
- Refused: other top-level objects (a controller beside a view, a cell or
  view before the view), an owner's connections beside a cell alone,
  macOS xibs, and xibs in the archive format of Interface Builder before
  Xcode 5.

## The compiled storyboard

`X.storyboardc/` holds `Info.plist` (binary: the designated entry point,
version 1, view controller identifiers to nib names), one nib per view
controller (`UIViewController-<id>.nib`: the scene's placeholders and a
`UIClassSwapper` standing for the controller, with outlets to the storyboard
placeholder and from the owner) and one per view (`<vc>-view-<view>.nib`:
the view, its colour and safe-area guide, the owner's `view` outlet).

- A controller with a `storyboardIdentifier` has that name instead of
  `UIViewController-<id>`, as its nib, its key and value in Info.plist and
  the entry point; its class swapper records it as `UIStoryboardIdentifier`
  (after `UINibName`). The entry point is then the same plist object as
  the identifier's key (a generated name is written twice). A scene that
  is neither initial nor identified is left out, nibs and all; without an
  `initialViewController` there is no entry point (12 probes of one to
  twelve scenes). Identifiers equal ignoring case, which Xcode renames
  (`1__#$!@%!#__B.nib`) or merges, are refused, as are ones with a slash
  (Xcode fails) or non-ASCII ones (written decomposed).
- A custom class in a module is named as the Swift runtime names it:
  `_TtC`, the module and the class, each after its length in UTF-8 bytes
  (`_TtC3Bär3Foo`), with `s` for the module `Swift`; a dotted class name
  is taken whole (`_TtC3Bar11Outer.Inner`). With a
  `customModuleProvider`, `--module` is the module when given, the
  document's `customModule` otherwise (an empty provider is none). Any
  provider counts, not only `target`: Xcode's file templates leave
  `___VARIABLE_ibCustomModuleProvider___` unexpanded (one probe, 20 of the
  corpus's templates). An empty or missing module leaves the class name
  as written (18 probes).

## NIB archives

`NIBArchive`, format 1, coder 10; four tables (objects, keys, values,
classes), varints whose last byte has the high bit set, and an `LNE\0`
trailer. Observed from Xcode's nibs:

- objects are numbered depth first in encoding order; shared values (one
  empty array, a class name string) are one object;
- each object's values are contiguous, in encoding order; integers take the
  smallest signed width, negative ones 8 bytes (a presentation style of
  -2); rects and points are data: 7 then doubles;
  booleans are type 4 (false) and 5 (true);
- one object per distinct string or number, encoded where first used (the
  hugging priority `{251, 251}`, a font name, `labelColor` are shared),
  in encoding order rather than the order the document gives them: a root
  view's runtime attribute of 0, listed in the nib's key-value pairs at its
  end, refers back to the 0 a later button's states are keyed by (one
  probe); an integer and a double of one value are one number, of the
  type encoded first (a runtime attribute's `<integer value="16">` is a
  font size's 16.0 encoded earlier; five probes); a runtime attribute's
  zero size is one `NSValue` wherever it is used, any other size one each
  (a corpus storyboard and a probe);
- classes are listed in first use, then the fallback classes not already
  listed (UIColor's NSColor, UIFont's NSFont), which the class entry
  references, in byte order: NSColor, NSFont, then UIView for a table
  cell's content view (a set's order gives NSFont first with all three).

The key table's order is a hash table's. Comparing Apple's key tables with
containers filled through public APIs in candidate orders, `NSMutableSet`
filled with keys in depth-first first-use order reproduced all ten nibs
examined (five storyboards); CoreFoundation's CFSet and CFDictionary did
not. foundation-hashing.md describes that set's layout.

## Colour conversion

Xcode converts a colour in single precision through ColorSync, which
CoreGraphics evaluates with vImage (`crates/ibtool/src/colorsync.rs`):
- calibrated RGB to Display P3, then to extended sRGB;
- Display P3 to extended sRGB;
- calibrated white to the gamma 2.2 grey;
- without `colorMatched`, sRGB and gamma 2.2 grey to Generic RGB, clamped
  to 0 to 1; device RGB converts as sRGB does (a corpus xib).

Each step is a curve per channel, a 3×3 matrix and a curve per channel.

- **The parameters.** ColorSync describes each step's curves and matrix as
  float parameters (`ColorSyncTransformCopyProperty` with
  `kColorSyncTransformParametricConversionData`). The curves are ICC
  parametric curves with signed reflection:
  - Generic RGB's and Generic Gray's decoding is gamma 1.8 with a linear
    toe of slope 1/16 below 1/32;
  - Display P3's and sRGB's decoding is the sRGB curve;
  - the encodings are their inverses, the sRGB one as
    `(1.137119·x)^(1/2.4) − 0.055`.
- **The curves.** Each is `vImageSymmetricPiecewiseGamma_PlanarF`'s:
  `(a·|x| + b)^g + e` (a·|x| + b fused) from the boundary up, `c·|x| + f`
  below it, with x's sign. This matches vImage for every float from
  −1.25 to 1.25 on each of the curves above.
- **The matrices.** Each row is its middle product, then the first and
  last terms added with fused multiply-adds. This matches CoreGraphics'
  conversions on 20,000 random colours in both steps.
- **The power function.** vImage computes `x^g` as `exp2(g·log2 x)` in
  floats, values below the smallest normal float flushed to zero.
  - exp2 of t is 2^⌊t⌋ times a degree-6 polynomial of the fraction: fused
    Horner steps, the last product rounded before adding 1.
  - log2 of x is its exponent plus, added in floats, log2 of its mantissa
    reduced to [√2/2, √2). That is r·q(r) with r = m − 1, where q is the
    fused sum of its even and odd parts, each fused Horner steps in r².
    Mantissas from 1 up and below 1 have coefficient sets of their own.
  - The polynomials match vImage's exp2 at all but 9,973 of the
    1,065,353,215 floats in [0, 1), and its log2 at all but 1,050 and
    9,768 of the 8,388,607 mantissas. Those exceptions are recorded as
    vImage gives them (`vimage_pow.bin`, 166 KB). With them, the power
    function matched vImage on 16 million random bases and exponents.

The whole conversion matched CoreGraphics on 20,000 random calibrated RGB
colours and 20,000 greys (CoreGraphics gives opaque white 1, which ibtool
archives by name). Interface Builder's palette, 32 random colours and 36
components without `colorMatched` match Xcode's archived values, in xibs
and storyboards (`tests/ibtool_xib`: `converted-colors`,
`unmatched-converted-colors`, `palette-colors`).

**Provenance.** The parameters and curves come from public APIs:
ColorSync's conversion data and vImage's piecewise gamma. Sampling our own
process's call stack while CoreGraphics converted showed that vImage does
the work. The power function's polynomials were fitted, and its exceptions
recorded, by calling vImage's public functions on constructed inputs in
our own process: exp2 through `2^t` for every float fraction t, log2
recovered from `x^(2^k)` for every mantissa. No Apple binary was
disassembled or decompiled. This runtime probing is in
[README.md](README.md#how-they-were-made)'s allowed sources on this fork
only.

## Supported elements

Scenes whose view controller (with or without a custom class) has a root
view with a frame and autoresizing mask, a background colour that is one of
40 iOS system colours or an sRGB colour, and either a safe-area layout guide or legacy top and bottom layout
guides (in a document using safe areas, a root view that lists neither
has the safe-area guide all the same: four probes; so does one with
legacy guides, beside them, whether it lists the safe area or not, the
safe-area guide following the root view in the objects as without them:
three probes; constraints on that guide beside legacy ones are as without
them: 15 probes on iOS, 15 on tvOS, three corpus storyboards), alone or in a navigation, tab bar or split view controller (and table and collection view
controllers). Subviews: labels (the system font in any weight, bold or
italic, a font Xcode finds for iOS, or the iOS and tvOS text styles; the
default or an sRGB, grey or system text colour), text fields
(any border, alignment, text input traits, a delegate and editing actions on
the controller), text views (text, colours, scrolling and editing flags,
traits, a delegate),
rounded-rect, system and custom buttons with a normal state (title, title
and shadow colours) and touch-up-inside actions on the controller, vertical stack views, image views of named images,
plain views containing any of these, and container views.
Constraints between those, with constants, and labels' and image views' own
width and height; outlets from the controller into its view. Everything else, and
other values of the attributes these use, is refused. Observed:

- Without a `<device>`, an iOS root view is laid out at 600x600 (size
  classes' canvas; 393x852 without `useTraitCollections="YES"`, eight
  probes and RxSwift's ImagePicker) whatever its
  frame says when the deployment target is below iOS 17.0, and at its frame
  from 17.0 on (`resources/Launch`, at 12.0, 16.99, 17.0 and 26.0). A tvOS
  storyboard without one is on the Apple TV, landscape, as if it named it
  (two corpus storyboards); compiled for anything but a TV, Xcode's
  ibtool fails on any tvOS storyboard, and so does this one. A tvOS device
  without an orientation is landscape (a probe and a corpus storyboard).
- Colour components are kept to 10 significant digits (`%.10g`). Each is
  archived as a float (`UIRed`, `UIWhite`, `UIAlpha`, ...) followed, when
  the float isn't exactly that value, by a `-Double` key with it; the text
  form (`NSRGB`, `NSWhite`) is the components as `%.3g` would print them,
  alpha only when it isn't 1 (`0.123 0.988 0.00392 0.3`, `1 1e-05 0.333
  0.8`), for ten probe colours.
- A system colour is archived with Xcode's own values whatever the
  document's copy says (`systemBackgroundColor` given as white 0.333 is
  white 1): white and alpha in the grey colour space (4) for the greys
  (`labelColor`, `systemBackgroundColor`, `lightTextColor`, ...), sRGB (2)
  otherwise, after its `UISystemColorName`. The 40 colours of the system
  colour list measured on iOS are those this ibtool knows, and
  `tableCellGroupedBackgroundColor` (white: a corpus storyboard); others, and
  system colours on tvOS, are refused. A system colour needn't be defined
  in the document's resources, and Xcode 11's forms with a fallback name or
  components beside `systemColor` archive the same (four probes). Documents
  from older Xcodes name
  them with `xcode11CocoaTouchSystemColor` or `cocoaTouchSystemColor`
  instead of `systemColor`; those archive the same (five probes, and
  covid_tracker's and MixTag's launch screens).
- A grey in the generic gamma 2.2 space is archived as given, in the grey
  colour space (4), and named `blackColor`, `whiteColor` or `clearColor`
  (`UISystemColorName` first) when, at 10 digits, it is white 0, 1 or 0
  with alpha 1, 1 or 0; other greys, including 0.5, 1/3, 2/3 and white
  with alpha 0, have no name (17 probes, as text and background colours).
  Calibrated white, calibrated RGB and Display P3 are converted, as
  [Colour conversion](#colour-conversion) describes: calibrated white
  to the gamma 2.2 grey (0.5 is archived as 0.572), calibrated RGB and
  Display P3 to extended sRGB. The converted components are floats,
  archived without `-Double` in storyboards too, and alpha is kept (84
  components of Interface Builder's palette, opaque and not, in a xib
  and a storyboard, and 32 random colours). Calibrated white 0 is
  archived as 0; 1 as `whiteColor` when opaque and otherwise as the float
  after 1 (1.0000001192092896, text `1`), named as the grey would be
  (eight probes). `colorSpace="custom" customColorSpace="calibratedRGB"`
  is the same space. Device spaces are refused.
- A document without `colorMatched="YES"` (older ones) takes calibrated
  colours' components as they are instead of converting them:
  calibrated RGB is archived as sRGB with the document's values,
  calibrated white as sRGB grey (the grey as a float, the alpha not).
  sRGB and gamma 2.2 grey are converted to Generic RGB and clamped to 0
  to 1 (sRGB 1, 0.5, 0.25 as 0.989, 0.413, 0.195; [Colour
  conversion](#colour-conversion)), then archived as sRGB with those
  components, a storyboard's rounded to 10 digits as the document's
  are; none are named (36 probes, a xib and a storyboard; the corpus's
  16 such storyboards).
- Equal colours are one object, compared after rounding to 10 digits
  (0.18039215686 and 0.1803921568627451 share), sRGB and grey alike; a
  named colour is shared only by name (`whiteColor` and a white
  `systemBackgroundColor` stay two). Equal fonts are one object too, system
  fonts and text styles included.
- The system font's weights are faces of SF UI: ultraLight
  (`.SFUI-Ultralight`, usage `CTFontUltraLightUsage`, traits 16), thin
  (Thin, `CTFontThinUsage`, 4), light (Light, 8), regular (Regular, 0),
  medium (Medium, 0), semibold (Semibold, `CTFontDemiUsage`, 2), heavy
  and black (2). Bold, and `boldSystem`, is `.SFUI-Bold` with usage
  `CTFontEmphasizedUsage` and `NSCTFontUIUsageOverrideAttribute`
  `CTFontBoldUsage`, in an `NSMutableDictionary`. `italicSystem` is
  `.SFUI-RegularItalic`, `CTFontObliqueUsage`, traits 1, with an
  `NSCTFontTraitsAttribute` of symbolic trait 1 first. The iOS text styles
  are `.SFUI-Regular` (Headline `.SFUI-Semibold`, traits 2) with `NSSize`
  Body 14, Subhead 12, Footnote 12, Caption1 and Caption2 11, Title0 31,
  Title1 25, Title2 19, Title3 17, Headline 14, Callout 13 (23 probes).
  tvOS's are `.SFUI-Medium` (Footnote `.SFUI-Regular`), none bold, with
  `NSSize` Headline 38, Body 29, Callout 31, Subhead 29, Footnote 29,
  Caption1 25, Caption2 23, Title1 76, Title2 57, Title3 48 (one
  storyboard labelling each).
- A label's declared text colour, even `labelColor`, sets
  `UIDisableUpdateTextColorOnTraitCollectionChange`; a nil one is
  `labelColor` without it. Its tint colour is `UITintColor` after its
  content mode (a corpus storyboard); beside clipping it is refused.
- A view gets `UIViewDoesNotTranslateAutoresizingMaskIntoConstraints` only
  when its element says `translatesAutoresizingMaskIntoConstraints="NO"`
  (labels, a text field, a button and a stack view, with it removed) and a
  constraint is on it: as an item, by its own constraints, as a stack
  view's arranged subview, or as a stack view with arranged subviews
  (two probes). A label that says so but has no constraint doesn't get
  it.
- A font named in `<fontDescription name=...>` is resolved from a set of iOS
  fonts that is neither the Mac's nor the simulator's: of the 1,182
  PostScript names macOS 27 lists (CoreText) and the 304 the iOS 27
  simulator's `UIFont` lists, ibtool finds 301, including Rockwell (not in
  the simulator's list) and not NotoSerifMyanmar (in it) or AndaleMono (on
  the Mac). `family` is ignored. A found font is archived under its own
  name (`STHeitiSC-Light` as `PingFangSC-Light`), with the descriptor's
  `NSFontNameAttribute` and `NSFontSizeAttribute`, options 0x80000000,
  `UISystemFont` false, and its traits (italic 1, bold 2: `Avenir-Black` is
  bold; Courier, missed by that measure, is found as in one of the
  corpus's); `.AppleSystemUIFont` is the regular system font (one of the
  corpus's); fonts.rs has them all, measured from one storyboard labelling
  each. A name it doesn't find is archived without the scaling keys and
  with options 0, its traits 0 and `UISystemFont` false (seven corpus
  files: OpenSans, OpenSans-Bold, Inter-Regular, Montserrat-Regular; one
  probe). A name installed nowhere is one it doesn't find; one installed on
  the Mac or in a simulator that it doesn't find is archived otherwise
  (Courier, in the simulators, keeps the scaling keys and options
  0x80000000; `.AppleSystemUIFont` is the system font) and is refused, as
  are names starting with a dot. Installed means in the list of every font
  of macOS 27 (its downloadable system fonts included, 1,605 names) and the
  iOS 18.4, 18.6, 27.0 and 27.2 simulator runtimes' font files, read with
  CoreText (`CTFontCollectionCreateFromAvailableFonts`,
  `CTFontManagerCreateFontDescriptorsFromURL`): fonts.rs's
  `installed_fonts.txt`. `--auto-activate-custom-fonts` doesn't find a
  font file beside the storyboard (Rubik-Bold).
- Legacy layout guides become hidden `_UILayoutGuide` subviews, after the
  others, each with a `_UILayoutSupportConstraint` (width, priority 999),
  `_UIViewControllerTop`/`Bottom` and three constraints pinning it (top:
  leading, top, height; bottom: leading, height, view bottom to guide
  bottom), which the root view's constraints list first. A root view with
  neither a safe-area guide nor legacy ones (the document not using safe
  areas) gets the legacy guides as if listed when the document uses Auto
  Layout (three probes, one of the corpus's), and no guides without it
  (one of the corpus's). There a subview without an autoresizing mask has
  mask 40, not 36 (one probe), which this ibtool refuses.
- The document's constraints follow, whatever their order or ids in the
  document. They are grouped by item: a constraint with the root view by
  its other item, one between siblings (subviews, the safe-area guide) by
  its first item. Groups go by where their item is: its frame's origin by
  distance from the root view's origin, then by x, then by id (byte
  order), the safe-area guide counting as at the origin on every iPhone
  (seven devices, by bisection), so a subview at (0, 0) whose id sorts
  before the guide's comes first (two image views at different heights,
  labels at one frame told apart by id, and probes that tell the distance
  from x + y, the centre and y then x). A view nested in plain views, with
  the root view as its constraint's other item either way round, goes by
  its origin in the root view's coordinates (four probes; a label or text
  field nested so goes the same way, two probes and one of the corpus's,
  as does any other view, its origin alone counting: buttons, image, text
  and collection views in four of the corpus's,
  and so do views in stack and scroll views, their document frames summed
  whatever the scroll view's content, five probes and one of the corpus's;
  one whose root subview is resized where it is laid out is refused). A
  container view nested so goes the same way (40 random probes in plain
  and stack views, and two corpus storyboards), and so does a stack view
  nested in a scroll view, by its document frame (two corpus storyboards). But Xcode compares two
  views nested in one of the root's subviews by their places in the
  nearest view holding both, and other pairs by their places in the root
  view: two children of a view at (33, 249), at (4, 77) and (58, 60),
  come in that order although the second is nearer the root's origin,
  and a sibling of their parent at (0, 324) comes before both, at
  (0, 330) after both. That comparison needn't be an order, so where the
  two places of two such views disagree the storyboard is refused (13
  probes). In a group, constraints whose
  second item is deeper than the root's subviews come first, the deepest
  first, then by first and second attribute in the siblings' order below
  (22 probes: depths two to four, from the safe-area guide or a sibling,
  attributes and document order against depth; and a corpus storyboard's
  seven from its safe-area guide),
  then the constraints with the root view, by kind and then by first and
  second attribute: those with a horizontal centre (either attribute,
  `centerXWithinMargins` before `centerX`), then a vertical centre, then
  between left and right edges (right margin, left margin, right, left),
  then horizontal edges with a leading or trailing one (trailing margin,
  leading margin, trailing, leading, each tied with its right or left
  counterpart; between a leading and a trailing edge, the one whose root
  view's edge is the trailing one first), then the rest (width, top margin,
  bottom margin, bottom, top, first baseline, height), among which a
  subview's width or height to the root view's other dimension goes by the
  same ranks (16 random probes; with the root view as the first item Xcode
  leaves such a constraint out, twelve probes, so it is refused). Measured with all
  272 combinations of horizontal and vertical edges and centres, both ways
  round, between the root view and a subview at once, in three document
  orders and a random subset, 40 random sets of eight, and 30 more of ten
  (ten in a nested view, whose own constraints go the same way). A left or
  right edge tied to a leading or trailing one is refused: Xcode rewrites
  it (a right margin to a trailing edge becomes a trailing margin). Then
  those between siblings, by first and
  then second attribute in centerX, trailing, leading, centerY, bottom,
  top, width, height. Measured with 176 constraints on two labels, the
  root view and the safe-area guide at once, in three document orders,
  eight random subsets and some 30 smaller probes; legacy guides as the
  second item of a sibling constraint fit too. Constraints with equal keys
  (the same first item and pair of attributes, different second items)
  go by the length of the line Interface Builder draws for each, longest
  first, whatever the ids and document and subview orders (380 probe
  scenes of random frames, constants and ids):
  - An alignment (the same edge or centre both ways) without a constant,
    or any constraint whose two edges meet in the frames, is drawn along
    the edge across both items: its length is their frames' span across
    it (centerY: from the leftmost left edge to the rightmost right one).
  - Any other is drawn between the two edges: its length is the distance
    between them in the frames, whatever its constant.
  - The safe-area guide measures as an empty rectangle at the root view's
    origin (80 probe scenes on two devices; its laid-out frame fits half
    of them), the top legacy guide as the root view's width and a point
    tall at its top, and the bottom one as its width and no height at its
    foot (240 probe scenes on three devices, placed to a quarter point).
  - A view without a frame measures by its laid-out frame (1000x1000 at
    the origin, see subviews: Xcode's Action Extension template).
  - Equal lengths go by where the line is, the one further down or right
    first: a line along an edge by the middle of its span, one across a
    gap by the middle of where the two frames overlap; equal places, and
    gaps whose frames don't overlap (the safe-area guide's among them), go
    by id. Centres go the lower one first, then the one further right (19
    probes). Lengths compare rounded up to a whole point: a gap of 49.4 to
    49.6 ties with one of 50, and 50.4 to 50.6 comes before it; an edge's
    line a quarter or half a point longer comes first (twelve probes).
    Lines of different kinds that tie are refused, as are edges less than
    a point apart (a centre half a point from another was drawn along the
    edge). Twelve probes and 19 corpus files, whose orders were read by
    compiling each order of the tied constraints against Xcode's output.
    In a subview's own constraints its safe-area guide measures as the
    root view's does, an empty rectangle at the origin, and a view nested
    in a subview by its frames added up (two of the corpus's).
  - Equal widths or heights are drawn between the two views' centres: the
    farthest first, equal distances by id, multipliers changing nothing
    (three corpus files and four probes). The same two items twice or
    more, either way round, whatever their frames and constants, go by id
    (four corpus files, two probes, and 60 of a view and a subview it
    holds two to four constraints between, on one edge, with random
    constants and ids).

  - Before the length come the priority, lowest first (required ones
    last), and the relation: less than or equal, greater than or equal,
    then equal (100 probe scenes). The same items and attributes twice or
    more tie as any others do: second items with the same frame, so equal
    lengths go by id (100 more).

  Multipliers, margins, baselines and widths or heights against another
  dimension in such ties weren't probed and are refused, as are constraints from a legacy guide (but the bottom guide as the first
  item of a sibling constraint, placed at the root view's foot, three
  probes) or between one and the root view
  (their frames aren't in the document), constraints between the root
  view and a nested view, between axes, and between a width and a
  height (Xcode drops some of those).
- Margins, baselines, relations, priorities and symbolic constants (two
  probes of every attribute against the root view and a sibling, and 28
  smaller ones): the rank orders above extend to centerXWithinMargins,
  centerX, centerYWithinMargins, centerY, rightMargin, leftMargin, right,
  left, trailingMargin, leadingMargin, trailing, leading, width,
  topMargin, bottomMargin, bottom, top, firstBaseline, height with the
  root view, and centerXWithinMargins, centerX, rightMargin, leftMargin,
  right, left, trailingMargin, leadingMargin, trailing, leading,
  centerYWithinMargins, centerY, topMargin, bottomMargin, bottom, top,
  firstBaseline, baseline, width, height between siblings (the last
  baseline, `baseline`, is code 11; three probes, one of 20 constraints
  between vertical attributes; with the root view it is refused). A margin's
  `NSFirstAttributeV2`/`NSSecondAttributeV2` is its own code (leftMargin
  13 ... centerYWithinMargins 20) and the legacy key its edge's; the
  legacy `NSConstant` moves by the item's margin on that edge, the layout
  margins its document gives it (leading as left and trailing as right,
  plain or directional) or else 8: plus for a leading, left or top margin
  as the second item, minus as the first, the other way for trailing,
  right and bottom, nothing for centres (ten probes: root and nested
  views, either kind of margins, either attribute naming, margins on the
  subview). A centre within uneven margins is refused: Xcode moved one
  by 2 where half the difference was 1.5. `relation` is
  `NSRelation` (greaterThanOrEqual 1, lessThanOrEqual -1) after the first
  attribute, before `NSMultiplier` (an old document's `type`, `user` or
  `default`, changes nothing: one of the corpus's); a `symbolic` constant is
  `NSSymbolicConstant` `NSSpace` before `NSShouldBeArchived`, the constant
  itself counting as 0; `priority` is `NSPriority` after
  `NSShouldBeArchived` and before the legacy constant. Xcode compiles
  `lastBaseline` against a view as no attribute (0): refused.
- The nib's objects: the placeholders, the root view, the safe-area guide,
  its document constraints, the legacy guides, then breadth first: each
  view in turn from a queue lists the constraints it holds, then its
  subviews, which join the queue (six probes nesting views two deep beside
  image views with their own constraints; depth first and level by level
  each fail one).
- A subview's constraints may name, as their second item, a view deeper
  than its own subviews (in plain, stack or scroll views): ordered as the
  root view's nested ones are, first in their group. A deeper view with
  the holder itself, either item, is grouped by its place in the holder,
  as the root view's are (eight probes, plain, stack and scroll holders,
  and two corpus storyboards). In the breadth-first walk a holder's
  constraints take a turn of their own, between its constraints and its
  subviews: it lists the views they name that aren't listed yet, in their
  constraints' order, and those join the queue there. That is right before
  the turn of the holder's first subview, unless the root's constraints
  pulled that subview up already, whose own turn then listed them first;
  the root view's are its level's, as above (25 probes: depths two to
  four, siblings before and after, views of other branches between, two
  holders side by side, plain, stack or scroll views, either item; and
  four corpus storyboards). Twelve of those fit the earlier reading, after
  the last turn of the holder's level, which two holders side by side
  disprove.
- A constraint a view's default variation excludes (`<variation
  key="default"><mask key="constraints"><exclude reference=…/>`,
  Interface Builder's Installed unchecked) is left out of the view's
  `UIViewAutolayoutConstraints` (none if it was the only one; excluding
  an item's own aspect ratio that the view holds changes nothing, it being
  the item's, and with nothing else excluded there is no variation: two
  of the corpus's) but keeps
  its place in the objects, where it is archived; an outlet to it is as to
  any constraint. The safe-area guide is then archived where an installed
  constraint first uses it, or else in the view's `UIViewLayoutGuides`.
  The nib's `UINibTraitStorageListsKey` holds a `_UITraitStorageList`:
  `UITopLevelObject` the root view; `UITraitStorages` an array of a
  `_UIRelationshipTraitStorage` per view with an exclusion, in the
  document's order depth first (not the objects' breadth-first order):
  `UIObject` the view, `UIKeyPath` `traitStorageConstraints`, `UIRecords`
  a mutable array of one `_UIRelationshipTraitStorageRecord`
  (`UITraitCollection` an empty `UITraitCollection`, one for the nib;
  `UIAddedObjects` an empty `NSSet`; `UIRemovedObjects` an `NSSet` of the
  excluded constraints) and `UIRelationshipCandidates` an `NSOrderedSet`
  of all the view's constraints in the objects' order; then
  `UIDescendants`, an `NSSet` of the items those constraints relate, but
  the root view, in the order they first appear across the storages. The
  safe-area guide is there as its placeholder in the objects, a legacy
  guide as itself; a legacy guide's own constraints aren't candidates.
  Sets are keyed `NS.object.<i>` and not inlined. The removed constraints
  go by places counted from 1 and compared as decimal strings (the 10th
  before the 9th), in the objects with each run of constraints moved
  after the run of other objects that follows it (a view's constraints
  after its subviews) and, within a run, each view's installed
  constraints before its excluded ones. 198 probes, among them views
  nested two deep with exclusions at several levels, size and aspect
  constraints, legacy guides and outlets, and 150 with several exclusions
  in one view, whose order is a rotation of the objects' order where those
  places cross 10 (a colour, another scene or other ids don't move it;
  more views or installed constraints do); and a corpus storyboard. This
  is not the order of key-value pairs, which go by the objects' own places
  (eight probes with runtime attributes behind root constraints).
  A size class's variation of a view's constraints (`<variation
  key="heightClass=compact"><mask key="constraints">` including and
  excluding them) or subviews (`<mask key="subviews">` excluding some)
  adds records after the default's: one per size class, those of one
  trait first, each group sorted by the variation's key
  (`heightClass=compact` before `widthClass=regular`, both before
  `heightClass=regular-widthClass=compact`), whatever the document's order
  (30 probes of three to five classes), each its own `UITraitCollection`, its
  added objects the constraints it includes and its removed objects those
  it excludes, those on the subviews it leaves out and, for a subview left
  out, all of the subview's own (a storage of its own). A view leaving
  out subviews has a `traitStorageSubviews` storage first, its candidates
  all its subviews in document order, a record per size class removing
  them; its descendants begin with the view itself. Placeholder
  constraints are in no record, but a size class whose mask names only
  placeholders still has its record, empty, as does a default variation
  excluding only placeholders; a placeholder's own variations (a size
  class's constant) compile to nothing (four probes, a corpus storyboard).
  The removed and added constraints go by decimal strings: withheld ones
  by those places, installed ones by their places in the objects counted
  from 1 without the withheld ones (438 sets of two to seven constraints
  in 150 probes, a root view's variations and a root view's and a
  subview's, the device in or out of their size classes, and a corpus
  storyboard). In a table's footer and a xib's cells beside its view, a
  set whose keys cross ten is refused (three of 180 probes differed
  there). When the document's
  device has one of the size classes, the nib archives the device's state,
  that class's included constraints installed and its excluded ones
  withheld, the records unchanged (two probes); the device having several,
  or leaving out subviews, is refused (one probe). Eleven probes and two
  corpus storyboards, a prototype cell's among them, whose nib then has
  its trait storage list as a xib's cell's does. Variations for size
  classes are refused but for those and attributes' (below), as are
  variations anywhere but in a plain controller's view nib, a table view
  controller's table's header and footer views, a xib's cell's nib or a
  prototype cell's. A table's are as a plain controller's, the table the
  top-level object, but its footer's storage comes before its header's, as
  the table lists them, and a footer's record whose constraints' ranks
  cross ten is refused (100 probes of a header's and a footer's
  variations, and a corpus storyboard's headers). There the cell is the trait storage list's top-level object,
  and each view's excluded constraints are encoded where the objects list
  them, in their places among its others (a corpus xib and a probe);
  several excluded from one view are refused. A visual effect view's content view counts among the views in
  the document's order, and its subviews' storages come twice, in order
  and then again right after them (two corpus xibs); varied views after
  them are refused.
- A size class's variation of a stack view's `axis` or a label's font is a
  `_UIAttributeTraitStorage` in the view nib's trait storage list:
  `UIObject` the view, `UIKeyPath` `axis` or `font`, `UIRecords` the size
  class's `_UIAttributeTraitStorageRecord` (a `UITraitCollection` of
  `UITraitCollectionBuiltinTrait-_UITraitNameHorizontalSizeClass` then
  `…VerticalSizeClass`, compact 1 and regular 2, and its `UIValue`), then
  the default's (the shared empty traits and the default: the shared
  number, or the default font object); `UIDescendants` the varied views
  (two corpus storyboards and a xib). A storyboard archives the value of
  the variation its device has (an iPhone compact and regular in portrait,
  a small one compact both ways in landscape, an iPad regular both ways,
  nothing without a device: seven probes); a xib the default, whatever its
  device (two probes). Several size classes of one attribute, several
  attributes of one view, the root view's, other iPhones in landscape and
  attributes beside constraints' variations are refused. A constraint's
  constant for a size class (`<variation key=… constant=…/>` inside it) is
  an `_UIAttributeTraitStorage` of the constraint, `UIKeyPath`
  `encodedConstant`, its values `_NSLayoutConstraintConstant`s
  (`UISymbolicConstant` nil, `UINumericConstant` a double); Xcode lists
  that storage twice, each with objects of its own, and the descendants
  hold the constraint (a corpus storyboard). When the device has the size
  class, for several, or beside other variations it is refused.
- An older document's frame inset in percentages (`minX="50.00%"`) puts
  the view at that share of the room its superview leaves it, `(superview
  − size) × percentage` (a corpus storyboard's centred stack view).
- An Xcode 4 document (`version="2.0"`) without a device is on the generic
  canvas whatever the deployment target (393x852 without trait
  collections), and its table view cells' content view is a `<view
  key="contentView">` without an id, compiled as a content view whose
  background colour Xcode leaves out (a corpus storyboard); with
  constraints it is refused. On the generic canvas an iPad document
  (`targetRuntime="iOS.CocoaTouch.iPad"`) is the default iPad's 820x1180.
  One Xcode 5 saved for iOS 6 and earlier (`variant="6xAndEarlier"`) gets
  no implicit legacy layout guides, and a rounded rectangle button's
  normal title colour there is left out when it is that button's old
  default blue (0.196, 0.310, 0.522; two corpus storyboards, iPhone and
  iPad); other variants are refused.
- A xib deploying before iOS 8 (`<deployment version>` below 0x800) is
  compiled, for each target device, into a nib directory beside the nib
  asked for (`out~iphone.nib`, `out~ipad.nib`; with one device only its
  own, the same bytes; with none, nothing). Each holds `objects-8.0+.nib`,
  the xib's nib as from iOS 8 with its size classes' variations, and
  `runtime.nib`, the same with the variation of the idiom's size classes
  archived (an iPhone's compact width and regular height, an iPad's both
  regular), a label's minimum scale factor from that variation's font, and
  no trait storage list; without variations the two are equal. The iPhone's
  nibs are otherwise the single nib's. The iPad's archive each system
  colour as sRGB components, not by name: `darkTextColor` black,
  `lightTextColor` and `tableCellGroupedBackgroundColor` white (alpha 0.6
  and 1) a float's step off in each component (0.99999994, 1.0000001,
  1.0000001), `groupTableViewBackgroundColor` (0.94901973, 0.9490196,
  0.9686275), and every newer colour, a label's default `labelColor`
  among them, one blue (-5.4925334e-07, 0.45999992, 0.89), all floats
  without doubles (probes of every system colour above and of the
  defaults, `tests/ibtool_xib/cases/before-ios8-*`, and a corpus xib).
  Other colours Xcode names (white, black, clear), iPad xibs, cells and
  main window xibs deploying before iOS 8, and storyboards are refused.
- A storyboard in a `Base.lproj` directory has its translatable strings
  archived as `NSLocalizableString`s: the string inline (`NS.bytes`),
  `NSKey` `<id>.<property>` and `NSDev` the string (shared as strings
  are), for a label's `text`, a button's `normalTitle` (and
  `highlightedTitle`, `selectedTitle`, `disabledTitle`: a probe of each), a text field's
  `placeholder` and a controller's or navigation item's `title` (whose
  attributed title holds the same object). Elsewhere, `en.lproj`
  included, they are plain strings (five probes, two corpus storyboards).
  An empty string is never localizable: it is the nib's shared empty
  string, and an empty navigation item title is left out with its
  attributed title (ten probes, one corpus storyboard).
- A button: a system button is archived as a rounded-rect one
  (`UIButtonType` 1), a custom one has no `UIButtonType` and an 18-point
  default font (15 otherwise); `fixedFrame` changes nothing. A background
  colour follows `UICenter`, `UIOpaque` `UIDeepDrawRect` unless
  `opaque="NO"`, a tint colour the mask, its own constraints the
  autoresizing key, a font replaces the default; the normal state's
  `UIButtonContent` holds `UITitle`, `UITitleColor`, `UIShadowColor` in
  that order (twelve probes). The system image types are `UIButtonType`
  detailDisclosure 2, infoLight 3, infoDark 4 and contactAdd 5, without a
  font, their hugging written unless 750 both ways (missing counting as
  250, so `{250, 250}` when not given: nine probes); their states, which Xcode drops (a title: one probe), and a
  font are refused.
- A multiplier other than 1 is `NSMultiplier` after `NSFirstAttribute`: a
  double holding the float nearest it (`1.2` as 1.2000000476837158, `2:3`
  and `2/3` as 2/3), four probes.
- A subview's constraints are its `UIViewAutolayoutConstraints`, after
  `UIViewDoesNotTranslateAutoresizingMaskIntoConstraints`: its own width
  and height constants (no second item) and aspect ratios (the view
  itself as the second item, width to height or height to width): width,
  width ratio, height, height ratio, whatever the document's order. Then,
  for a container view, those between it and its subviews or among them,
  ordered as a root view's are, by its subviews' frames in it. A ratio is
  a two-item constraint with its multiplier and constant as any other
  (17 probes, on labels and containers). A text field's are in the same
  place as a label's (five probes), followed by its hugging when either
  priority is given (each 250 unless given; two probes). An aspect ratio
  the root view (or any superview: a corpus storyboard) holds on one of
  its subviews is the subview's own, in its constraints and in its turn
  among the objects (one probe). Two aspect
  ratios on one dimension, its own and one the root view holds, go by id
  (three probes and one of the corpus's). A subview's own constants on
  one dimension go by priority, lowest first, then relation (less than or
  equal, greater than or equal, then equal), then id (five probes, and 120
  of one to four constants on each dimension with random priorities,
  relations and ids).
- A plain `<view>` subview archives its subviews, background colour, the
  flags, its autoresizing mask (36 without one), the autoresizing key
  (left out when it doesn't autoresize its subviews: one probe) and its
  constraints. With a custom class it is a `UIClassSwapper` with the
  same values, then `UIClassName` (the runtime name, as a controller's)
  and `UIOriginalClassName` `UIView`. A text field with a custom class is
  swapped in the same way, `UIOriginalClassName` `UITextField` (three
  probes: a module, none, and the target's). A button's
  `UIViewInsetsLayoutMarginsFromSafeArea` 0, when turned off, comes after
  its compression resistance, before its semantic content attribute (two
  probes).
- A `<device>` without an orientation is portrait (one probe). Without a
  device, a root view without a frame has the frame a subview without one
  has (below); an image view with `opaque="NO"` leaves
  `UIOpaque` out, and `multipleTouchEnabled` is `UIMultipleTouchEnabled`
  before its autoresizing keys (Flutter's accessibility launch screen).
- With a `<device>`, the root view is laid out at the device's size (or
  its freeform size) at its frame's origin from iOS 17.0 on (a view below
  an opaque bar is at y 64; eight probes, with simulated bars changing
  nothing), at the origin below 17.0 (four probe storyboards at 12 to 16.4,
  at y 20 or 64, below a bar or not),
  whatever its frame's size says: 26 device ids measured in both orientations
  (landscape swaps width and height; `retina3_5` and `retina6_72` are
  393x852 either way; `appleTV` is 1920x1080). Ids Xcode doesn't know
  (`retina5_4`, `ipad10_5`, `ipad12_9`, `ipad8_3rounded`,
  `ipad13_0rounded`, `appleTV4K`) take the frame from iOS 17.0 on, and are
  the generic canvas below (a corpus storyboard at 16); other ids are
  refused.
- The root resizes its subviews by their masks when it is laid out at
  another size than its frame (sizable width and height grow with it:
  100x80 under a 375x667 frame laid out at 600x600 is 325x13), whether or
  not they translate their masks into constraints, and on a device whether
  or not they are constrained (eight probes, then six at iOS 16 and 18 and
  a corpus storyboard; on the generic canvas a constrained one kept its
  frame in a probe and was resized in a corpus storyboard). Where the
  root's size isn't its frame's, a subview whose mask has a flexible left
  or top margin or a sizable width or height is refused; the rest keep
  their frames, with or without a device.
- An image view's `<imageReference key="image" image="…"/>` naming only its
  image compiles as an `image` attribute (a probe).
- An image view (`UIUserInteractionDisabled` when `userInteractionEnabled="NO"`;
  `UIViewInsetsLayoutMarginsFromSafeArea` 0 after its priorities when it
  says NO: one probe; its placeholder's intrinsic size changes nothing: a
  corpus storyboard, with and without it;
  any content mode but `topLeft`, as UIViewContentMode's: ten probes) archives as a label's geometry and flags (a
  background colour first, hidden and tag after `UIOpaque`, alpha after
  the mask), then `UIContentMode` unless scale to fill, `UIClipsToBounds`,
  `UITintColor`, the autoresizing key (left out with
  `autoresizesSubviews="NO"`: one probe), its own constraints, its hugging
  (none without priorities; a missing one is 250), its compression
  resistance if given (each 750 unless given), and `UIImage`:
  a `UIImageNibPlaceholder` per image name (width and height 1, the name,
  nil symbol, configuration and catalog), shared by the views using it;
  a system symbol (`catalog="system"`) is a placeholder of width and
  height 0 naming it as both resource and symbol, shared by name apart
  from an image of the document's of that name (five probes, image views
  and a button's normal state; one the document's resources don't list
  is archived as a plain image, so Interface Builder's own documents
  always list theirs); no image, no key; then `UIAdjustsImageSizeForAccessibilityContentSizeCategory`
  when set (one probe). snag's launch screen is byte-identical.
- A button state's `<preferredSymbolConfiguration>` is
  `UIImageSymbolConfiguration` in its content right after `UIImage` (before
  the title colour): `UISymbolScale` when given (default -1, small 1,
  medium 2, large 3); for a point size, `UIPointSize` as a float and
  `UISymbolWeight` when given (ultraLight 1 to black 9); for a text style,
  the weight (4 unless given) and `UITextStyle`; for a system font, its
  point size and weight (15 probes and a corpus storyboard). A system
  font without a weight, other fonts and a configuration without an
  image are refused.
- A controller's own `restorationIdentifier` is `UIRestorationIdentifier`
  after `UINibName` (one probe; beside a storyboard identifier, or in an
  embedded controller, it is refused).
- A root view's mask is 36 without one and left out when empty (two
  storyboard probes, two xib probes). Its `tag` is `UITag` after `UIOpaque`
  (one probe), and its `userInteractionEnabled="NO"` and
  `multipleTouchEnabled="YES"` are `UIUserInteractionDisabled` and
  `UIMultipleTouchEnabled` after `UIOpaque`, as a subview's are (two
  probes), and its `restorationIdentifier` is `UIRestorationIdentifier`
  just before its semantic content attribute (two probes).
- A root view's content mode other than scale to fill is `UIContentMode`
  after its mask (two probes), then `UIClipsToBounds` when it clips (one);
  beside `clearsContextBeforeDrawing` or a tint they are refused.
  `UIClearsContextBeforeDrawing` false comes before its own or the
  document's tint (two probes). One that
  doesn't autoresize its subviews leaves `UIAutoresizeSubviews` out (one
  probe). A table view controller's `autoresizesArchivedViewToFullSize="NO"`
  is `UIAutoresizesArchivedViewToFullSize` false after its identifiers,
  before its segues (two probes). A controller's size in a popover
  (`<value key="contentSizeForViewInPopover" type="size">`) is
  `UIPreferredContentSize`, a size, after its identifiers and segue
  templates (three probes, AdaptivePopover); beside gesture recognizers,
  presentation styles, extended edges, scroll view insets or table
  selection settings, and in an embedded controller, it is refused.
  Transition and presentation styles follow the segue templates (three
  probes and a corpus storyboard). A navigation controller's
  `extendedLayoutIncludesOpaqueBars` is `UIKeyExtendedLayoutIncludesOpaqueBars`
  1 before `UIViewControllers` (one probe). A text view's custom class
  swaps in for it as a text field's does, and a segmented control's mask is
  a label's (below).
- A scene controller's `modalTransitionStyle` (flip horizontal 1, cross
  dissolve 2, partial curl 3; cover vertical left out) and
  `modalPresentationStyle` (as a navigation controller's) are
  `UIModalTransitionStyle` and `UIModalPresentationStyle` after its
  identifiers, before its extended edges and scroll view insets (seven
  probes); beside segues or gesture recognizers, or in an embedded
  controller, they are refused.
- A controller's `title` is its swapper's `UITitle`, first;
  `useStoryboardIdentifierAsRestorationIdentifier` adds
  `UIRestorationIdentifier` (the identifier's string) after
  `UIStoryboardIdentifier`. `userLabel` (on any element) and a
  controller's simulated metrics (`simulatedMetricsContainer`,
  `simulatedNavigationBarMetrics`, `<nil key="simulated...">`, ...) change
  nothing (five probes), except a freeform size: `<size
  key="freeformSize">` is the root view's size instead of the device's or
  the generic canvas's, though a device-less view from iOS 17.0 keeps its
  frame (seven probes and rules_apple's iMessage example). A
  `<navigationItem>` without items is the swapper's `UINavigationItem`,
  after `UITitle`: its title, the same string as an `NSAttributedString`
  with no attributes, the nib's one empty array as `UICenterItemGroups`
  (so the nib's visible windows and the rest reuse it) and
  `UILargeTitleDisplayMode` (automatic 0, always 1, never 2), listed last
  in the controller nib's objects (six probes). A label's
  `clipsSubviews` is `UIClipsToBounds` after `UIContentMode`, and a text
  field's after `UIAutoresizingMask` (only when it says YES); a text
  field's `placeholder` is `UIPlaceholder` after `UIFont`, and its declared
  text colour its `UITextColor` (`UIDisableTextColorUpdateOnTraitCollectionChange`
  is set either way; three probes). A text field with no text colour
  element is as with a nil one; a label with none is as with a declared
  `labelColor` (two probes). A text field with `usesAttributedText` and
  an empty `<attributedString>` has no `UIFont` or `UITextColor`; after
  `UITextAlignment` come `UIAttributedText`, an `NSAttributedString` of
  the empty string without attributes, and `UIText`, an empty
  `NSMutableString`, and `UIDisableTextColorUpdateOnTraitCollectionChange`
  is false (one probe); other attributed text is refused.
- A view's `UIBounds` is left out when its size is zero, and its
  `UICenter` when the centre is the origin. A label, button or plain view
  without a frame, or with an all-zero one, is 1000x1000 at the origin in
  a document of Interface Builder's `toolsVersion` 11019 or later, and has
  no frame at all (zero) in an older one (nine probes, bisected by
  `toolsVersion`, root views and a table cell's content view among them),
  or has no size when the document says `fixedFrame="YES"` (18 probes,
  three corpus storyboards); so is an image view or a segmented control
  (four probes, one corpus storyboard), and a table view, navigation bar
  or stack view or page control (corpus storyboards) without `fixedFrame` (two probes), an image view's `fixedFrame`
  otherwise changing nothing. An image view's `UIAutoresizingMask` is a
  label's (below). An older document's `<frame key="frameInset">` with
  only `minX`, `minY`, `width` and `height` is the frame at (minX, minY)
  (a probe); inset from the other edges it is refused. A view's `<userGuides>`, the canvas's, change nothing
  (one probe). An empty `<toolbarItems/>` on a view
  controller changes nothing (one probe).
- A label's `lineBreakMode` is `UILineBreakMode` after `UIMinimumFontSize`
  (or where it would be): wordWrap 0, characterWrap 1, clip 2,
  headTruncation 3, middleTruncation 5; tailTruncation, the default, is
  left out (eight probes).
- A label adjusts its font size to fit unless `adjustsFontSizeToFit="NO"`:
  without that attribute it has `UIAdjustsFontSizeToFit` after
  `UINumberOfLines`, then `UIMinimumFontSize` (a 32-bit float) before
  `UITextAlignment` and `UIMinimumScaleFactor` (also 32-bit) after it.
  A `preferredMaxLayoutWidth` is `UIPreferredMaxLayoutWidth`, a 32-bit
  float, after the alignment and any `UIMinimumScaleFactor` (three probes
  and a corpus storyboard; beside letter spacing it is refused).
  With `minimumScaleFactor` (which wins over `minimumFontSize`) the factor
  is kept and the size is factor times the font's point size rounded to
  the nearest quarter point, halves up (0.33 of 17 is 5.5, 0.375 of 17 is
  6.5); with only `minimumFontSize` the size is kept and the factor is the
  size over the font's (10.5 of 17 is 0.6176470518112183), even above 1. A
  zero `minimumFontSize`, and both attributes when the label doesn't
  adjust, are left out. `adjustsLetterSpacingToFitWidth` is
  `UIAdjustsLetterSpacingToFit` after the scale factor (32 probes, at 9,
  13, 17, 23 and 32 points). A text field's `clearButtonMode` is
  `UIClearButtonMode` after `UIMinimumFontSize`: whileEditing 1,
  unlessEditing 2, always 3 (three probes).
- Text alignment (labels and text fields) is NSTextAlignment: left 0,
  center 1, right 2, justified 3, natural 4, and left when the document
  gives none (nine probes). A text field's `borderStyle` is
  `UIBorderStyle` (line 1, bezel 2, roundedRect 3, none left out), with
  `UIRoundedRectBackgroundCornerRadius` only for roundedRect; `text` is
  `UIText` after `UITextAlignment`, and `enabled="NO"` is `UIDisabled`
  before `UIContentHorizontalAlignment`.
- Text input traits follow `UIClearButtonOffset`, each left out at its
  default (0), in this order: `UIAutocapitalizationType` (none, words,
  sentences, allCharacters), `UIAutocorrectionType` and
  `UISpellCheckingType` (default, no, yes), `UIKeyboardAppearance` (dark
  and alert 1, light 2), `UIKeyboardType` (UIKit's order: alphabet 1 to
  webSearch 10), `UIReturnKeyType` (go 1 to continue 11),
  `UIEnablesReturnKeyAutomatically`, `UISecureTextEntry`,
  `UITextContentType` (the document's string), then the smart insert
  delete, quotes and dashes types. Secure entry also sets autocorrection
  to no, whatever the document says, and `UITextInlinePredictionType` 1
  last (45 probes; `asciiCapableNumberPad` is not compiled and refused).
- A text view (`UITextView`) is a scroll view: after its view values (its
  hugging and compression resistance, when given, after its constraint
  keys; its runtime attributes pairs as a label's; `fixedFrame` and a
  placeholder intrinsic size changing nothing: seven probes),
  `UIScrollViewIndicatorInsetAdjustmentBehavior` 1, its `UIIndicatorStyle`
  (black 1, white 2: three probes), then `UIBounceEnabled`,
  `UIShowsHorizontalScrollIndicator`, `UIShowsVerticalScrollIndicator`
  (each false when turned off), `UIScrollDisabled`, `UIBouncesZoom` (left
  out when off), `UIKeyboardDismissMode` (onDrag 1, interactive 2: one
  probe) and `UIScrollViewContentInsetAdjustmentBehavior` 0, with
  `UIAlwaysBounceVertical` right after `UIBouncesZoom` (a corpus
  storyboard); one
  that doesn't scroll doesn't clip. Its data detectors are
  `UIDataDetectorTypes` (phone numbers 1, links 2, addresses 4, calendar
  events 8) after its alignment,
  left out when none (two probes). Its text (a `<string key="text">` or
  the older `text` attribute) is `UIText` (in `Base.lproj` localizable as
  `<id>.text`, the attributed string sharing its development value: one
  probe; text that fits an Objective-C tagged pointer string, up to 7
  ASCII characters, 9 of the 6-bit alphabet or 11 of the 5-bit one, is one
  object with the localizable string, which the attributed string refers
  to; text of more than 512 characters gives the attributed string a
  mutable string of its own, and outside `Base.lproj` makes `UIText` one
  too: 19 probes) and `UIAttributedText`, an
  `NSAttributedString` whose attributes are `NSColor` (the text colour, if
  any: none given is nil), `NSFont` and `NSParagraphStyle` (an `NSMutableParagraphStyle` with
  `NSAlignment` in AppKit's order, right 1 and center 2, left left out,
  and nil `NSTabStops`); the view's own `UIFont` and `UITextColor` then
  refer to them. Text views of the same colour, font and alignment in one
  nib share that attributes dictionary (a fuzzed storyboard). Then `UIAdjustsFontForContentSizeCategory`, `UIFont`,
  `UITextColor` (none for a nil colour), `UITextAlignment`, `UIEditable`
  (false when turned off, before any data detectors: one probe and one of
  the corpus's) and `UISelectable` (false when turned off), the text input traits,
  `UITextHighlightAttributes` (`NSColor` system blue,
  `NSTextHighlightOutlineColor` secondary system fill) and
  `UITextAllowsNumberPadPopover`. A text view's autocapitalization
  defaults to sentences: that is left out, and none (or no attribute) is
  written as 0. An opaque grey white background is
  `systemBackgroundColor`, as a plain table's is its own. Secure text
  views (which add more keys), `alwaysBounceVertical` and text in
  localizable storyboards are refused (32 probes).
- Nothing in the output comes from a plain view's placeholder intrinsic
  size or a root view's `simulatedAppContext` (two probes), both
  Interface Builder's alone.
- Nothing in the output comes from an empty `<gestureRecognizers/>` or `<animations/>`, an empty
  `<constraints/>`, a navigation controller's simulated metrics, a root
  view's `insetsLayoutMarginsFromSafeArea="YES"`, nor from a label's `fixedFrame`, from a label's
  or button's `<autoresizingMask>` when it doesn't translate it into
  constraints (constrained or not), or from a scene's `<exit>` when
  nothing unwinds to it (ten probes). An exit something refers to is
  refused.
- A view controller's `automaticallyAdjustsScrollViewInsets="NO"` is
  `UIKeyAutomaticallyAdjustsScrollViewInsets` 0 after its identifiers in
  its controller nib, and after `UIParentViewController` as a navigation
  controller's root (seven probes).
- The safe-area guide in the objects is an unowned `UILayoutGuide` with an
  empty identifier. A subview's own safe-area guide that no constraint
  uses is the same, listed first in the subview's turn in the objects'
  breadth-first order, before the constraints it holds and its subviews
  (two probes). One in use gives the subview its own archived guide, as
  the root view's below: owned by the subview, defined where its first
  constraint in order uses it (either item; it counts as at the origin,
  as the root's), in the subview's `UIViewLayoutGuides` after its
  constraints, its placeholder still listed (seven probes: plain and
  scroll views, the latter beside their content and frame guides, and a
  corpus storyboard). One used beyond its view's own constraints is
  refused. When a constraint uses the guide, the root view also
  owns a second one, archived where a constraint first refers to it (in
  the order above): identifier `UIViewSafeAreaLayoutGuide`, locked,
  archived and allowing negative dimensions, with four system constraints
  pinning its top, left, bottom and right to the view's
  (`UIViewSafeAreaLayoutGuide-top`, ... as `NSLayoutIdentifier`, not
  archived), listed in the view's `UIViewLayoutGuides` after its
  constraints. Its connections: the controller's outlets with `view`
  among them (to constraints too, the root view's or a subview's: three
  probes; a text field's `delegate` too, from the field to the
  controller's placeholder), then control actions, each in byte order of
  their names (`aButton`, `view`, `zButton`; `B:`, `a:`; `tap1:`, `tap:`,
  `tap_:`), whatever the document order, ids or positions; actions with
  the same selector in the order Xcode visits their controls, whatever
  their ids: the root view's subviews in document order, then, from the
  last visited, each one's subviews the same way, as a stack (so siblings
  stay in document order, 19 probes, and controls in containers of their
  own come last container first, eight probes and one of the corpus's).
  A xib's go in document order, each view before its subviews, whatever
  the containers (three probes, two of the corpus's). Outlets of one
  name from several views (two `delegate`s) go by the views' UIKit
  classes, whatever their custom classes: a picker's before a text
  field's, a text field's before a text view's (six probes); views of one
  class in the order Xcode visits them, as for actions (a probe and a
  corpus storyboard).
  Actions on other
  events sort by event mask first (valueChanged 4096,
  primaryActionTriggered 8192, editingDidBegin 65536, editingChanged
  131072, editingDidEnd 262144, editingDidEndOnExit 524288), then by name
  (two probes).
- A view nib that refers to its controller (an outlet into it, an action
  to it) has an `UpstreamPlaceholder-1` proxy, which the controller's nib
  maps to its class swapper in `UIExternalObjectsTableForViewLoading`.
- tvOS views have no `UIViewLargeContentStoredProperties`. A tvOS button's
  font is, whatever its type, the Body text style at size category 3:
  `.SFUI-Medium` 29 with a descriptor of `NSCTFontSizeCategoryAttribute`
  3, `NSCTFontUIUsageAttribute` `UICTFontTextStyleBody` and the size,
  options 2147517444, not as a label's text style (four probes and a
  corpus storyboard); one with a font of its own is refused. A tvOS
  collection view's `UICollectionViewPrefetchingEnabled` is false unless
  it says `prefetchingEnabled="YES"` (three probes and a corpus
  storyboard; the attribute is refused). A tvOS navigation bar has no
  `UIViewLayoutMargins` or `UIViewLayoutMarginsAreDirectional` (two
  probes and a corpus storyboard). A tvOS segmented control has no
  `UIPointerInteractionEnabled`, a tvOS rounded-rect text field no
  `UIRoundedRectBackgroundCornerRadius`, and a tvOS control's segue
  performs on its primary action (`perform:` with mask 8192, not 64):
  three corpus storyboards.

## Stack views, content modes, named colours and gesture recognizers

- The document's tint colour (a `<color key="tintColor">` after its
  resources) is `UITintColor` on each controller's root view without one
  of its own (after its mask), on a root table or collection view (after
  its clipping) and on a navigation bar (after its mask); a root view's
  own tint goes in the same place, and its colours other than background
  and tint are refused. A segmented control beside it takes nothing of it
  (one probe), and neither does a toolbar, in a view or a navigation
  controller (three probes and two corpus storyboards); beside tab bars,
  split and page view controllers and search bars the document's tint is
  refused, not yet probed (six probes).

- A stack view's `userInteractionEnabled="NO"` is
  `UIUserInteractionDisabled` after its hidden flag and tag (one probe).
  A content mode other than scale to fill is `UIContentMode` after its
  mask (one probe and one of the corpus's). Without `opaque="NO"` it is
  `UIOpaque` after `UIDeepDrawRect`, and it may be a table's header or
  footer view (corpus storyboards).
- A text field's `userInteractionEnabled="NO"` is
  `UIUserInteractionDisabled` right after `UIDeepDrawRect`, and
  `adjustsFontSizeToFit="NO"` leaves its `UIAdjustsFontSizeToFit` out (a
  probe and two of the corpus's).
- A stack view's `axis` is `UIStackViewAxis` (horizontal 0, vertical 1),
  `distribution` `UIStackViewDistribution` (fill 0, fillEqually 1,
  fillProportionally 2, equalSpacing 3, equalCentering 4), `alignment`
  `UIStackViewAlignment` (fill 0, leading or top 1, firstBaseline 2,
  center 3, trailing or bottom 4, lastBaseline 5, whichever the axis), a
  missing `spacing` 0;
  `baselineRelativeArrangement` sets `UIStackViewBaselineRelative`. Edge
  insets are its `UIViewLayoutMargins` and set
  `UIStackViewLayoutMarginsRelative` whether or not
  `layoutMarginsRelativeArrangement` does, which without them changes
  nothing; a hidden one has `UIHidden` after `UIDeepDrawRect` (one probe);
  an empty one has neither `UISubviews` nor `UIStackViewArrangedSubviews`
  (one probe and a corpus storyboard); a custom class is swapped in last
  and `insetsLayoutMarginsFromSafeArea="NO"` follows the margin flags (two
  probes);
  directional margins add `UIViewLayoutMarginsAreDirectional`
  after them and after `UIViewLayoutMarginsFollowReadableWidth`;
  preserving its superview's margins adds
  `UIViewPreservesSuperviewMargins` and
  `UIViewPreservesSpecificSuperviewMargins` 15 after the margins, before
  the directional flag (two probes and one of the corpus's; beside a
  readable width it is refused). A
  background colour comes after its subviews, its own constraints after
  the not-translating key (26 probes), then its hugging and its
  compression resistance, each `{h, v}` when either is given (250 and 750
  the defaults; nine probes).
- A view's or label's `contentMode` is `UIContentMode` after
  `UIAutoresizingMask`, UIViewContentMode's values, scaleToFill (0) left
  out; Xcode compiles `topLeft` as scaleToFill, so it is refused, but
  older documents' `TopLeft` as UIViewContentModeTopLeft, 9 (19 probes). A label is `UIUserInteractionDisabled` only when it says so, and
  so is a button, after its hidden flag and tag (five probes).
- A named colour (`<color name=...>`, defined by a `<namedColor>` in the
  document's resources) is `UIDynamicCatalogName`,
  `UIDynamicCatalogUseNibBundle` and its definition's components, unrounded
  (sRGB and grey; three probes and one corpus storyboard); a Display P3
  definition's are converted to extended sRGB as a plain Display P3
  colour's are (four probes).
- A scene's gesture recognizer whose delegate is the controller has a
  `delegate` outlet to it in the controller's nib, after the controller's
  storyboard outlet (one probe; several are refused). A navigation
  controller's root's has it in the navigation controller's nib, after
  the storyboard outlets, before `sceneViewController` (a corpus
  storyboard); beside the root's outlets to custom views, or in other
  embedded controllers, it is refused.
- A scene's gesture recognizer (tap, pan or swipe) is in
  the controller's nib (one no view has there alone, kept alive with the
  others but without a placeholder in the view's nib: one probe), after the storyboard placeholder, with
  `UIGestureRecognizer.allowedTouchTypes` 15 and
  `requiresExclusiveTouchType` (after `UIGestureRecognizer.disabled` when
  `enabled="NO"`, a probe of a tap and a pan, or
  `UIGestureRecognizer.delaysTouchesBegan`, a corpus storyboard; both
  together are refused), then a tap's `_imp` `UITapRecognizer`
  (taps or touches when not 1), a pan's minimum and maximum touches and
  `allowedScrollTypesMask` 0, a swipe's direction (left 2, up 4, down 8,
  right left out), a long press's (`<pongPressGestureRecognizer>`, as
  Interface Builder spells it) `_imp` `UITapRecognizer` of its taps (0
  unless given, left out when 1) and touches (unless 1), then its touches,
  `minimumPressDuration` and `allowableMovement` as floats unless 1, 0.5
  and 10 (three probes and a corpus storyboard); with a custom class, a `UIClassSwapper` of those values
  and the class names, as a view's (two probes). The controller's external objects map
  `UpstreamPlaceholder-1` to it, before the controller's own placeholder
  and the segues', which move up by one; `UITopLevelObjectsToKeepAliveFromStoryboard`
  follows the segue templates; its action is an event connection without
  a mask after the templates' outlets. In the view nib, each view that has
  it (plain views, labels, image views, the root view) has a
  `gestureRecognizers` `UIRuntimeOutletCollectionConnection` to the
  placeholder (an `NSMutableArray`, `runtimeCollectionClassName`
  `NSArray`, appending), before the outlets (13 probes).
- Several recognizers in a scene: their placeholders are numbered from 1
  through the views that have them from the last in the document (the
  root view, whose connections follow its subviews, first), each view's in
  its document order, and the controller's own placeholder follows them.
  A view with several has one collection connection holding them all; the
  root view's comes first, then the others in document order. The
  controller's nib lists the recognizers by UIKit class name (long press,
  pan, swipe, tap), each class's in reverse document order (eleven probes
  of up to four classes and a corpus storyboard; recognizers of one class
  alone, as the earlier ten probes were, look like reverse document order),
  `UITopLevelObjectsToKeepAliveFromStoryboard` in document order, and
  their actions by selector, one selector's in document order (ten
  probes). A recognizer of two views is refused, and so is a view whose
  collection names its recognizers in another order than the document's
  (they are then defined otherwise: one probe). The controller's outlets
  to its recognizers are in its nib, after its storyboard outlet and
  before a recognizer's delegate outlet, by name (four probes and one of
  the corpus's). Beside bar button items in the controller's own nib, the
  recognizers' actions and the items' go together by selector (two probes
  and two random storyboards), and so does one item segue's `perform:` (a
  corpus storyboard); several items' segues, custom views' actions there,
  and a selector both share, are refused.
- A navigation controller's root that nothing loads on its own keeps its
  recognizers in the navigation controller's nib: defined in the root's
  `UIExternalObjectsTableForViewLoading` in their placeholders' order,
  kept alive (`UITopLevelObjectsToKeepAliveFromStoryboard`, after
  `UIParentViewController`), top-level objects after the navigation
  controller (in reverse document order) and listed right after it, their
  actions after `sceneViewController` (two probes). A tab bar
  controller's child that nothing else loads keeps them in the tab bar
  controller's nib the same way: defined in the child's external objects,
  kept alive after `UIParentViewController`, top-level and listed right
  after the tab bar controller in reverse document order, their actions
  after `sceneViewController` by selector (two probes and a corpus
  storyboard). Beside segues or bar button items, in several children, in
  a split view controller's child and in other containers, they are
  refused. An identified collection view controller that is also a
  navigation controller's root has them in both nibs; its archived-view
  setting precedes their keep-alive list (a corpus storyboard).
- A collection view controller's recognizers, all its collection view's:
  the view nib gives their placeholders the first numbers, then the
  controller's and the segues', and its collection connection comes
  first; the controller's nib lists them after the controller (top-level
  and objects, before its navigation item), defined in its external
  objects table, its outlets to them as a plain controller's (two corpus
  storyboards). A table view controller's are refused.
- A navigation controller's root's runtime attributes are key-value pairs
  on it in the navigation controller's nib too (two probes and a corpus
  storyboard); other embedded controllers' are refused.
- A recognizer's segue (`<segue>` in its connections) is templated as a
  bar button item's is: in `UIStoryboardSegueTemplates` after the
  controls', listed after them and the controller, its `viewController`
  outlet with the others, and the recognizer's `perform:` on it sorted by
  selector with the recognizers' actions. In a navigation controller's
  nib the root's templates come before its recognizers kept alive (five
  probes). Its destination is loaded as a control's segue's is (a corpus
  storyboard). Several recognizers' segues, one beside bar button items' segues
  or items, and in a navigation root beside other segues, are refused.

- A scene's custom objects (`<customObject>`, of a custom class) are
  top-level in the controller's nib before its recognizers, in reverse
  document order, each a `UIClassSwapper` of `UIClassName` (the runtime
  name) and `UIOriginalClassName` `UICustomObject`, and listed so;
  `UITopLevelObjectsToKeepAliveFromStoryboard` holds them and the
  recognizers in document order. The controller's outlets to them are in
  its nib after its storyboard outlet, by name (with those to its bar
  button items when it has both: three probes). Those the view's outlets
  name (a text field's delegate, a collection view's data source or
  delegate, in document order beside the controller's) have placeholders
  in the view nib, numbered after the recognizers' in the same order; the
  controller's own placeholder comes after them, but before them when the
  controller has outlets into the view (16 probes and a corpus
  storyboard). Custom objects with connections of their own, without a
  class, beside segues, insets or edge settings, in table or collection
  view controllers' scenes or in an embedded controller's, and beside
  outlets to recognizers or custom views, are refused.
- A scene's views beside its controller (a `<view>`, `<label>` or other
  view in its objects) are the controller's nib's: top-level after the
  storyboard placeholder, kept alive with the recognizers and custom
  objects in document order, each compiled as a plain subview. The
  objects list them after the placeholders and then, level by level
  after the controller, each level's views' turns (an unused safe-area
  guide's placeholder, constraints, subviews) before the controller's own
  objects of that level (its navigation item, tab bar item, templates and
  toolbar items, then its bar button items). The controller's outlets
  into them are by name with those to its custom views, their controls'
  actions on it follow the bar items', and their runtime attributes are
  the nib's pairs (four corpus storyboards). A table view controller's
  come after the controller instead, top-level and in the objects, defined
  where its keep-alive list names them (a corpus storyboard). An embedded
  controller's, a collection view controller's, and ones beside
  recognizers or custom objects, are refused.

## Navigation controllers

A `<navigationController>` (its bar, a root view controller by a
relationship segue, empty toolbar items) compiles to
`UINavigationController-<id>.nib` (or its identifier's), which embeds the
root controller; the root's scene adds its view nib, and its own nib only
when it is initial or identified. A navigation controller nothing loads
(neither initial, identified nor a segue's destination) compiles to
nothing, nor does its root unless the root is identified, nor a controller
only its root's segues reach (three probes and a corpus storyboard). One
without a root controller (no relationship segue) that something loads
is a nib of the owner, the first responder, its storyboard placeholder
and the controller holding its bar alone: `UIStoryboardIdentifier` and
`UIRestorationIdentifier` when identified, `UINavigationBar` without
`UIItems`, and the class names when swapped in; the objects list the
three placeholders, the controller and the bar, the connections its
storyboard outlet and the owner's `sceneViewController` (five probes,
initial, identified, of a custom class and a modal segue's destination,
and a corpus storyboard's five). With a title, a tab bar item or
presentation settings, or as a tab or split view controller's child, it
is refused. Observed with 25 probes:

- The root's own tab bar item is its `UITabBarItem` before its navigation
  item, listed after its navigation item (two probes and a corpus
  storyboard); beside the navigation controller's own, it is refused.
- The navigation controller's own empty navigation item is its first
  value, `UINavigationItem` (as any empty item: its centre item groups and
  large title mode), listed after its bar (one probe); one with contents,
  or beside a title, an identifier or a tab bar item, is refused.
- Top-level objects: the root's scene's storyboard placeholder, the owner,
  the first responder, the navigation controller's storyboard placeholder
  and a class swapper for the controller (`UINavigationController`, or the
  custom class's runtime name): `UITitle`, `UIStoryboardIdentifier`,
  `UIRestorationIdentifier`, `UIChildViewControllers` (the root),
  `UIDefinesPresentationContext` and
  `UIProvidesPresentationContextTransitionStyle` true when it says so
  (three probes), `UIModalPresentationStyle` (UIKit's order: full screen 0, page sheet 1,
  form sheet 2, current context 3, over full screen 5, over current
  context 6, popover 7; none when automatic: eight probes),
  `UIKeyAutomaticallyAdjustsScrollViewInsets` 0 when it says NO,
  `UIViewControllers` (the root again), `UINavigationBar`, the class names.
  The objects list then has the bar, the root and its navigation item, if
  the document declares one; the connections are the two storyboard
  outlets (controller, then root) and `sceneViewController`.
- The root is a `UIViewController` (a `UIClassSwapper`, its class names
  last, when it has a custom class or a nib of its own): `UITitle`, its
  navigation item (as in its own nib, or one with only
  `UILargeTitleDisplayMode` when the document has none), the external
  objects table (`UpstreamPlaceholder-1` for outlets into its view), its
  view nib's name, its identifiers, `UIParentViewController`,
  `UIKeyAutomaticallyAdjustsScrollViewInsets` when it says NO. The item
  ends with `UINavigationBar`, a reference to the bar archived later. Its
  title is encoded before its navigation item, which shares an equal
  string; the navigation controller's own title is encoded before both,
  so a root title equal to it refers to its string (one corpus
  storyboard). A table view controller root is a `UITableViewController`, with
  `UIClearsSelectionOnViewWillAppear` last (four probes).
- The bar is laid out by Xcode, not the document: bounds 0x54 centred 27
  below its frame's top (the root view's top without a frame, one probe
  and one corpus storyboard), mask 2, hugging `{250, 250}`, zero layout margins
  (directional), `UIViewInsetsLayoutMarginsFromSafeArea` 0 when the
  document turns it off, `UIIsLocked`, `UIItems` (the root's item),
  `UIDelegate` the controller, `UIBarPosition` 2, `UIBarTranslucence` 1;
  `largeTitles` is `UIBarPrefersLargeTitles` after the large content
  properties, the layout the same (two probes).
- A navigation controller's hidden bar (`navigationBarHidden="YES"`) is
  `UINavigationBarHidden` true right after its `UINavigationBar` (a corpus
  storyboard); beside a toolbar it is refused.
- A navigation controller's shown toolbar (`toolbarHidden="NO"` and a
  `<toolbar key="toolbar">`) is `UIToolbar` after its `UINavigationBar`,
  then `UIToolbarShown` true, listed after the bar: a `UIToolbar` 54
  points high at its document place and width (as the bar's 54), with
  `UIDeepDrawRect`, `UIAutoresizeSubviews`, mask 10,
  `UIClearsContextBeforeDrawing` false when it says so, hugging, no
  layout margins, the safe-area insets setting, semantic and large
  content, `UIBarStyle` 1 for black, the nib's empty array as its items,
  `UIBarPosition` 1 and `UIBarTranslucence` 1 (a corpus launch screen).
  A hidden one, or one beside a parent or its own navigation item, is
  refused. A toolbar in a view takes a black style the same way, before
  its items (two probes and a corpus storyboard).
- The bar's appearance: a background colour before `UIDeepDrawRect`;
  `translucent="NO"` adds `UIOpaque` after it and makes
  `UIBarTranslucence` 2 (a translucent bar's `opaque="NO"` changes
  nothing: one corpus storyboard; beside `translucent="NO"` it is
  refused); its tint (or the document's) after the mask;
  after the large content properties `UIBarStyle` 1 for black, then
  `UIBarPrefersLargeTitles`, then `UIBarLargeTitleTextAttributes`; its
  bar tint `UIBarTintColor` after `UIIsLocked`; `UITitleTextAttributes`
  after `UIDelegate`. Text attributes are a dictionary of `NSColor` (the
  text colour) then `NSFont` (ten probes). Other bar settings, other
  segues and controllers nothing loads are refused.

A navigation item's left and right bar button items are its
`UILeftBarButtonItem` and `UIRightBarButtonItem`, after its title, and each
alone in `UILeftBarButtonItems` and `UIRightBarButtonItems`, around
`UICenterItemGroups`; with a list on a side (`<leftBarButtonItems>`), the
first is that side's single item and the rest are defined in its array,
the objects listing the left side's then the right side's in order (three
probes; list items sharing a selector are refused). An item is `UIEnabled`, then for a system item
`UIIsSystemItem` and `UISystemItem` (UIBarButtonSystemItem's numbering
by the item's name, but `close` is 0) and for a space `UIHidesSharedBG`,
then `UIStyle` (1 with no style, 2 for done, none for plain), then its
`UITitle`, then its image (`UIImage`, a placeholder as an image view's);
an item may have a title, an image, both or neither (five probes), then
its image insets (`UIImageInsets`, top, left, bottom, right: four probes;
its landscape insets, which change nothing here, are refused), then its
tint (`UITintColor`; two probes). A width is `UIWidth`, a float, after its
title (a corpus storyboard; beside an image, insets, a tint or a custom
view it is refused). The items follow the controller's segue templates in its nib's
objects (its own nib's, or its navigation controller's).
- **Actions.** An item's action on its controller is an event connection
  without a mask.
- **Segues.** An item's segue is a template in the controller's nib, after
  its own and its view's in the templates array and objects, with a
  `perform:` event connection from the item. An item with both an action
  and a segue compiles as with the segue alone: the action is dropped,
  whichever comes first (two probes, two of the corpus's storyboards).
- **Order.** These connections come after the templates' outlets, by
  selector: actions with one selector the right item's first, and the
  `perform:`s in the order of their templates' outlets.
- **Back item.** A back bar button item is `UIBackBarButtonItem`, compiled
  as any other item, right after the title and before the other items, and
  first among the items in the objects, wherever the document puts it (six
  probes, a root and a pushed controller). One with connections or a
  custom view, or in a navigation bar inside a view, is refused.
- **Custom views.** An item's custom view (`<button key="customView">`, a
  segmented control, a plain view: two probes) is its `UICustomView`, first among its values: the
  view compiled as a subview, its document mask kept. The objects list the
  custom views after all the items, in the same order. Their actions on
  the controller keep their event masks and follow the items' actions and
  `perform:`s, by mask and then selector, whatever the items' sides. A
  controller's outlet to one is in the nib holding the item, after the
  controller's storyboard outlet (13 probes, pushed and root controllers);
  so is one to a navigation root's bar button item, in the navigation
  controller's nib by name with those (a corpus storyboard). A
  controller's runtime attributes and its items' pairs go together, its
  own first, as the objects list them; with a tab bar item too, the
  objects list the navigation item, the tab bar item, then the bar button
  items (a corpus storyboard's two controllers). A controller's outlets to
  a toolbar's bar button items or custom views in its view are the view
  nib's (a corpus storyboard).
  An item's runtime attributes, or its custom view's key-value pairs (a
  switch's frame), are key-value pairs in the nib holding the item: the
  controller's, or the navigation controller's for its root (two corpus
  storyboards); several items', or beside the controller's own, are
  refused. A custom view's control's segue is templated as an item's is,
  the control performing it on touch up inside (`UIEventMask` 64) after the
  other events (a corpus storyboard's button presenting a controller);
  several, or one beside custom views' actions, are refused. Custom views
  with subviews, constraints, outlets or accessibility settings are
  refused, as are two actions with one mask
  and selector, and custom views in a navigation bar in a view. A custom
  view of a custom class is swapped in as a subview is, and several
  outlets to custom views are sorted by name (two corpus storyboards).
- **Toolbar items.** An identified controller's `<toolbarItems>` are
  `UIToolbarItems` after its `UIStoryboardIdentifier`: bar button items
  (plain ones without `UIStyle`, a flexible space with `UIHidesSharedBG`),
  listed after its navigation item, then the item's bar button items,
  then their custom views. Their custom views' outlets (several, by
  name), actions (with the navigation item's) and key-value pairs are as
  the navigation item's (two corpus storyboards' toolbar scenes). In
  unidentified, embedded, table or collection view controllers, beside a
  navigation item's custom views, segues, recognizers or later settings
  they are refused.

A child navigation controller's root may have segues (Xcode's
master-detail template): its templates follow that child's deeper parts
(its root's navigation item) in the parent's objects, before the next
child's, and their `viewController` outlets to the root follow the scene
outlet (a corpus tab bar controller and two probes with the segue on a
later child). A child navigation controller a segue shows (the detail) has
a nib of its own too, as any controller a segue loads; so does an
identified one. In its parent's nib an identified child is swapped in for
its class: `UIStoryboardIdentifier` (and `UIRestorationIdentifier`) after
its title and tab bar item and before `UIParentViewController`, the class
names last (ten probes, split and tab bar controllers, titled or not, and
two corpus storyboards). An initial one is refused. An embedded
controller's `autoresizesArchivedViewToFullSize="NO"` is
`UIAutoresizesArchivedViewToFullSize` false after its parent (two probes
and a corpus storyboard); beside gesture recognizers it is refused. Its bar's `multipleTouchEnabled` is
`UIMultipleTouchEnabled` before the autoresizing keys and `clipsSubviews`
`UIClipsToBounds` after them (two corpus storyboards, Swift and
Objective-C). A navigation controller's tab bar item in its own nib (a
tab's child loaded on its own, or one with none) is its first value
after its title, before its identifiers, and follows its bar in the
objects, as a tab's child's (four probes, a corpus storyboard's).

A tab or split view controller's children's bar button items (a view
controller child's, or a navigation child's root's) are in the parent's
nib with their navigation items: listed after every child's parts and the
roots' navigation items, child by child, and their actions on their
controllers follow the scene outlet by selector, children sharing a
selector in their order (seven probes, and 150 random storyboards). A
child's segue templates come before them, after the roots' navigation
items, whether the segues and the bar items are one child's or two
(six probes and a corpus storyboard). A child that also has a tab bar
item has no nib of its own to refuse them in. Bar items with segues or
custom views' actions and two actions of one child sharing a selector are
refused, as are a child's bar button items beside gesture recognizers and with
images (46 probes).

## Page view controllers

An identified `<pageViewController>` without pages of its own compiles to
a nib named by its identifier: the owner, first responder and storyboard
placeholders and the controller, swapped in (its custom class's runtime
name, or `UIPageViewController`), with `UIStoryboardIdentifier`,
`UIAutoresizesArchivedViewToFullSize` false when the document says NO,
`UITransitionStyle` (page curl 0, scroll 1), `UINavigationOrientation`
(horizontal 0, vertical 1), `UISpineLocation` (none 0, min 1, mid 2, max 3)
and `UIPageSpacing` as a whole number (12.5 is 12), then its storyboard and
`sceneViewController` outlets; the Info.plist lists it as a controller
(seven probes). Its navigation item comes first in the controller's
values, listed after the controller with its title view and bar button
items (then the title view's constraints and subviews, one level deep),
the controller's outlets to those views by name after its storyboard
outlet, the items' actions after `sceneViewController` (two corpus
storyboards). Fractional spacing, an unidentified or initial one,
children and segues to one are refused.

A SwiftUI `<hostingController>` a segue leads to compiles to a nib named
`UIHostingController-<id>` (so its segue templates' destination), listed
in the Info.plist: the owner, first responder and storyboard placeholders
and a plain `UIViewController` with no view nib, swapped in for a custom
class (`UIOriginalClassName` `UIViewController`) or when a segue to it
names a `destinationCreationSelector` (`UIClassName` `UIViewController`
too: two probes), its empty navigation
item (`UICenterItemGroups`, `UILargeTitleDisplayMode` 0) its only value
and after it in the objects, then its storyboard and `sceneViewController`
outlets (three probes, Kingfisher's demo). Initial, identified or
contained ones, and anything inside one but an empty navigation item, are
refused.

## Tab bar controllers

A `<tabBarController>` (its tab bar, children by `viewControllers`
relationship segues, empty toolbar items) compiles to
`UITabBarController-<id>.nib` (or its identifier's), which embeds every
child: view and table view controllers as a navigation controller embeds
its root, and navigation controllers with their bar and root. The
children's scenes add their view nibs, and their own nibs only when they
are identified or a segue reaches them. An empty navigation item of its
own is its first value, listed after it (one probe). Observed with 17 probes and 19
storyboards from the corpus that use one (4 of them compile today; the
rest are refused for other elements):

- **Title views.** A navigation item's title view (`<label
  key="titleView">`, a button, a plain view) is its `UITitleView`,
  compiled as a custom view is, its document frame and mask kept, after
  `UILeftBarButtonItems` and before `UICenterItemGroups`; the objects list
  it right after the item, before the item's bar button items and their
  custom views. `<nil key="title"/>` is as no title (eight probes: beside
  left and right items and custom views, as a root controller's item and
  in its own nib too). Beside a back item, and with connections, runtime
  attributes or accessibility, it is refused; with subviews or
  constraints it is refused but in a page view controller's item (Page
  view controllers).
- Top-level objects: a storyboard placeholder per controller the nib holds,
  in reverse: the last child's first, a navigation child's root before the
  child itself. Then the owner, the first responder, the scene's own
  storyboard placeholder and the controller (`UITabBarController`, or a
  class swapper): `UITitle`, its identifiers, `UIChildViewControllers` (the
  children, when it has any), the presentation context flags as a
  navigation controller's (two probes),
  `UIKeyAutomaticallyAdjustsScrollViewInsets`,
  `UITabBar`, `UICustomizableViewControllers` and `UIViewControllers` (the
  children both times), the class names.
- The objects list continues with the tab bar and the children, then each
  child's parts in order (a view controller's navigation item and tab bar
  item; a navigation child's bar, tab bar item and root), then the roots'
  navigation items. Only items the document declares are listed.
- The connections are the scene's storyboard outlet, then one per held
  controller in the placeholders' forward order, stably sorted by original
  class (`UINavigationController`, `UITableViewController`,
  `UIViewController`), then `sceneViewController`.
- Each child begins as in its own nib, with `UITabBarItem` after its title
  and `UIParentViewController` the tab bar controller; a view controller
  child with no navigation item has none, while a navigation child's root
  keeps its implied one. A child with no `<tabBarItem>` gets one Xcode
  implies: `UIEnabled` and `UISpringLoaded`, both true. A declared item has
  `UISystemItem` (UIKit's `UITabBarSystemItem` order), `UIEnabled`, `UITag`,
  `UITitle`, `UIImage` (a system symbol as an image view's, one probe),
  `UISelectedTemplateImage`, then `UIImageInsets`;
  a `largeContentSizeImage` is `_UIBarItemLargeContentSizeImageCodingKey`,
  the item's first key (two probes)
  (top, left, bottom, right from its `imageInsets`; four probes). A view
  controller's own
  nib carries its declared item the same way, after its title, and lists it
  after the navigation item.
- The bar is laid out by Xcode: bounds 0x54 centred at its frame's x and
  27 below its top (without a frame, at the origin: a corpus storyboard) (a mask of flexible width and top margin changes
  nothing: six probes and two corpus storyboards), its background colour if any, opaque, mask 10, hugging `{250, 250}`,
  `UIViewInsetsLayoutMarginsFromSafeArea` 0 when turned off, `UIDelegate`
  the controller, `UIItems` (an `NSArray` of the children's items),
  `UIBarTranslucence` 1, or 2 when `translucent="NO"`.
- One empty external objects table and one empty attributes dictionary (of
  navigation item titles) are shared by every object that has one.
- Simulated bottom bar metrics change nothing. A plain child's segues
  (a container view's embed segue) are templates after its
  `UIParentViewController`, defined in its external objects table, listed
  among its own parts after its tab bar item, their `viewController`
  outlets to it after `sceneViewController` (one corpus storyboard, twice);
  several children's, or one beside a navigation item, bar button items or
  recognizers, are refused. A navigation child's root's bar button items'
  segues are templates in the root's `UIStoryboardSegueTemplates`, listed
  among its deeper parts after its navigation item and its own tab bar item
  (which comes before its navigation item, as in a navigation controller's
  nib), their `viewController` outlets to the root after the scene outlet
  and the items' `perform:` after all the outlets, both in the children's
  order (a corpus storyboard's three children, a probe of two in reverse
  document order); beside the items' actions they are refused.
- A navigation controller's root may be a plain tab bar controller with an
  empty navigation item of its own: the navigation controller's nib holds
  it as its root, as a held controller, its placeholder before its
  children's and its storyboard outlet first among theirs; its navigation
  item is on the navigation bar, `UIParentViewController` the navigation
  controller after that item; the objects list the navigation controller
  and its bar before it, then its parts as in its own nib; it has no nib
  of its own (a corpus storyboard, a probe). With navigation children it
  is refused (their placeholders' outlets go otherwise: a probe), as is
  one loaded on its own too.
- A custom object in the tab bar controller's scene, of a custom class,
  is top-level and listed after the storyboard placeholder, before the
  controller, which keeps it alive (`UITopLevelObjectsToKeepAliveFromStoryboard`
  after `UIChildViewControllers`); its outlets to the controller follow the
  scene outlet (a corpus storyboard, a probe). Several, outlets to anything
  else, connections to it, and one in a split view controller's scene are
  refused. Identified navigation children or ones loaded
  on their own too, tab bar items with insets, and tab bar controllers
  nothing loads are refused.

## Split view controllers

A `<splitViewController>` with master and detail relationships (view,
table view or navigation controllers) compiles to
`UISplitViewController-<id>.nib` (or its identifier's), which embeds its
children as a tab bar controller's nib does, without tab bar items: a
view controller child has a navigation item only if the document gives it
one. Observed with 17 probes and two corpus storyboards:

- The controller: its identifiers, `UIChildViewControllers` (master, then
  detail, whatever the document's order),
  `UISplitViewControllerPrimaryEdge` 0, the class names.
- Three orders meet. The storyboard placeholders are in reverse, and the
  storyboard outlets stably by class, of the held controllers in the
  relationships' document order (a navigation child before its root).
  Xcode's own order of these outlets varies from run to run beside
  several navigation children (three orders in twelve runs of a corpus
  storyboard); this one is one of them.
  The objects list the children by relationship name (detail before
  master), then their parts and then the roots' navigation items in that
  order too. With a tab bar controller's single relationship name, all
  three are document order.
- A navigation child is as in a tab bar controller's nib, with
  `UIKeyAutomaticallyAdjustsScrollViewInsets` 0 when it says NO (as in
  both).
- Not compiled yet, though probed: `style` (doubleColumn and
  tripleColumn alone change nothing; with a supplementary column,
  tripleColumn adds `UISplitViewControllerStyle` 2 before the primary
  edge), supplementary columns (in `UIChildViewControllers` between
  master and detail) and compact columns (a separate
  `UISplitViewControllerCompactColumnVC`). The objects' order by
  relationship name holds for those too (compact, detail, master,
  supplementary).

## Table view controllers

A `<tableViewController>` compiles as a view controller does, its class
`UITableViewController` (`UITableViewController-<id>.nib` when it has no
identifier), with `UIClearsSelectionOnViewWillAppear` false after
`UIKeyAutomaticallyAdjustsScrollViewInsets` when it says NO. Its view nib's
root view is its `<tableView>`, with no layout guides; the table's
`dataSource` and `delegate` outlets to the controller are sorted with
`view` (the upstream placeholder is in the nib for them). A table view
with a custom class, here or as a subview, is a `UIClassSwapper` of its
values and the class names after them (two probes). Observed with 94
probes:

- The table's values: geometry (the device's size, as a root view's),
  `UIBackgroundColor`, `UIDeepDrawRect`, `UIOpaque` (unless it says NO),
  `UIAutoresizeSubviews`, `UIAutoresizingMask`,
  `UIClearsContextBeforeDrawing` false when it says NO, `UIClipsToBounds`,
  the semantic content attribute, `UIViewLargeContentStoredProperties`,
  `UIScrollViewIndicatorInsetAdjustmentBehavior` 1, `UIBounceEnabled`
  false when it says so (one probe), `UIShowsHorizontalScrollIndicator`
  false when it says NO (the vertical one changes nothing), `UIBouncesZoom`
  (left out when it says NO: one probe), `UIAlwaysBounceVertical` when
  YES, `UIKeyboardDismissMode` (onDrag 1, interactive 2),
  `UIScrollViewContentInsetAdjustmentBehavior` 0, `UITableViewStyle`
  (plain 0, grouped 1, insetGrouped 2), `UIStyle` (0 plain, else 1), the
  separator style, `UIPrefetchingEnabled`, the row, section header and
  footer heights and their three estimates as floats, `UIFillerRowHeight`
  0.0 and `UISectionHeaderTopPadding` -1.0 as doubles,
  `UIAllowsSelection` false, `UIAllowsSelectionDuringEditing`,
  `UIAllowsMultipleSelection` and `UIAllowsMultipleSelectionDuringEditing`
  true when set, in that order (five probes, together and alone),
  `UIInsetsContentViewsToSafeArea`, the header and footer views,
  `UISeparatorColor` (four probes with a header, a footer or both), the
  prototypes,
  `UISectionIndexBackgroundColor`, `UISeparatorInsetReference` 0; a
  separator colour, a section index background and a separator inset go
  together in that order (five probes, plain and static tables). A picker
  view may be the header or footer view too, its outlets to the
  controller as a search bar's (two probes). The header's and footer's
  controls' actions on the controller follow the outlets, by event mask
  then selector, and their runtime attributes are the nib's key-value
  pairs (three of the corpus's; beside the cells' they're refused, and
  the header's actions come before a refresh control's: a corpus
  storyboard).
- A static table's cell's views' outlets to the controller (a collection
  view's data source and delegate) are among the outlets before the
  header's and footer's and the table's own of one name (three corpus
  storyboards); several cells' are refused.
- Left out, the row height is 44, a plain table's section heights 22, and
  the estimates 0.
- A table without user interaction has `UIUserInteractionDisabled` after
  `UIOpaque` (one probe).
- The separator style: default and singleLine are `UISeparatorStyle` and
  `UISeparatorStyleIOS5AndLater` 1; none only
  `UISeparatorStyleIOS9AndLater` 0; singleLineEtched all three, 2. A
  table with no separator style has the default (one probe).
- A plain table's white background, `systemBackgroundColor` or any opaque
  grey white (gamma 2.2, calibrated or device), is
  `tableBackgroundColor`. A grouped or inset grouped table's sRGB
  background of `systemGroupedBackgroundColor`'s own value (242, 242 and
  247 of 255, at 10 digits) is that system colour, `UISystemColorName`
  first (six probes and a corpus storyboard; a plain table's and a near
  value are not). Other colours are archived as usual.
- Each prototype cell with a reuse identifier is a nib of its own, in
  `UITableViewCellPrototypeNibs`: a dictionary of the identifiers, in byte
  order, each followed by a `UINib` (`captureEnclosingNIBBundleOnDecode`,
  `archiveData` an `NSData` of the cell's nib), then in
  `UITableViewCellPrototypeNibExternalObjects` the identifiers again, each
  with an empty dictionary. A prototype without an identifier is left
  out, and a cell's `rowHeight` changes nothing.
- A cell's nib: the owner, the first responder and the cell (a
  `UIClassSwapper` with a custom class), its connections the cell's
  outlets in byte order of their names (the nib's empty array when there
  are none), then its controls' actions on the cell, by event mask and
  then selector (five probes and a corpus storyboard). Its objects are the
  cell, its content view and the content view's subviews; then, breadth
  first, each view's constraints and its subviews, the content view's not
  listed again. Its switches' frames and its views' runtime attributes are
  its key-value pairs, ordered as a controller's view's: by the views'
  places in its objects, counted from 1 and compared as decimal strings
  (eleven probes, up to eleven views and two switches), and so are a
  collection view cell's (two probes). The cell's own runtime attributes
  are pairs of its own, before its views' (two probes).
- The cell: geometry from its frame in the document, `UISubviews` (the
  content view and a `_UITableViewCellSeparatorView`), its background,
  `UIDeepDrawRect`, `UIOpaque`, `UIAutoresizeSubviews`, its
  `UIAutoresizingMask` when it has one (prototype and static cells, three
  probes), `UIClipsToBounds` when clipping, `UIViewPreservesSuperviewMargins` and
  `UIViewPreservesSpecificSuperviewMargins` 15 when preserving margins,
  `UIViewInsetsLayoutMarginsFromSafeArea` 0 when turned off (one probe),
  the semantic content attribute, large content, `UIReuseIdentifier`,
  `UIIndentationLevel` when not 0, `UISelectionStyle` (none 0, blue 1,
  gray 2; left out for default), `UIAccessoryType` (disclosureIndicator 1,
  checkmark 3, detailButton 4), `UIIndentationWidth` as a float,
  `UITableCellBackgroundColorSet` with a background colour (before the
  separator inset: two probes),
  `UISystemBackgroundView`, `UIContentConfigurationView` nil,
  `UIContentView`, and nil text, detail text and image labels.
- A cell with a background colour archives it with
  `UITableCellBackgroundColorSet`. Without one, a cell that selects other
  than by default gets `tableCellPlainBackgroundColor` (whatever the
  table's style) with `UIDidEncodeCompatibilityBackgroundColor`, and a
  `_UISystemBackgroundView` sized as the cell rounded up (43.67 high is
  44) holding a `UIBackgroundConfiguration` of that colour, a clear
  stroke, `{0, 0, 0, 0}` insets and nothing customized.
- The content view (a `UITableViewCellContentView`, archived with UIView
  as its fallback class): geometry from the document, its subviews, its
  background colour, `UIDeepDrawRect`, `UIMultipleTouchEnabled`,
  `UIAutoresizeSubviews`, `UIContentMode` 4, `UIClipsToBounds`, its
  constraints (ordered as a container's), its own layout margins (two
  probes), the margins as the cell's,
  `UIViewInsetsLayoutMarginsFromSafeArea` 0 when it says NO, the semantic
  content attribute, large content.
- The separator is a point high at the cell's foot, from 8 points in to
  its right edge, whatever the table's separator style or the device;
  a cell's `<inset key="separatorInset">` moves its start right by its
  left inset and is `UISeparatorInset` (top, left, bottom, right) after
  `UIIndentationWidth` (four probes, a zero inset among them). A cell's
  content view without a frame has the missing frame (see subviews),
  whatever the cell's (four probes).

Refused for now: static tables, cells with a style (Basic, Subtitle and
the rest), actions in cells, tables as subviews, header and footer views,
refresh controls, and other table and cell settings.

A table view can also be a subview of a controller's view: the same values
as a table view controller's (one probe), with a subview's mask (36) when
it gives none and its `UIViewDoesNotTranslateAutoresizingMaskIntoConstraints`
and own constraints after `UIClipsToBounds`; its outlets go to the
controller's placeholder. Its cells' segues are refused. A xib's table,
which has no `dataMode`, compiles as a prototypes table (one probe).

A controller's `<extendedEdge key="edgesForExtendedLayout">` is
`UIKeyEdgesForExtendedLayout`, a UIRectEdge (top 1, left 2, bottom 4,
right 8), left out for all four; it follows `UIStoryboardSegueTemplates`
and precedes `UIKeyAutomaticallyAdjustsScrollViewInsets` (eight probes).
Simulated metrics change nothing on table and collection view
controllers either.

A table view controller's table can be static (`dataMode="static"`): its
cells, custom ones, are not nibs of their own but objects of the table's
nib, held by a `UITableViewDataSource` after the table:
`UITableSections`, each a `UITableViewSection` with
`UITableSectionHeaderTitle` and `UITableSectionFooterTitle` if any and
`UITableSectionRows`, each a `UITableViewRow` with `UITableRowCell` (the
cell as a prototype's nib has it, without a reuse identifier) and
`UITableRowHeight`, a float (the cell's `rowHeight`, or -1). The objects
list the data source, the sections, the cells, then each cell's content
view and its subviews, then breadth first across the cells their
constraints and deeper views, each cell's content view and its subviews
walked together before the next cell's (one corpus storyboard). The connections begin with the owner's
`staticDataSource` outlet to the data source; the controller's outlets
may reach the cells' views (seven probes, and two corpus storyboards).
The cells' controls' actions on the controller follow the outlets, by
event mask and then selector, those sharing both in the cells' order
whatever their ids; their switches' frame pairs are the table nib's, cell
by cell (seven probes, one corpus storyboard). Two actions in one cell
sharing a mask and selector are refused.
Without `colorMatched`, a grey white background stays as it is rather
than becoming the table's or text view's system background.

A table's header and footer views (`<view key="tableHeaderView">`,
`key="tableFooterView"`, or a stack view there) are its subviews, the footer view first, and the
table's `UITableHeaderView` and `UITableFooterView`, after
`UIInsetsContentViewsToSafeArea`. Xcode lays them out itself: the header
view at the top, as wide as the table is laid out, whatever its frame
says, an older document's frame inset (`<frame key="frameInset"
height="125" maxX="225"/>`, with or without `minX` and `minY`) giving it
only its height (six probes and a corpus storyboard); the footer view
keeps its x, width and height (a frame inset with `maxX` is refused: one
probe has it take the table's width less both insets) and is right under the
header view (or at the top) in a plain table, or in a grouped or inset
grouped one under the header view (35 points without one) and a section
footer height (17 when automatic). Rows, section headers and estimates
don't move it. The objects list goes breadth first through the table's
views and a static table's data source together, level by level, the
table's views first in each. A container view in the header view embeds
as in any view: its segue's placeholder follows the controller's, before
the prototypes' or static cells' whatever the document order, its
template comes first in the controller's nib, its destination loads
first, and its `containerView` outlet is among the table nib's (five
probes; container views in a footer are refused). A table's `separatorInset` is
`UISeparatorInset` (top, left, bottom, right), even at the default,
before `UISeparatorInsetReference` (36 probes in all).

A view's outlets to another view of its nib, or to the controller, are
sorted by name with the controller's (the owner's `view` outlet before a
controller's explicit one: a corpus storyboard); one between views uses no
placeholder. One to a constraint of the nib is as a controller's (one
probe). A plain view may be `opaque="NO"` (no `UIOpaque`) or clip its
subviews (`UIClipsToBounds`, after its content mode), and its
`fixedFrame` changes nothing (seven probes), nor does a text field's
(one probe).

A static cell's segue has a placeholder in the table's nib, after the
controller's: section by section, each section's cells in reverse. The
cell's `selectionSegueTemplate` outlets go in their placeholders' order.
The controller's nib lists the templates in document order (its external
objects in placeholder order). A static cell's outlets to its own views
are the table nib's, collected in that same order (cells in reverse within
each section), which decides between outlets of the same name: one corpus
storyboard and two probes, its cells swapped (18 probes).

A cell may have a built-in style (`style="IBUITableViewCellStyle…"`):
`UITableViewCellStyle` (Value1 1, Value2 2, Subtitle 3, the default left
out) after its reuse identifier, and `UITextLabel` and
`UIDetailTextLabel` referring to the content view's labels named by
`textLabel` and `detailTextLabel`, which are `UITableViewLabel`s. The
objects list the detail label before the text label wherever the
document has them (12 probes). Labels, there and elsewhere, may:
- have no hugging priorities, and then no `UIViewContentHuggingPriority`;
- set `multipleTouchEnabled` (`UIMultipleTouchEnabled`, after
  `UIUserInteractionDisabled`);
- turn off `insetsLayoutMarginsFromSafeArea`
  (`UIViewInsetsLayoutMarginsFromSafeArea` 0, after the hugging
  priorities).

The objects list a view deeper than the root's subviews that the root
view's constraints name with the root's subviews, after them, in the order
of those constraints, and not again under its own superview (ten probes).

A controller's outlet collections (`<outletCollection>`, `collectionClass`
NSArray by default or NSMutableArray) are `UIRuntimeOutletCollectionConnection`s
from its placeholder, before its outlets, each property's in the order of
its first entry: the views (or a single constraint, as an outlet's: one
probe) in a mutable array in document order, `runtimeCollectionClassName`
the class and `addsContentToExistingCollection` false. Those of table and
collection view controllers are refused. Beside gesture recognizers, Xcode
orders several collections' connections among the recognizers', and so
numbers their placeholders, and orders several constraints in one
collection, as a hash table iterates: three or four different outputs in
four to six runs of one document (four probes and a corpus storyboard's
two runs). This ibtool always puts the collections first, the
controller's placeholder numbered 1 and the recognizers' after it, one of
the orders Xcode makes. With a single collection Xcode's order is stable
from run to run but not established (two probes, ten runs each), and it
is refused, as are collections beside recognizers and custom objects or
in an embedded controller.

A view's own outlet collections (other than `gestureRecognizers`) are
connections from the view in the same form, after the controller's, in
the order of their first entries (a corpus xib, a probe); beside the
controller's or recognizers' collections, and in cells, headers and
footers, they are refused.

A button may be disabled (`UIDisabled`) and aligned other than at the centre:
`UIContentVerticalAlignment` and `UIContentVerticalAlignment2` (top 1,
bottom 2, fill 3), then `UIContentHorizontalAlignment` (left 1, right 2,
fill 3, leading 4, trailing 5), after its large content properties (ten
probes). Its content, title and image insets (`<inset>`, minimum and
maximum x and y) are `UIContentEdgeInsets`, `UITitleEdgeInsets` and
`UIImageEdgeInsets` (top, left, bottom, right) after its font (five
probes). A view's, label's, table's, text field's or table cell's
`restorationIdentifier` is `UIRestorationIdentifier`, just before
`UIViewSemanticContentAttribute` (six probes). A text field may also be
hidden (`UIHidden`, after `UIDeepDrawRect`; one probe). A collection view
in a view may page (`UIPagingEnabled`, after
`UIScrollViewIndicatorInsetAdjustmentBehavior`; one probe).

A scroll view is compiled as a plain view, `UIScrollView` (or swapped for its custom
class), then after its large content properties
`UIScrollViewIndicatorInsetAdjustmentBehavior` 1, `UIShowsHorizontalScrollIndicator`
and `UIShowsVerticalScrollIndicator` false when hidden, `UIPagingEnabled`
when paging, `UIBouncesZoom` (left out for `bouncesZoom="NO"`),
`UIAlwaysBounceHorizontal` (three probes; with a vertical bounce too it is
refused), `UIAlwaysBounceVertical`, `UIDirectionalLockEnabled`, the minimum and maximum
zoom scales (floats) and `UIScrollViewContentInsetAdjustmentBehavior` 0
(eight probes). A controller's root view may be one (`<scrollView
key="view">`), with the same values after its large content (a corpus
storyboard); its content and frame layout guides are refused. Its content and frame layout guides are each a
placeholder `UILayoutGuide` at the start of its turn in the objects,
whether or not a constraint uses them, and nothing else when none does.
Its subviews' constraints to them (held by the scroll view, the guide
their second item) are ordered as constraints between siblings; a guide
is defined where a constraint first uses it: the content guide a
`_UIScrollViewLayoutGuide`, the frame guide a `UILayoutGuide`, owned and
locked by the scroll view, not archived, named
`UIScrollView-contentLayoutGuide` or `-frameLayoutGuide`, with system
constraints to the scroll view (the content guide's minimum x and y, the
frame guide's those and its width and height, each named after the guide
and attribute, not archived). Three probes and 210 random storyboards (44
with scroll views); a guide as a first item, and guides in cells,
headers or table views (whose nibs don't list them), are refused.

Interface Builder's `misplaced` and `ambiguous` flags and placeholder
constraints compile to nothing, though a view a placeholder constraint is
on still doesn't translate its mask; a constraint's `identifier` is
`NSLayoutIdentifier`, after its priority and before the legacy constant.
The root view takes the margin keys a plain view does, after its layout
guides, and `layoutMarginsFollowReadableWidth`
(`UIViewLayoutMarginsFollowReadableWidth`, after the margins, for views
and stack views too); it may be non-opaque, not clear its context
(`UIClearsContextBeforeDrawing` after its mask) or have no background (15
probes). A label's compression resistance is
`UIViewContentCompressionResistancePriority` (`{h, v}`, each 750 unless
given) after its hugging priorities; labels and buttons take user-defined
runtime attributes as views do; a disabled slider is `UIDisabled` before its
value; a stack view's standard spacing (`spacingType`) is archived as the least
normal float (1.1754944e-38), whatever its spacing (three probes, from a
deployment target of iOS 11; corpus storyboards).

A button's states are its `UIButtonStatefulContent`, keyed by
UIControlState in that order whatever the document's (normal 0, which it
needs, highlighted 1, disabled 2, selected 4, focused 8: two probes and a
corpus storyboard), each a `UIButtonContent` of
its title, image, background image (`UIBackgroundImage`) and colours (ten
probes; a localizable build's titles for other states than normal are
refused). A button may be hidden, tagged (both after `UIOpaque`) and
have an alpha (after its mask), as a plain view (two probes), and given
hugging (each 250 unless given) or compression resistance (each 750
unless given) priorities, written when given, before its semantic content
attribute (three probes). A plain view's and a text field's are written
the same way, after their constraint keys and hugging, as `{h, v}` (three
probes and 100 random storyboards), a plain view's before its margins
(layout margins, preserving its superview's, following the readable
width and the safe-area insets setting: four probes); beside a plain
view's scrolling or restoration identifier, or a text field's
restoration identifier, they are refused. A selected button has `UISelected` before
`UIDisabled`, and a line break mode other than middle truncation is
`UILineBreakMode` (word wrap 0, tail truncation 4) after its stateful
content (four probes). A button may have no states at all, and then no
`UIButtonStatefulContent`, or only others than normal (two probes); a
content mode other than scale to fill is `UIContentMode` after its mask
(one probe). Showing touches when highlighted is
`UIShowsTouchWhenHighlighted` after its stateful content, before
`UIAdjustsImageWhenHighlighted` (two probes; beside a line break mode it is
refused, and an info button differs in its hugging and font).
`adjustsImageWhenHighlighted="NO"` leaves `UIAdjustsImageWhenHighlighted`
out, and adjusting the image size for accessibility is
`UIAdjustsImageSizeForAccessibilityContentSizeCategory` after
`UIAdjustsImageWhenDisabled` (one probe).

A label's, plain view's, button's, text view's or control's (a switch or
slider: three probes) `<accessibility>` is a
`UIRuntimeAccessibilityConfiguration` in the nib's
`UINibAccessibilityConfigurationsKey`, several views' (the root view's
too, first: one probe) in the order Xcode visits the views for actions
(below): a storyboard's root view's subviews in document order, then from
the last visited each one's own (a stack); a xib's in document order, each
view before its subviews. That is the objects' order but where
constraints pull views up the objects list, which doesn't move them here
(six probes of siblings and containers, nine of pulled views, 100 random
trees as storyboards and xibs, and a corpus storyboard):
`UIAccessibileObject` (sic), then those given of `UIAccessibilityHint`,
`UIAccessibilityIdentifier`, `UIAccessibilityLabel`, `UIAccessibilityTraits`
(an `NSNumber` of UIAccessibilityTraits' bits: button 1, link 2, image 4,
static text 64, not enabled 256, header 65536, none 0) and
`UIIsAccessibilityElement` (0 or 1); equal numbers are one object (14
probes). In `Base.lproj` the hint and label are `NSLocalizableString`s
keyed `<id>.accessibilityHint` and `<id>.accessibilityLabel`, as titles
are; the identifier stays a string (one probe). Runtime attributes, switches (their frame pair) and accessibility
settings inside table headers, and accessibility settings inside cells, are
refused, but for a prototype cell's own accessibility configuration (three
probes, a corpus storyboard: it is the cell nib's, for the cell) when its
views have none; a prototype cell's runtime attributes and switches are its
nib's pairs (see above). Several switches' frames in a
controller's view are refused too: three in a corpus storyboard came
third, first, second.

A button with a plain `<buttonConfiguration>` and a title has no
`UIButtonStatefulContent` or `UIFont`; its `UIButtonConfiguration` is
UIKit's default configuration (style 0, a `UIBackgroundConfiguration` with
a clear background and stroke, corner radius 17 whatever the button's
size, stroke width 1; image placement 2, image padding 0, title padding
1), its title an `NSAttributedString` with the empty attributes,
localizable as `<id>.configuration.title`, and an empty subtitle (four
probes and one in `Base.lproj`). The gray, tinted and filled styles are
`UIButtonConfigurationButtonStyle` 1, 2 and 3, their backgrounds an sRGB
colour instead of the clear one: grey (120, 120 and 128 of 255) at alpha
0.16, the tint blue (0, 136 of 255, 1) at 0.18, and that blue opaque; the
stroke stays a clear colour of its own (four probes). A system symbol
(`image`, `catalog="system"`) is `UIButtonConfigurationImage`, a symbol
placeholder (two probes). A `<backgroundConfiguration key="background">`
sets the background's `cornerRadius` and its colour (`backgroundColor`,
in place of the clear or style's colour, a tinted button's too), each
with its `hasCustomized-` flag true; a `baseForegroundColor` is
`UIButtonConfigurationBaseForegroundColor` (seven probes and a corpus
storyboard). A `baseBackgroundColor`, which also becomes the background's
colour without the flag, other settings and images are refused; 200
random storyboards with configured buttons were byte-identical.

Below iOS 15.0 a configured button also archives its normal state as an
unconfigured one does (`UIButtonStatefulContent`, after its large content
properties), titled with the configuration's title whatever the state's
(localizable as `<id>.normalTitle`), and `UIImageEdgeInsets` before
`UIButtonConfiguration`: top, left and bottom 0 and right the smallest
normal double, `DBL_MIN` (a probe at 13.0 to 15.0 and four corpus
documents at 12.0). Other states, no title, and insets are refused there,
as is a configured button without a deployment target.

Buttons take a custom class, swapped in after `UIButtonConfiguration`
(two probes), and labels one swapped in last (two probes). Stack views and visual effect views take runtime
attributes (two probes). Image views take a custom class (swapped in, after their image), runtime
attributes and accessibility settings; text fields accessibility settings;
the root view a custom class (swapped in, after its large content
properties). A label may adjust its font for the content size category
(`UIAdjustsFontForContentSizeCategory`, after its text) and have a
highlighted colour (`UIHighlightedColor`, after its text colour); a button
may clip its subviews (`UIClipsToBounds`, after its mask; with a tint colour
refused) and give a state an image (`UIImage`, an image placeholder, after
its title) (eleven probes).

A table view controller's refresh control (disabled, as Interface Builder
writes it) is its table's `UIRefreshControl`, after
`UIAlwaysBounceVertical`: a `UIRefreshControl` of its frame, multiple
touches, content mode 4, `UIDisabled` and `UIRefreshControlStyle` 2, after
the table in the objects, its value-changed actions event connections
(mask 4096) after the outlets; in a static table, beside a keyboard
dismiss mode it is refused (two probes). Beside header and footer views
it compiles as without them (two corpus storyboards). Without a
frame it is 1000x1000 at the origin. Its attributed title (one fragment)
is `UIAttributedTitle` after the style: an `NSAttributedString` of the
text and attributes `NSFont`, the system font (11 points for
`metaFont="smallSystem"`, 13 for `message` and `system` unless sized), then
`NSParagraphStyle`, an `NSParagraphStyle` of its alignment (left left out,
right 1, centre 2, natural 4) and nil `NSTabStops`, when the document has
one (eight probes, two corpus storyboards). Other fonts, attributes and
paragraph settings are refused.

A collection view in a view is compiled as a collection view controller's
(mask 36 when none is given, its constraint keys after its clipping), its
outlets the controller's, its flow layout listed as its child (after the
collection view's siblings) and its automatic estimated item size a pair
of the layout's; its cells' segues are refused (three probes). Constraints
it holds are `UIViewAutolayoutConstraints` after its translation flag,
and the objects list its layout first in its turn, before them, in a
view or a cell (three probes and a corpus xib). A custom class swaps in for it, its class names last (one
probe). With `opaque="NO"` it leaves out `UIOpaque`; with
`dataMode="none"` its nil `UIDelegate`; zoom scales are
`UIMinimumZoomScale` and `UIMaximumZoomScale` after `UIBouncesZoom`, as a
scroll view's (four probes).

A search bar, in a view or as a table's header view (laid out as a header
view is), is a `UISearchBar`: `UIOpaque` when not translucent, its mask
(36 by default), content mode 3 (redraw), hugging `{250, 250}`, then after
its large content properties `UIBarStyle` 1 for black,
`UISearchBarTranslucence` (1, or 2 when not translucent), `UIText` (empty
when it has none), `UIPlaceholder`, `UIEnablesReturnKeyAutomatically` and
`UISecureTextEntry` false and `UISearchBarStyle` 2 for minimal; its delegate
outlet is an outlet like any other (six probes). As a table controller's
header view, its delegate outlet to the controller is in the table's nib,
before the table's own outlet of that name (two probes and a corpus
storyboard).

A segmented control (plain style, left and top content alignment) is a
`UISegmentedControl`: its segments are its `UISubviews`, in order but for
the selected one, which comes last; each a `UISegment` with bounds 0x32 at
(0, 16) whatever the control's size, `UIDeepDrawRect`,
`UIAutoresizeSubviews`, the semantic and large content keys, its title as
`UISegmentInfo` (localizable as `<id>.segmentTitles[i]`),
`UISegmentStyle` 5 and `UISegmentPosition` by index (a background colour
after the segments: one probe): none for the first,
3 for the second, 1 for the rest. Then the control's mask 36, constraint
keys, its hugging (250 each unless given: one probe), the alignments (vertical 1 and 1, horizontal
1), `UIPointerInteractionEnabled`, `UISegments` (the segments in order)
and `UISelectedSegmentIndex`; value-changed actions as other controls'
(24 probes, every selection of two to five segments, three heights, and
one in `Base.lproj`). The segments aren't in the objects list, and
`fixedFrame` beside a frame changes nothing (one probe), nor does the
deprecated bar style (two probes). A segment's fixed width is
`UISegmentWidth`, single precision, after its `UISegmentInfo`, leaving
its bounds alone (one probe).

A storyboard's date picker's date isn't archived: a `UIDatePicker` has the view keys,
then `UIDatePickerMode` unless date and time (time 0, date 1, count-down
timer 3), `UIPickerStyle` 3 for inline, `UIMinuteInterval` (0 unless
given, left out at 1, UIDatePicker's default: three probes) and, for a
count-down timer, `UIDatePickerUseCurrentDateDuringDecoding` false (eight
probes; Xcode lays the wheels style out at its own width, so it is
refused). A xib archives the document's date as `UIDate` (an `NSDate`,
`NS.time` the document's `timeIntervalSinceReferenceDate`), after the mode
and before the interval, none for a count-down timer (three probes). A
storyboard with a `<deployment>` naming a version (iOS 8 on), or
deploying before iOS 14.0 (a corpus storyboard at 12.0 to 15.0), archives
one in the same place, but the time it is compiled, which differs on every
run (fifteen probes): this ibtool archives the document's date there too,
so its output is reproducible and has Xcode's shape, differing only in a
value Xcode's own runs disagree on (a corpus storyboard).

A controller's view may be a SceneKit view (`<sceneKitView key="view">`,
Xcode's game templates): an `SCNView` with a root view's keys (white,
`whiteColor`, without a background), then SceneKit's defaults as an
`ARSCNView`'s, below, its `backgroundColor` the same colour object or nil.
It has no layout guides at all, neither a safe-area guide nor the legacy
ones (two corpus storyboards); one listing a guide or holding subviews is
refused.

An ARKit SceneKit view is an `ARSCNView`: its background (white,
`whiteColor`, unless the document gives one), opaque, multiple touch,
`UIAutoresizeSubviews`, its mask as a label's, clipping, its constraint
keys, semantic and large content properties, then SceneKit's defaults
(`loops` true, `playing`, `autoenablesDefaultLighting`,
`jitteringEnabled`, `temporalAntialiasingEnabled` and
`allowsCameraControl` false, `backgroundColor` the document's background
or nil, `ibSceneName` nil, `ibWantsMultisampling` false,
`ibPreferredRenderingAPI` 0): two probes and API-Examples' ARKit
storyboard. One as a controller's root view compiles as a SceneKit view
does, but comes before the nib's placeholders, top-level and in the
objects (a corpus storyboard; the same document with a plain SceneKit
view doesn't: a probe).

An ARKit SpriteKit view (`<arskView>`) is an `ARSKView`: a plain view's
values, then SpriteKit's: `_version` 1, `_frame` (0, 0, its width and
height), an empty mutable `_info` dictionary, `_scene` nil,
`_preferredFramesPerSecond` 60, `_paused` false, `_asynchronous` true,
`_allowsTransparency` and `_ignoresSiblingOrder` false,
`_shouldCullNonVisibleNodes` true (a corpus storyboard). With subviews,
or away from its superview's origin (whether `_frame` is its frame or
its bounds isn't established), it is refused.

A map view is an `MKMapView`: opaque, multiple touch, mask 36, clipping,
its constraint keys, zero `UIViewLayoutMargins`, then MapKit's keys
(`MKDelegate` nil, `MKMapType` 0, zoom, scroll, pitch and rotation on,
user location and traffic off, buildings on, `MKShowsCompass` unless
`showsCompass="NO"`, `MKShowsScaleKey` with `showsScale`, points of
interest on, an `MKStandardMapConfiguration`, `MKPitchButtonVisibility` 1,
no user tracking button); its delegate outlet is an outlet like others
(six probes; other map types and settings are refused).

A toolbar in a view is a `UIToolbar`: `UIDeepDrawRect`, mask 36 (the
document's when it gives one; an empty one is UIToolbar's own 10, flexible
width and top margin: three probes),
`UIClearsContextBeforeDrawing` false when it says so, its constraint keys,
hugging `{250, 250}`, semantic content and large content properties, then
`UIItems` (its bar button items, defined there as a navigation item's
are; with none, the nib's one empty array, where a tab bar's empty items
are an array of their own: one probe, three corpus storyboards),
`UIBarPosition` 1 and `UIBarTranslucence` 1, then the class names when a
custom class swaps in for it (one probe). The objects list its
items in its turn, after its constraints, as its subviews would be (two
probes). An item's action on the controller is an event connection
without a mask, which sorts before the controls' actions (one probe). An
item's custom view is listed in the item's turn, as if its subview, and
its action keeps its mask, sorting with the controls' (four probes).
Items with segues, opaque toolbars and toolbars in tables, cells or
collection views are refused.

A navigation bar in a view is a `UINavigationBar`: opaque unless
translucent (the default; then `UIBarTranslucence` 1, not 2: one probe), mask 36, its constraint keys, hugging `{250, 250}`, its compression
resistance when given (Xcode's Action Extension template), zero
directional layout margins, semantic content and large content
properties, then `UIItems` (an `NSMutableArray` of its navigation items,
each as a controller's with `UINavigationBar` referring back to the bar),
`UIBarPosition` 2 and `UIBarTranslucence` 2. The objects list each item
in the bar's turn, and its bar button items in the item's own turn, after
the views beside the bar (two probes and three of the corpus's: the
probes and two had nothing after the bar to tell the two apart). Its items' actions, on the controller or the first responder,
and a toolbar's, are event connections without a mask (two probes).

A web view is a `WKWebView`: a background colour if any, opaque unless it
says NO, mask 36, its constraint keys, semantic content and large content
properties, then `configuration`, a `WKWebViewConfiguration` of WebKit's
defaults for this SDK (a process pool, preferences, a user content
controller and the default data store, then its flags: data detectors
127, no media types needing a user action, and so on), then
`allowsBackForwardNavigationGestures` false, an empty `customUserAgent`
(the configuration's empty application name), `allowsLinkPreview` true and
`findInteractionEnabled` false (two probes). Only the default
configuration (`mediaTypesRequiringUserActionForPlayback` none, empty
preferences) is compiled; others are refused. `userInteractionEnabled="NO"`
is `UIUserInteractionDisabled` after `UIOpaque`, clipping
`UIClipsToBounds` after the mask (the document's mask, 36 without one, as
a `UIWebView`'s), its outlets (a navigation delegate) as any view's, its
restoration identifier
`UIRestorationIdentifier` before its semantic content, its runtime
attributes key-value pairs as a label's (one probe), a custom class
swaps in for it last, and `fixedFrame` changes nothing (four probes).

The older `<webView>` is a `UIWebView`: a background colour if any,
`UIDeepDrawRect`, `UIOpaque`, `UIAutoresizeSubviews`, the document's mask
(36 without one, left out when empty, whether or not it's constrained),
its constraint keys, semantic content and large content, a nil
`UIDelegate`, `UIScalesPageToFit` when it says so, its outlets (a delegate)
as any view's, and
`UIDataDetectorTypes` (1, phone numbers, without a `<dataDetectorType>`;
with one, phone numbers 1, links 2, addresses 4 and calendar events 8, left
out when none: twelve probes,
`fixedFrame` changing nothing).

A picker view is a `UIPickerView`: the view keys (opaque, hidden, the mask as a label's),
hugging `{250, 250}`, preserving its superview's margins when it says so
(one probe), and after its large content properties
`UIAllowSelectingCells` true; its data source and delegate outlets are
outlets like others (four probes). A visual effect view (non-opaque, with a
blur effect) is a `UIVisualEffectView`: `UIDeepDrawRect`,
`UIUserInteractionDisabled` when it says so (one probe),
`UIAutoresizeSubviews`, mask 36, `UIShouldNotAdaptToMaterials`, clipping,
its not-translating key, no `insetsLayoutMarginsFromSafeArea`
(`UIViewInsetsLayoutMarginsFromSafeArea` 0), then after its large content properties
`UIVisualEffectViewEffect` (a `UIBlurEffect`: extra light 0, light 1, dark 2,
and regular and prominent 1 and 0 with `UIBlurEffectAutomaticStyle` 4 and 5),
`UIVisualEffectViewGroupName` nil and `UIVisualEffectViewContentView`, its
content view as a `_UIVisualEffectContentView` (its subviews, mask 18,
`UIShouldNotAdaptToMaterials`, content mode 4, clipping, its constraints
with its subviews as a container's, then
`UIViewInsetsLayoutMarginsFromSafeArea` 0 when turned off), listed as its
child; the objects list its subviews before its constraints (one probe)
(seven probes; material styles are refused). A vibrancy effect
(`<vibrancyEffect>` holding a blur effect) is a `UIVibrancyEffect` with
`UIVibrancyEffectBlurAutomaticStyle` and `UIVibrancyEffectBlurStyle`, the
blur's values, and tints the content view white (`UITintColor`
`whiteColor` after its clipping: five probes). Its `fixedFrame` changes
nothing, and its mask is 36 whatever the document gives (a corpus
storyboard). The objects list its content
view's subviews right after the content view, in its own turn, before the
constraints it holds (one of the corpus's). Constraints it holds
itself follow its not-translating key, and the objects list them after
its content view (one probe).

A plain view may also be hidden (`UIHidden`), tagged (`UITag`), have no
user interaction (`UIUserInteractionDisabled`) or multiple touches
(`UIMultipleTouchEnabled`), in that order after `UIOpaque`; an `alpha`
(`UIAlpha`, a float) and no `clearsContextBeforeDrawing`
(`UIClearsContextBeforeDrawing` false), in that order after its mask and
before its content mode; and, after its constraints, layout margins
(`UIViewLayoutMargins` top, left, bottom, right, or leading and trailing
with `UIViewLayoutMarginsAreDirectional`, which follows
`UIViewLayoutMarginsFollowReadableWidth` when both are given: two probes
and a corpus storyboard), `preservesSuperviewLayoutMargins`
(`UIViewPreservesSuperviewMargins`, `UIViewPreservesSpecificSuperviewMargins`
15) and no `insetsLayoutMarginsFromSafeArea`
(`UIViewInsetsLayoutMarginsFromSafeArea` 0). `exclusiveTouch` is refused
(13 probes).

Labels take `hidden`, `tag` and `alpha` as plain views do, `UIOpaque`
after `UIDeepDrawRect` unless `opaque="NO"` (one probe), and a
background colour before `UIDeepDrawRect` and a shadow colour
(`UIShadowColor`, before `UIShadowOffset`, which is (0, -1) unless a
`shadowOffset` size gives it: two probes; other colours than text,
highlighted, background and shadow are refused); without a `baselineAdjustment`
(older documents) they have `UIBaselineAdjustment` 1 after the number of
lines (one probe); a missing
hugging priority is 250, and without either there is no
`UIViewContentHuggingPriority` (three probes); their text
may be a `<string key="text">` rather than an attribute. A minimum scale
factor with a text style font scales the style's default size (Body and
Headline 17, Callout 16, Subhead 15, Footnote 13, Caption1 12, Caption2 11,
Title1 28, Title2 22, Title3 20), to a quarter point (19 probes).

Switches, sliders, steppers, activity indicators, progress views and page
controls (42 probes):
- **Common.** Each is a non-opaque view: `UIDeepDrawRect`, hidden and tag,
  `UIAutoresizeSubviews`, the mask (below), a stepper's `UIClipsToBounds`
  when it clips (one probe; beside an alpha it is refused), alpha, the
  constraint keys, then its
  hugging (each priority 250 unless given; a switch's, stepper's or
  activity indicator's left out when it is 750 both ways, a slider's
  always written), its compression resistance if either is given (each
  750 unless given), `UIViewSemanticContentAttribute` and the large
  content properties, before its own values (nine probes). Their
  actions are event connections with their UIControlEvents mask, as a
  button's (value changed 4096; a slider's touch up inside 64: one probe).
  An action may also target the controller's own view, the root view its
  destination (one probe, as a xib's view's); one on another scene's
  object, which Interface Builder no longer makes, is dropped without a
  warning (three probes).
- **A switch.** `UISelected` when selected (a corpus storyboard),
  `UISwitchEnabled` (or `UIDisabled`), `UISwitchOn` when on,
  `UISwitchOnTintColor`. Its frame is also a key-value pair, `frame`
  (`NS.special` 3, `{{x, y}, {w, h}}`, the numbers to 17 significant
  digits as `%.17g` writes them: a corpus storyboard's laid-out
  0.66666666666665719), however it is laid out, before its
  runtime attributes' pairs; the other controls' runtime attributes are
  pairs as any view's (a corpus storyboard).
- **A slider.** `UIValue` (a float) unless 0 or not given, `UIMinValue`
  and `UIMaxValue` unless 0 and 1, `UIContinuous` false, the minimum, maximum and thumb tint
  colours, and `UIBehavioralStyle` 0.
- **A stepper.** A background colour before `UIDeepDrawRect`, a tint after
  the alpha; then `UIDisabled`, `UIValue`, `UIMinimumValue`,
  `UIMaximumValue` and `UIStepValue` (doubles) unless 0, 0, 100 and 1 (a
  missing maximum is 0, so written), `UIContinuous` and `UIAutorepeat`
  false, `UIWraps` true.
- **An activity indicator.** `UIActivityIndicatorViewStyle-Modern` and
  `UIActivityIndicatorViewStyle` (medium 100 and 1, large 101 and 0,
  whiteLarge 0 and 0, white 1 and 1, gray 2 and 2), `UIHidesWhenStopped`
  false unless set, `UIAnimating` (in that order: five probes, every
  style), and `UIColor`. A colour is also
  `UITintColor`, with `UIDisableUpdateColorOnTraitCollectionChange`;
  without one, a modern style's tint is the secondary label colour. One
  hidden when stopped and not animating is `UIHidden`, once if it is
  hidden as well; one hidden in the document is `UIHidden` only if it
  animates or hides when stopped (eight probes, every combination).
- **A progress view** (`UIProgressView`, eight probes, one of the corpus's
  storyboards). Xcode lays it out at least 4 points tall, a bar style one
  point less than its frame (a frame 2 tall is 4, a bar 10 tall 9), from
  the frame's top. Its hugging is left out when it is 250 across and 750
  down, its default. Then `UIProgress` (a float) unless 0, the progress and
  track tint colours (`UIProgressProgressTintColor`,
  `UIProgressTrackTintColor`), and `UIProgressViewStyle` 1 for a bar; its
  tint colour is `UITintColor` after its mask, as a stepper's (a corpus
  storyboard).
  Opaque ones, a bar under 4 points or with colours, and actions are
  refused.
- **A page control** (`UIPageControl`, five probes). Its hugging is always
  written, as a slider's. Then `UINumberOfPages` (which the document must
  give) and `UICurrentPage` (0 unless given), the current page's and the
  other pages' indicator tints (`UICurrentPageIndicatorTintColor`,
  `UIPageIndicatorTintColor`), and `UIHideForSinglePage` when it hides for
  a single page. Its actions are event connections as other controls'.
  Tints beside hiding for a single page are refused.

User-defined runtime attributes of a view, the root view or a controller
are its nib's `UINibKeyValuePair`s (`UIObject`, `UIValue`, `UIKeyPath`), in
document order; a controller's are in its own nib. The pairs of several
views (their runtime attributes and switches' frames) go by each view's
place in the nib's objects list, counted from 1 and compared as decimal
strings: the 10th to 19th objects' before the 2nd's, the 100th's before
the 11th's, a view's own pairs in document order. That explains every
earlier observation: the objects' order while fewer than ten objects come
before them, "flips" when unrelated views pushed one past the tenth, and
nothing from renaming. 436 probes of 2 to 30 views (before and after other
objects, labels and proxies, nested), and nine corpus storyboards (among
them Styles' three switches, third, first, second); accessibility
configurations stay in the objects' order (twelve views). A number is an `NSNumber` (`NS.intval` or `NS.dblval`), a
boolean one of 0 or 1, a string the string, a colour the colour, an image the
`UIImageNibPlaceholder` an image view's image would be, declared or not (two
probes), and a size an
`NSValue` (`NS.special` 2, `NS.sizeval` `{w, h}`). Those of an embedded
controller and of other kinds of view are refused (13 probes).

A label's, button's, picker view's, image view's, plain view's, search
bar's, segmented control's, SceneKit view's or control's (a switch,
slider, stepper, activity indicator, progress view or page control)
`UIAutoresizingMask` is the document's mask (36 without one; left out when
empty, as UIKit leaves out defaults) unless the view doesn't translate it: when it says
`translatesAutoresizingMaskIntoConstraints="NO"` and a constraint is on
it, the mask is 36. `translatesAutoresizingMaskIntoConstraints="NO"`
alone, or `fixedFrame`, doesn't change it (ten probes: labels, buttons and
switches, constrained or not, translating or not, and a corpus
storyboard's switches, steppers and labels; 210 random storyboards of
these views, which first showed image views, plain views and empty masks
following it). Text fields, text views, segmented controls and stack views
follow it too, with hidden, tag and alpha in a view's usual places
(`UIHidden` and `UITag` after `UIDeepDrawRect` and any `UIOpaque`,
`UIAlpha` after the mask). A button's `buttonType="custom"`, a text
field's `borderStyle="none"` and a stack view's `axis="horizontal"` are
the defaults they name (one probe each, and 260 random storyboards of
these views, all byte-identical or refused for several switches). A system font with `size="button"` (older
documents) is 18 points (one probe), with `size="system"` 14 points,
its minimum scale factor relative to that (a corpus xib). A system font of size 0, regular or bold, is 12
points (14 probes).

## Collection view controllers

A `<collectionViewController>` whose collection view has a flow layout and
prototype cells compiles like a table view controller: the collection
view is its view nib's root view, and each prototype is a nib of its own
inside it. Observed with 15 probes and one corpus storyboard:

- A collection view in a view may hide its indicators
  (`UIShowsHorizontalScrollIndicator`, `UIShowsVerticalScrollIndicator`
  false, before `UIPagingEnabled`) and bounce vertically
  (`UIAlwaysBounceVertical`, after `UIBouncesZoom`; four probes).
- The collection view: geometry, background colour (with `colorMatched`,
  an opaque grey white is `systemBackgroundColor`, at the root or in a
  view: nine probes), opaque (left out for `opaque="NO"`, and zoom scales
  after `UIBouncesZoom`, at the root too: a corpus storyboard), multiple
  touch, mask, clipping, then `UIScrollViewIndicatorInsetAdjustmentBehavior`
  1, `UIBouncesZoom`, `UIDelegate` archived as nil,
  `UIScrollViewContentInsetAdjustmentBehavior` 0, `UICollectionLayout`,
  `UIAllowsUserInitiatedMultipleSelection`, the cells' external objects
  (`UICollectionViewCellPrototypeNibExternalObjects`) and nibs
  (`UICollectionViewCellNibDict`), each keyed by reuse identifier in byte
  order, and `UICollectionViewPrefetchingEnabled`. The objects list it and
  then its layout; its outlets are sorted with `view` as a table's are.
- The flow layout (`UICollectionViewFlowLayout`) has `UIInteritemSpacing`
  and `UILineSpacing` as floats when they are not 10, and `UIItemSize`
  when it is not 50x50. `automaticEstimatedItemSize` is not a value of
  the layout but a key-value pair of the nib (`UINibKeyValuePairsKey`):
  `estimatedItemSize` set to an `NSValue` size of the largest doubles.
- A flow layout with a custom class is swapped in after its values (one
  probe); a horizontal one has `UIScrollDirection` 1 after its item size
  (one probe), and a section inset other than zero `UISectionInset` (top,
  left, bottom, right) there too (one probe), before the scroll direction
  when it has both (two probes).
- Section headers and footers (`<collectionReusableView>`,
  `sectionHeaderView` or `sectionFooterView`) are nibs of their own, keyed
  `UICollectionElementKindSectionHeader/<reuse identifier>` (or `Footer`)
  in byte order: an empty external objects table each in
  `UICollectionViewSupplementaryViewPrototypeNibExternalObjects` after the
  cells' and a `UINib` each in `UICollectionViewSupplementaryViewNibDict`
  after theirs. A header's nib holds the owner, the first responder and a
  `UICollectionReusableView` (geometry, subviews, `UIDeepDrawRect`,
  multiple touch, `UIAutoresizeSubviews`, `UIContentMode` 4, clipping,
  semantic content and large content properties, `UIReuseIdentifier`), its
  objects listing an unowned safe-area guide after it where the document
  uses safe areas (none without: a corpus storyboard), then its subviews
  breadth first. The flow layout's non-zero `headerReferenceSize` and
  `footerReferenceSize` are `UIHeaderReferenceSize` and
  `UIFooterReferenceSize` after its item size (five probes). A header's
  background colour is `UIBackgroundColor` after its subviews, its tint
  `UITintColor` after clipping, then its constraints (its own, listed
  before its subviews as a container's); a custom class makes it a
  `UIClassSwapper` with `UIClassName` and `UIOriginalClassName` after its
  reuse identifier; its outlets to its subviews are a cell's, by name (five
  probes). Its actions, other colours and masks are refused, as are
  reference sizes beside a section inset or horizontal scrolling.
- A custom layout (`<collectionViewLayout>` with a custom class) is a
  `UIClassSwapper` of `UICollectionViewLayout` with nothing else, at the
  root or in a view (three probes).
- A cell's nib: the cell (`UICollectionViewCell`, or swapped in) holding
  its content view, a plain `UIView`; the cell's `UIReuseIdentifier`,
  nil `UISystemBackgroundView` and `UIContentConfigurationView`, and
  `UIContentView`. A content view holding constraints does not translate
  its mask, and the cell gets four constraints of its own pinning it
  (leading, trailing, top, bottom, each `UIIBSystemGenerated`), which the
  objects leave out. The objects list the cell and its content view, then
  breadth first each view's constraints and subviews (a table cell lists
  its content view's subviews first). A collection cell's content view's
  mask is `UIAutoresizingMask` after its `UIAutoresizeSubviews` when it has
  one, and its `customSize` changes nothing (three probes); the cell's own
  empty mask is left out (one corpus storyboard). A collection
  cell's background colour is `UIBackgroundColor` after its `UISubviews`
  (two probes). A control's
  action on the cell itself is an event connection after the cell's
  outlets, the cell its destination (one corpus storyboard).
- An old-format content view (`<view key="contentView">`, no id) is not
  the document's: it compiles to a plain opaque view the objects leave
  out, its subviews compiled as usual. Its background colour is left out;
  its mask is 36 without one, the document's when it has one, after
  `UIAutoresizeSubviews` (five probes, four corpus xibs). The constraints the cell holds are
  that view's (the cell's id in them, explicit or implied, is the view,
  margins included). Then the view doesn't translate its mask and is
  pinned to the cell, as a new-format content view holding constraints is.
  The objects list its subviews before its constraints, then go on
  breadth first (six probes, among them two labels, a label's own width,
  and the cell named explicitly). Constraints a new-format cell holds
  itself are refused.
- Cell outlets and a cell's segue work as a table cell's.
- An object without values (a flow layout with every default) is written
  with its first value index 0, not the running index.

## Segues

A segue (other than a relationship) makes its destination loadable: Xcode
compiles a nib for every controller a segue leads to from a loaded scene
(initial or identified, reached by a segue, or a loaded navigation
controller's root), and lists it in the Info.plist as it does identified
ones; a scene nothing loads takes its segues' destinations with it. A
controller, navigation controller or root is swapped in for its class
(`UIClassSwapper`) only when it is initial, identified or of a custom
class: one only segues load is a plain `UIViewController`,
`UITableViewController` or `UINavigationController`, its other values
the same. Observed with 55 probes:

- A segue's template: `UIStoryboardShowSegueTemplate` (show, and
  showDetail with `UIActionName` `showDetailViewController:sender:`
  instead of `showViewController:sender:`),
  `UIStoryboardPresentationSegueTemplate`,
  `UIStoryboardPushSegueTemplate` or, for an old `modal` segue that
  animates, `UIStoryboardModalSegueTemplate` (a probe and a corpus
  storyboard); a custom segue (with a class) a plain
  `UIStoryboardSegueTemplate` without an action name. Its values:
  `UIIdentifier` (the segue's identifier, if any), `UISegueClassName` (a
  custom class's runtime name, for any kind: three probes; in the
  target's module, the `--module` one, as a controller's: one probe),
  `UIDestinationViewControllerIdentifier`
  (the destination's nib's name), then for show `UIAnimates` false when it
  doesn't animate, a segue action's selector (`destinationCreationSelector`)
  as `UICustomPrepareForChildViewControllersSegueName` (one probe; beside
  `animates` or on other kinds it is refused) and `UIActionName`; for
  presentation
  `UIModalPresentationStyle` (fullScreen 0, pageSheet 1, formSheet 2,
  currentContext 3, overFullScreen 5, overCurrentContext 6, popover 7,
  automatic -2), `UIModalTransitionStyle` (crossDissolve 2) or
  `UIAnimates` false; for push `UIDestinationContainmentContext` and
  `UISplitViewControllerIndex`, both 0.
- A button's popover segue (`popoverPresentation`) is a
  `UIStoryboardPopoverPresentationSegueTemplate`: its identifier and
  destination, `UIPermittedArrowDirections` (up 1, down 2, left 4, right 8
  from its `popoverArrowDirection`, 0 without one), then nil
  `UIPassthroughViews`, `UIAnchorBarButtonItem` and `UIAnchorView`; its
  anchor view is an `anchorView` outlet from its placeholder in the view's
  nib, sorted with the outlets (popovers' in their placeholders' order).
  Popovers' placeholders come first, in reverse document order, then the
  other segues' in document order, the templates listed in document order
  all the same (ten probes, CodeExamples' AdaptivePopover). A popover
  from a controller's own navigation item's bar button item, anchored on
  that item (`popoverAnchorBarButtonItem`), has the same template in the
  controller's nib, performed by the item, and an `anchorBarButtonItem`
  outlet from the template to the item after the controller's outlets and
  before `sceneViewController` (three probes). Popovers from cells or the
  controller itself, without an anchor, not animating or beside container
  views are refused.
- In a controller's own nib, its outlets to its navigation item's bar
  button items are there too, by name after its storyboard outlet, and
  its items' actions on the scene's first responder are event connections
  to the first responder proxy, sorted by selector with the others (five
  probes). Beside outlets to recognizers, custom views or custom objects,
  in an embedded controller and beside gesture recognizers they are
  refused.
- A controller's freeform size is ignored on a device Xcode doesn't know
  (ipad10_5, ipad12_9) from iOS 17.0 on: the view keeps its frame (three
  probes). Below 17.0, on the generic canvas there, it is the view's size
  (a corpus storyboard).
- A control's segue (a button's, or a prototype cell's) is triggered from
  its view nib through a placeholder of its own: the view nib has
  `UpstreamPlaceholder-<n>` proxies, the controller's first if the view
  refers to it (an outlet or an action to it), then one per segue in
  document order (the controller's external objects table lists them by
  name as strings, `UpstreamPlaceholder-10` before `-2`: RxSwift's example,
  16 placeholders), and a button's segue is a `perform:` action (touch up
  inside, 64) on its placeholder, sorted with the other actions. Actions
  go by event mask, then selector, then document order. A control's action
  on the scene's first responder goes to the `IBFirstResponder` proxy and
  doesn't bring in the controller's placeholder; a button's events are
  UIControlEvents from touch down (1) to touch cancel (256), value
  changed (4096) and primary action triggered (8192). Xcode writes
  `menuActionTriggered` with no mask, so it is refused (13 probes). The controller's
  external objects table maps each placeholder to the controller or the
  segue's template. Xcode keeps only a control's last segue (refused).
- The controller's nib (or, for a navigation controller's root, the
  navigation controller's) holds `UIStoryboardSegueTemplates`: the
  controller's own segues (each with an identifier), then its controls',
  each in document order. In a controller's nib the key follows the
  identifiers and precedes `UIKeyAutomaticallyAdjustsScrollViewInsets`;
  in a root's, it follows `UIParentViewController`. The objects list the
  templates in that order after the controller (and its navigation item,
  or after the root's), and each has a `viewController` outlet to the
  controller after the others: in the reverse of the array's order, then
  stably by class name (Presentation, Push, Show), as 18 probes of up to
  eight templates of mixed kinds show.
- A prototype cell's segue: the table's nib has the segue's placeholder,
  and the cell's external objects map the cell nib's
  `UpstreamPlaceholder-1` to it; the cell nib has that placeholder after
  the first responder (also in the objects) and a
  `selectionSegueTemplate` outlet from the cell to it, sorted with the
  cell's outlets.

An unwind segue leads to its scene's `<exit>` and loads nothing; one to
another scene's exit compiles the same (a probe). Its
template is a `UIStoryboardUnwindSegueTemplate`: `UIIdentifier` if any,
`UIDestinationViewControllerIdentifier` archived as nil, `UIAnimates`
false when it doesn't animate, and `UIAction` (the unwind action's
selector). Otherwise it is triggered, listed and connected like any other
template, from buttons, prototype cells or the controller itself, in a
navigation root's nib as well; its `viewController` outlet sorts after a show
template's, by class name. An exit placeholder adds nothing to the
nibs: a scene with one compiles as it does without. Observed with 14
probes.

A container view (`<containerView>`, with one embed segue) is a plain
`UIView` in its scene's view nib, without `UIOpaque` when the document
says `opaque="NO"`, with `UIHidden` when hidden (a corpus storyboard).
Without an embed segue it is only that (two probes, one of a custom
class). Its embed segue gets a placeholder like a control's,
in document order with the others, and instead of an action the
placeholder has a `containerView` outlet to the view, sorted by label
with the view nib's outlets, before a controller's own outlet of that
name (a corpus storyboard). The template is a
`UIStoryboardEmbedSegueTemplate`: `UIIdentifier` if any,
`UIDestinationViewControllerIdentifier`, `UIPerformOnViewLoad` true and
`UIContainerView` archived as nil. A custom class swaps the view as a
plain view's does, and its placeholder's intrinsic size changes nothing
(four probes). The destination, a view or table view
controller, gets its own nib as other segue destinations do. Observed
with 24 probes:

- The embedded controller's root view is laid out at its container's
  frame size, not its own frame's or the device's. So is every
  controller a segue (of any kind) leads to from it, transitively, even
  when another segue reaches it from a full-size controller; nested
  containers use the inner container's size. A controller's own freeform
  size still wins, and is not passed on.
- The Info.plist inserts the controllers' identifiers (into its
  CFDictionary order) depth first from each initial or identified
  controller in document order, through its relationships, its own
  segues and then its view's. It differs from document order only when
  a controller is reached before one that precedes it in the document,
  as a controller's own segue is before its container's. It also explains 48 of the
  64 corpus storyboards with three or more entries; document order
  explains 35, and none that depth first gets wrong.

A storyboard reference (`<viewControllerPlaceholder>`) a segue leads to,
embed segues included, has a nib of its own, named as a view controller's
(`UIViewController-<id>`): the owner, the first responder, the storyboard
placeholder and a `UIStoryboardViewControllerPlaceholder` whose
`UIStoryboardIdentifier` is that name, with only the `sceneViewController`
outlet. It is listed in `UIViewControllerIdentifiersToNibNames` as the
controllers are, and the Info.plist's
`UIViewControllerIdentifiersToExternalStoryboardReferences` maps its name
to `UIReferencedBundleIdentifier` (if any), `UIReferencedControllerIdentifier`
(the referenced identifier, if any) and `UIReferencedStoryboardName`;
equal keys and values in those are one string each. A reference nothing
leads to compiles to nothing. Observed with ten probes. A reference's
empty navigation item is a `UINavigationItem` (an empty
`UICenterItemGroups`, `UILargeTitleDisplayMode` 0) after the placeholder
in its nib's objects; anything else inside a reference is refused. Xcode
adds the references' Info.plist table entries in the order their
placeholders load, as the controllers are listed (four corpus storyboards,
RxSwift's example's 15 references among them: document order, its reverse
and name order don't fit); where one doesn't load and the hashes collide
so that the order would matter, it is refused. The
identifiers' table beside them is filled as without references (a corpus
storyboard with one reference, and RxSwift's 16 identifiers).

Refused for now: custom and popover segues, segues from
other elements (bar button items, gesture recognizers, a cell's
accessory), and to other controllers.

## WatchKit storyboards

Compile to `Interface.storyboardc/Interface.plist`, a binary property list
(`controllers`: per controller `controller-<id>`, its class and items;
`root`), which linking writes to the link directory itself. Each item is a
dictionary of its `type` and properties (a label's `text`, `numberOfLines`,
`fontScale` 1, a text style's WatchKit name as `fontSize` and `fontSize2`,
`color`, an outlet's `property`; a group's `width`, alignments, `layout`,
`items`). A label's `alignment` is left out when `left` and is the integer
3 when the document has none; one line is left out; its keys go in in
the order `fontSize2`, `color`, `fontSize`, `text`, `property`, `type`,
`fontScale`, `alignment`, `numberOfLines`, which reproduces all 47 key
orders of 80 random probes' labels (any subset of those keys). A controller's class is its runtime name: a Swift class of a
custom module mangled (`_TtC5Other19InterfaceController`), the target's
module (`customModuleProvider="target"`) being `--module`'s; a controller
without items has no `items` (four probes and a corpus storyboard). A
calibrated white of 2/3 is recorded as `B8`; other colours are not
established.

The writer shares an object between equal strings and numbers only where
Interface Builder's writer does (by identity: everything here except the
controller name, written once as a key and once as `root`), and Info.plist
shares none. Dictionary keys enumerate in CoreFoundation's order for the
order they were inserted (foundation-hashing.md): for the compiled plist,
an order verified only for these key sets, so other items are refused;
linking reads the plist back and writes it again, each dictionary created
whole from its keys in the order read, which reproduces the linked file.

## AppKit

A macOS document (`targetRuntime="MacOSX.Cocoa"`) compiles to a NIBArchive
like UIKit's, without UIKit's `LNE` trailer. Observed with Xcode 27.2,
compiling for `--target-device mac`, by probes and a corpus of 124
documents (Xcode's macOS templates and real projects' xibs and
storyboards).

**The object data.** The root is an `NSObject` holding `IB.objectdata` and
`IB.systemFontUpdateVersion` 1. Its `NSIBObjectData` holds, in order:
- `NSRoot`: the File's Owner, an `NSCustomObject` of its class (`NSObject`
  without one; a custom class adds an `IBClassReference` of its name,
  module and module provider);
- `NSVisibleWindows`: a set of the windows not `visibleAtLaunch="NO"` (a
  storyboard's never: its window controllers show them);
- `NSConnections`;
- `NSObjectsKeys` and `NSObjectsValues`: every object and its parent, in
  depth-first order (a view, its subviews and their constraints, then its
  own constraints; a menu, each item and its submenu);
- `NSOidsKeys` and `NSOidsValues`: the root, the objects and the
  connections, numbered from 1;
- the accessibility connectors (empty) and two keys sharing one empty
  array.

Every object is defined where the encoder first meets it. A menu item's
`NSMenu` is a conditional reference: it doesn't define the menu.

**Placeholders.** The First Responder isn't archived; actions targeting it
leave `NSDestination` out. The Application placeholder is an
`NSCustomObject` named `NSApplication`, referring to its class
(`IBClassReference` of `NSObject`) when that is its class. A custom object
of an AppKit class (`NSFontManager`) is an `NSCustomObject` of it; one of
the app's own classes is an `NSClassSwapper` for `NSObject`, made with
`init` (`NSInitializeWithInit`) when the document says
`customObjectInstantitationMethod="direct"`.

**Connections**, from any object including views, in a xib: outlets, then
actions, each by their source's id as bytes (Xcode's main menu template's
87); one source's outlets by their labels as bytes (four probes), its
actions in document order. A storyboard's nibs list them in an order that varies from
run to run; this ibtool keeps the xib's. `toggleSourceList:` is archived
as `toggleSidebar:`.

**Windows** are `NSWindowTemplate`s: the style mask; backing 2; the content
rectangle, centred on the screen rectangle along an axis without a strut;
`NSWTFlags`; title, an empty subtitle, the class; the content sizes'
limits, then the content view, the screen rectangle, the frame's limits
(the content's plus a 32-point title bar when titled), the autosave name,
the collection behaviour (`NSWindowCollectionBehavior`'s bits: can join
all spaces 1, move to active space 2, managed 4, transient 8, stationary
16, participates in cycle 32, ignores cycle 64, full screen primary 128,
auxiliary 256, none 512, allows tiling 2048, disallows tiling 4096),
restorability and, in a xib, empty full-screen sizes. `NSWTFlags`: deferred
0x20000000 (the default), not released when closed 0x40000000, shadowless
0x1000, hidden on deactivation 0x80000000, tooltips while inactive 0x2000
and recalculating the key view loop 0x800 (defaults), and the initial
position's struts in bits 19 to 22: per axis both bits without a strut or
with both, one with one (left keeps 0x80000, right 0x100000, top 0x200000,
bottom 0x400000), and none for a left and a bottom strut alone (64
probes). A custom window class adds its `IBClassReference` last.

**Views.** An `NSView`'s keys: next responder, touch bar, `NSvFlags` (256
and the autoresizing mask; without a mask, flexible right and bottom
margins), subviews, `NSFrameSize` at the origin or `NSFrame`, superview, layer
hosting, best-resolution OpenGL, `NSDoNotTranslateAutoresizingMask` when it
doesn't translate its mask and constraints refer to it, its constraints,
content hugging and compression resistance as strings when other than the
class's own (250 each, a text field's vertical hugging 750; 750 each; a
priority the document leaves out is Interface Builder's default, 250 or
750, not the class's), and
the safe-area, margins and clipping keys. A custom view, or a view of a
custom class, is an `NSClassSwapper` for it.

**Own classes.** A custom class naming the element's own AppKit class is
no custom class, with or without a module: Xcode compiles the document as
though the class weren't there. This holds for a view's or custom view's
`NSView` in both kinds of xib, a button's `NSButton`, a window's
`NSWindow`, a text field cell's `NSTextFieldCell` and a content
controller's `NSViewController` (about a dozen probes). A plain view naming
`NSView` is an `NSView`, not a class swapper; a custom view is swapped (or
an `NSCustomView`) all the same, with no class reference.

**Constraints** (`NSLayoutConstraint`): items, attributes twice
(NSLayoutAttribute), the relation, the constant twice when given (a
symbolic one, the standard space, as `NSSymbolicConstant` `NSSpace` in their
place), the priority when not required; a multiplier (`NSMultiplier`, after
the relation; a ratio `a:b` as its quotient) rounded to a float and
archived as a double. A view's are in an order of their own (about
60 probes and a corpus storyboard, none from the document's order):
- first the view's own, relating none of its subviews;
- then each subview's (those whose first item it is, or whose second
  when the view is the first), nearest the view's top-left corner first,
  by the distance from it to the top-left corner of the subview's frame;
- within a subview's: those relating it to the view, and its size, by
  the view's attribute (the subview's for a size or a baseline), in the
  order centerX, centerY, right, left, trailing, leading, width, bottom,
  top, firstBaseline, baseline, height;
- then those relating it to other subviews, by its own attribute
  (centerX, trailing, leading, centerY, bottom, top, firstBaseline,
  baseline, width, height; a baseline only to a baseline), then
  the other subview's, then the other subview farthest from it first, by
  the distance between their centres (four ties in two corpus
  storyboards that their corners don't order).

Subviews as far from the corner go leftmost first, and at the same corner
by their ids as bytes. Refused as not established: two different constraints on the same attributes of the same
views or of subviews as far apart, and between subviews left and right,
or a centre or size related to another attribute. A view controller's own view never says
`NSDoNotTranslateAutoresizingMask`.

**Layout.** Xcode lays constrained views out again with Auto Layout as it
compiles, with the Mac's control metrics and text measurement: 133 of the
corpus's 227 text field and button frames differ from their documents'.
Strict, this ibtool refuses constrained controls sized by their content,
and constraints that the document's frames don't satisfy (a view
Interface Builder shows misplaced, which probes show Xcode moving; Auto
Layout measures left to right and down from the view's top);
`--config=relax_ibtool_layout` compiles them with the document's frames,
which only their first layout at run time uses.

**Menus.** An `NSMenu`: title, items, and a system menu's name
(`_NSMainMenu`, `_NSAppleMenu`, `_NSServicesMenu`, `_NSRecentDocumentsMenu`,
`_NSFontMenu`, `_NSWindowsMenu`, `_NSHelpMenu`). An `NSMenuItem`: its menu,
disabled and separator flags, key equivalent localization and mirroring,
title, key equivalent, its modifiers (shift 1 << 17, control 1 << 18,
option 1 << 19, command 1 << 20; command for a key equivalent without a
mask), the mnemonic location 0x7fffffff, the system's checkmark and mixed
state images, its tag, and for a submenu `submenuAction:` targeting it. A
storyboard's items opening a submenu are allowed in limited app mode.

**Text fields.** The cell's flags, from 28 probes: `NSCellFlags` 0x4000000,
0x40 unless wrapping, scrollable 0x100000, selectable 0x200001, editable
0x10200001, bezel 0x400000, `state="on"` the top bit; `NSCellFlags2`: the
alignment in bits 26 to 28 (left 0, right 1, center 2, justified 3, natural
4), the line break mode in bits 9 to 11 (word wrapping without one), small
0x20000 and mini 0x40000 sizes, sending on end editing 0x400000, no focus
ring 0x8000, no undo 0x1000, truncating the last line 0x100, single line
0x40. System fonts are `.AppleSystemUIFont…` with each metafont's size and
`NSfFlags`; named fonts flags 16. An appearance font (`usesAppearanceFont`)
is a text field's system font at 13 points with
`NSFontUsesAppearanceFontSize`, whatever its control size, and a button's
plain system font of its control size (13, 11, 9). A control's cell comes
before the control's own constraints in the objects' list. System colours
that resolve to the same colour share it. A text field allowing the Touch
Bar's character picker adds `NSCharacterPickerEnabled` last in its cell;
not allowing it changes nothing. A key equivalent Interface Builder writes
in base64 (`base64-UTF8`, without padding: the escape key) is decoded. A System catalog colour carries the
colour it resolves to, in Generic Gray Gamma 2.2 or sRGB with the profile
embedded: the Mac's own profiles, which the SDK archive carries
(`tools/package-sdks.sh`) rather than this repository.

**Buttons** (around 100 probes). An `NSButton`'s view keys, then
`NSEnabled`, the cell, and the control keys (`NSControlTextAlignment` as
`NSTextAlignment`: left 0, centre 1, right 2). A bordered button of a
rounded kind has its own content hugging, 250 and 750, and archives any
other, even 250 each. Its `NSButtonCell`: the cell flags as a text
field's, the state the top bit; title and font; `NSButtonFlags`, its behaviour (push in
0x80000000, change contents 0x40000000, background 0x20000000, gray
0x10000000, light by contents 0x8000000, background 0x4000000, gray
0x2000000, not dimming the image 0x1000), the image position in bits 19
to 22 (above 1, below 3, right or trailing 5, left or leading 7, only 8,
overlapping 9), a border 0x800000, the inset in bits 13 and up, and 0x100
for a check box or radio button; `NSButtonFlags2`, the bezel's bits, the
image scaling in bits 6 and 7 and the key equivalent's modifiers from bit
8; the bezel style; the check box's and radio button's
`NSButtonImageSource` (`NSSwitch`, `NSRadioButton`); alternate title, key
equivalent, a periodic delay of 400 and interval of 75; and
`NSAuxButtonType`: AppKit's button type for its behaviour, whatever the
`type` says (7 pushing in and lighting by background and gray, 5 lighting
by contents, 6 changing and lighting by background and gray, 1 the same
pushing in, 0 for changing and lighting by contents, lighting by
background and gray, or pushing in and lighting by contents; a check box 3
and a radio button 4). A catalog image is `NSNormalImage`, after the bezel
style. The other behaviours, AppKit's own images and the inline bezel are
refused.

**Boxes** (about 30 probes). A separator or a custom box: `NSBox`'s view
keys, then `NSOffsets` (`{5, 5}` for a separator, `{0, 0}` otherwise),
the title's `NSTextFieldCell` (the small system font, text background and
label colours; a plain `Title` without a title; a storyboard's leaves out
`NSControlView`), the content view, the border type (none 0, line 1,
bezel 2, groove 3, the default), the box type (separator 2, custom 4), the
title position (no title 0 to below bottom 6, at top 2 the default),
`NSTransparent` false and `NSFullyTransparent` when transparent, the
border width as an `NSNumber` (numbers are shared by value: an object id
of the same integer is the same object), the corner radius, border and
fill colours, and `IBNSBoxIsUsingDocumentContentView` with a content view.
Xcode lays a custom box's content view out again, inset from the box's
bounds by its border width (1 unless given, none without a border),
whatever its frame says; in a storyboard it clips to its bounds. Old
documents write the content view without an id; nothing can refer to it.
A box whose subviews don't autoresize clears the view flag and says so
again through an `NSIBUserDefinedRuntimeAttributesConnector` (the box,
`NSValues` holding a shared `NSNumber` false, `NSKeyPaths` holding
`autoresizesSubviews`). These connectors follow the stack views'
connectors and come before every other connection. They are ordered by
the box's id as bytes and numbered with the connections (four probes, a
corpus xib). A plain view with the same attribute gets no connector. A
primary or secondary box puts its content below its title, measured with
the Mac's fonts, and is refused.

**Pop-up buttons** (about 15 probes) are buttons whose
`NSPopUpButtonCell` holds a menu. The cell's flags add border and bezel
0x400000 and disabled 0x20000000 to a button's; push and bevel pop-ups
without pushing in are button type 0. After the button keys: the selected
item (`NSMenuItem`, defined there, or nil), `NSMenuItemRespectAlignment`,
the menu (an untitled `NSMenu` referring to the selected item), the
selected index unless 0 (-1 with none), the preferred edge 1, using the
item from the menu, altering state, and the arrow position (bottom 2,
centre 1). Its items are menu items with command as their modifiers, the
selected item's state on, and `_popUpItemAction:` targeting the cell. A
pop-up cell's menu font is the message font, and its title is a plain
string even where its items' titles are localizable (Base.lproj). A cell's connections come
from the cell.

**Sliders** (about ten probes), linear: vertical when the frame is taller
than wide; their own content hugging is 250 and 750 (750 and 250
vertical), left out; no subviews array. The `NSSliderCell`: state 0x80000000,
continuous 0x40100 and disabled 0x20000000 in `NSCellFlags`, the control
size in `NSCellFlags2`; maximum, minimum and value (the minimum without
one), an alternate increment of 0, the tick marks' count and position
(below or right 0, above or left 1), whether only their values are
allowed, and the orientation. A continuous slider sends its action on
dragging too (`NSControlSendActionMask` 70). A slider's height is its own,
so a constrained one is laid out by Xcode, as a content-sized control is.

**Stack views** (about 20 probes), with a flat hierarchy: their subviews
are the arranged views (which, constrained by the stack, all say
`NSDoNotTranslateAutoresizingMask`), then `NSStackViewOrientation`, the
secondary and primary alignments (horizontal: top 1 and 3, centerY 3 and
10, bottom 4 and 4, firstBaseline 2 and 12; vertical: leading 1 and 5,
centerX 3 and 9, trailing 4 and 6), clipping resistance and hugging,
spacing (with `NSStackViewHasEqualSpacing` for equal spacing), the
distribution (fill 0, fill equally 1, fill proportionally 2, equal spacing
3, equal centering 4) and edge insets, all numbers as floats; an
`NSStackViewContainer` (outside the objects list) holding the arranged
views; whether it detaches hidden views; a flat hierarchy. Custom spacing
isn't archived. Each stack view adds an `NSNibConnector` asking to be
decoded early, first among the connections by the stack's id and without
an object id. A view's next responder and superview are conditional
references, so a stack reached first through its connector has its
superview defined later. A stack lays its views out, so only
`relax_ibtool_layout` compiles it; gravity areas and visibility priorities
other than 1000 are refused.

**Image views** (about 15 probes). An `NSImageView`'s view keys, with
`NSDragTypes` before its frame: a set of eight pasteboard types (PDF,
PICT, PNG, file names, TIFF, file promises, and a dynamic type), always
the same. Then `NSEnabled`, the cell, and the control keys, refusing
first responder as the cell says, editable, and
`NSImageViewPlaceholderPrecedence` 0. Its `NSImageCell`: the text field
cell's flags (refusing first responder 0x2000000 in `NSCellFlags2`), a
named image as an `NSCustomResource` of the size the document's resources
give, the image alignment (center 0, top 1, top left 2, top right 3, left
4, bottom 5, bottom left 6, bottom right 7, right 8), scaling
(proportionally down 0, axes independently 1, none 2, proportionally up or
down 3), frame style (none 0, gray bezel 2), and animation in a storyboard
only. An image
sizes its view, so a constrained image view is laid out by Xcode, as a
content-sized control is. AppKit's own images (`NSAddTemplate`), which
Xcode names in the `system` namespace, are refused.

**Scroll views**, as Xcode archives those it doesn't lay out again: no
border, hidden scrollers that hide themselves, no content insets, and a
document view (a plain or custom view) no larger than the clip view. With
a border, a scroller showing, or a larger document, Xcode tiles it with
AppKit's scroller metrics, and this ibtool refuses it. The scroll view's
subviews are its clip view and two scrollers; `NSsFlags` 0x20a00, with the
horizontal and vertical scrollers 0x20 and 0x10, predominant-axis
scrolling 0x10000, content insets 0x100000, and the scroll elasticity,
vertical in bits 12 and 13 and horizontal in 14 and 15 (automatic 0, none
1, allowed 2); the line and page scroll
amounts as big-endian floats (vertical, then horizontal); magnification
0.25 to 4. The clip view holds the document view at its origin and the
control background colour, drawing it (4) and copying on scroll (2). The
scrollers are layer-backed and target the scroll view with
`_doScroller:`. In a storyboard whose document view is off its clip view's
origin, the vertical scroller comes before the horizontal one among the
scroll view's subviews (not in the objects' list).

**Storyboards.** `Info.plist`: the main menu's nib name `MainMenu` (whether
or not there is one), the controllers' nib names, the entry point, the
window controllers' UUIDs and the version, inserted in that order. The
application scene is `MainMenu.nib`; a window controller
`NSWindowController-<id>.nib`, holding its window, its content view
controller (an `NSClassSwapper` naming its view's nib, or without a class a
plain `NSViewController`; each with its own `storyboard` outlet) and
storyboard placeholders, with the window's delegate its controller whether the
document connects it or not; the content controller's view
`<controller>-view-<view>.nib`. When the controller has outlets, or its
view's objects connect to it, the view's nib stands it in with an
`NSNibExternalObjectPlaceholder` named `UpstreamPlaceholder-1`, after the
Application placeholder, and the controller's
`NSExternalObjectsTableForViewLoading` maps that name to it (an empty
dictionary otherwise). Xcode makes each controller's
`uniqueIdentifierForStoryboardCompilation` anew on every run; this ibtool
derives a UUID from the document and the controller's id.

A window controller with a storyboard identifier is named by it: its nib,
and its Info.plist keys sharing one string, with
`explicitStoryboardIdentifier` after its content controller (Xcode's
document app template). A view controller's view may be a SceneKit view,
an `SCNView` with SceneKit's keys after the view's (`loops`, `playing`,
`jitteringEnabled`, `allowsCameraControl`, `ibWantsMultisampling`,
`ibPreferredRenderingAPI`, `autoenablesDefaultLighting`, `ibSceneName`,
`backgroundColor`), in the form Xcode's SceneKit game templates give it.
An initial view controller (a storyboard whose entry point isn't a
window controller) has its own nib, `NSViewController-<id>`, held as an
identified controller's, and is the Info.plist's entry point (Xcode's
notification content extension template). A text field cell of a custom
class is an `NSClassSwapper` for `NSTextFieldCell`. A xib's Application
placeholder without a class refers to `NSObject`, as one of that class
does. A text field's `NSPreferredMaxLayoutWidth` follows its control keys. A
window controller shown as a single window (presentation style 1) adds
`NSStoryboardSegueDestinationOptions` last, mapping
`NSSingleInstancePresentationIdentifier` to its UUID string.

**Web views.** Xcode archives a live `WKWebView`, so its keys are WebKit's
on the Mac running Xcode (macOS 27 for these notes). In the form Xcode's
Safari extension templates give it (a layer-backed view with the default
configuration), its keys are:
- a view's keys, with 17 drag types before its frame;
- subviews outside the objects list: a `WKFlippedView` of its size,
  clipping to its bounds, and two hidden `WKColorExtensionView`s;
- a `WKWebViewConfiguration` holding a process pool, preferences, a user
  content controller, the default data store and WebKit's defaults;
- back and forward gestures off, an empty user agent, link previews on,
  magnification off at 1.

**Xibs without direct instantiation** (older documents, without
`customObjectInstantitationMethod="direct"`; about ten probes). A custom
view is an `NSCustomView`: the view's keys, then its class reference
when it has a class, then its runtime class name (`NSView` without one);
its subviews are archived in reverse document order (a plain view's keep
theirs; the objects list keeps document order). A custom object is an
`NSCustomObject` of its class, whatever the class. With direct
instantiation, an `NSCollectionViewItem` object is made with `init` as an
app class's is.

**Smaller rules.**
- `NSvFlags`' 256 is resizing subviews, left out for
  `autoresizesSubviews="NO"`.
- A text field cell's placeholder (`NSPlaceholderString`) follows its font,
  and its title may be a `<string key="title">` element.
- A menu item's `NSIndent` follows its disabled flag.
- A view's accessibility identifier is an `NSNibAXAttributeConnector` (the
  view, a new `NSMutableString` `AXIdentifier`, the identifier) in
  `NSAccessibilityConnectors`, by the view's id as bytes, listed again in
  `NSAccessibilityOidsKeys` with ids numbered after the other objects'.
- Subview groups as far from the corner go leftmost first, and at the
  same corner by their ids as bytes (nine probes).
- A view controller shown as a single window has presentation style 1 and
  `NSStoryboardSegueDestinationOptions`, as a window controller does.
- `simulatedAppContext` changes Xcode's layout of the text in it, so only
  `relax_ibtool_layout` compiles it.
- A window with a hidden title has `NSWindowTitleVisibility` 1, and with a
  transparent title bar `NSTitlebarAppearsTransparent`, after its
  full-screen sizes.
- A window controller of a custom class is an `NSClassSwapper` for
  `NSWindowController`.
- An older xib's Application placeholder may have outlets (its delegate).

**Collection views** (about ten probes), as a scroll view's document:
an `NSCollectionView`'s view keys, then its `_NSCollectionViewCore` (the
flow layout; `UIAllowsSelection` false when not selectable;
`UIAllowsMultipleSelection` when it allows several), empty minimum and
maximum grid sizes (one string), no grid row or column limits, its
background colours, `NSSelectable` and `NSAllowsMultipleSelection` when
set, and dragging masks -1 and 0. The flow layout, listed after its
collection view, archives its interitem and line spacings as floats when
not 10, then its item size when not 50 by 50. A scroll view leaves out its
scroll amounts when all four are 10. Laying a constrained scroll view out
again, Xcode may turn its collection view's horizontal scroller off, so a
constrained one without `hasHorizontalScroller="NO"` is refused.

**Top-level objects.** A xib may hold controls at its top level
(boxes, text fields, buttons, pop-up buttons, image views, scroll views),
without a superview or next responder, and collection view items: an
`NSClassSwapper` for `NSCollectionViewItem`, then
`showSeguePresentationStyle` 0. A control of a custom class is an
`NSClassSwapper` naming its AppKit class before its keys; of its own AppKit
class, itself.

**Images.** A control's named image is an `NSCustomResource`. AppKit's
named images, NSImage.h's 138 `NSImageName` constants as their values
(the retired `NSDotMac` aside), are in the `system` namespace, with the
design sizes of the Mac's images (macOS 27), whatever the document's
resources say; this ibtool holds those sizes in a table from one probe of
all 138. Other images take the document's sizes and no namespace.

**Colours.** Calibrated white and RGB (`NSColorSpace` 3 and 1) and device
white and RGB (4 and 2) archive their components as `%.10g` text (`NSWhite`
or `NSRGB`, ending in a NUL), alpha only when not 1. Custom colour spaces
(sRGB, gamma 2.2 grey, Display P3) archive the components converted to
calibrated ones, the colour space with its profile, the original
components (alpha always), and `NSLinearExposure` (`0` or `1`). For sRGB
(about 60 probes) the conversion is the one `colorsync` models, and the
exposure is `1` exactly when all four components are multiples of 1/255,
as Interface Builder's colour picker makes them. For gamma 2.2 grey the
exposure follows no rule found yet (0.1 and 0.25 give 1, 0.123 gives 0),
so grey and Display P3 colours are refused.

**Container views** (seven probes). A container view is a plain view.
Its embed segue makes the parent controller hold an
`NSStoryboardSegueTemplates` array, after its nib name, of
`NSStoryboardEmbedSegueTemplate`s (the segue's identifier if any, the
trigger `IBSegueTriggerViewDidLoad`, the destination
`NSViewController-<id>`, and a nil container view). The templates are in
document order in the array, but listed in reverse after the controller
in the objects list, each with outlets to the controller (`controller`)
and to its storyboard placeholder (`storyboard`). The controller's
external objects table maps `UpstreamPlaceholder-<n>` to itself (when it
needs one) and then to each template. Its view's nib has a placeholder of
each, with a template's `containerView` outlet to its container view. The
embedded controller has its own nib, `NSViewController-<id>`, in the
Info.plist's nib names and UUIDs, holding it as a window's content
controller is held (a plain `NSViewController` without a class). Xcode's
UUIDs change from run to run, so these are checked by probes, not by
`tests/ibtool_xib`. A view
controller with a storyboard identifier has its own nib of that name,
holding it and a storyboard placeholder, and requiring its class swapper
for connections whether or not it has a class.

Refused: other elements and attributes (primary boxes, collection
views and other controls, scroll views Xcode tiles, bindings, other segues
and controllers, other colours).
