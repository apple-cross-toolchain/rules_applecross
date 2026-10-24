# ibtool

Reproduced: `ibtool --compilation-directory DIR ... X.storyboard` and
`ibtool --link DIR ... X.storyboardc`, as rules_apple runs them, for every
storyboard in the examples: CleanRoom's launch screen, `resources/Launch`,
HelloWorldObjC's `Main` (iOS), tvOS's `Main`, the watchOS host's `Main` and
the watch app's WatchKit `Interface`. Compiled and linked, every file is
byte-identical to Xcode 27.0's (toolchain compiler mode, the default).
Linking copies an iOS or tvOS storyboard; a WatchKit one is rewritten
(below).

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
  is taken whole (`_TtC3Bar11Outer.Inner`). With
  `customModuleProvider="target"`, `--module` is the module when given,
  the document's `customModule` otherwise (an empty provider is none); an
  empty or missing module leaves the class name as written (17 probes).

## NIB archives

`NIBArchive`, format 1, coder 10; four tables (objects, keys, values,
classes), varints whose last byte has the high bit set, and an `LNE\0`
trailer. Observed from Xcode's nibs:

- objects are numbered depth first in encoding order; shared values (one
  empty array, a class name string) are one object;
- each object's values are contiguous, in encoding order; integers take the
  smallest signed width; rects and points are data: 7 then doubles;
  booleans are type 4 (false) and 5 (true);
- one object per distinct string or number, encoded where first used (the
  hugging priority `{251, 251}`, a font name, `labelColor` are shared);
- classes are listed in first use, then the fallback classes not already
  listed (UIColor's NSColor, UIFont's NSFont), which the class entry
  references, in a set's order (NSColor before NSFont either way round;
  alphabetical order would fit too).

The key table's order is a hash table's. Comparing Apple's key tables with
containers filled through public APIs in candidate orders, `NSMutableSet`
filled with keys in depth-first first-use order reproduced all ten nibs
examined (five storyboards); CoreFoundation's CFSet and CFDictionary did
not. foundation-hashing.md describes that set's layout.

## Supported elements

Scenes whose view controller (with or without a custom class) has a root
view with a frame and autoresizing mask, a background colour that is one of
40 iOS system colours or an sRGB colour, and either a safe-area layout guide or legacy top and bottom layout
guides. Subviews: labels (system font, a font Xcode finds for iOS, or the
Headline, Callout and, on tvOS, Title1 text styles; the default or an sRGB,
grey or system text colour), rounded-rect text fields,
rounded-rect buttons with a normal-state title and touch-up-inside actions
on the controller, vertical stack views, image views of named images,
plain views containing any of these.
Constraints between those, with constants, and labels' and image views' own
width and height; outlets from the controller into its view. Everything else, and
other values of the attributes these use, is refused. Observed:

- Without a `<device>`, an iOS root view is laid out at 600x600 whatever its
  frame says when the deployment target is below iOS 17.0, and at its frame
  from 17.0 on (`resources/Launch`, at 12.0, 16.99, 17.0 and 26.0); Xcode's
  ibtool fails on a tvOS storyboard without one.
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
  colour list measured on iOS are those this ibtool knows; others, and
  system colours on tvOS, are refused. Documents from older Xcodes name
  them with `xcode11CocoaTouchSystemColor` or `cocoaTouchSystemColor`
  instead of `systemColor`; those archive the same (five probes, and
  covid_tracker's and MixTag's launch screens).
- A grey in the generic gamma 2.2 space is archived as given, in the grey
  colour space (4), and named `blackColor`, `whiteColor` or `clearColor`
  (`UISystemColorName` first) when, at 10 digits, it is white 0, 1 or 0
  with alpha 1, 1 or 0; other greys, including 0.5, 1/3, 2/3 and white
  with alpha 0, have no name (17 probes, as text and background colours).
  Calibrated and device spaces and Display P3 are converted (calibrated
  white 0.5 is archived as 0.572), so they are refused, except calibrated
  white 0 and 1 (most of the corpus's): 0 is archived as 0; 1 as
  `whiteColor` when opaque and otherwise as the float after 1
  (1.0000001192092896, no `-Double`, text `1`), named as the grey would
  be (eight probes).
- A document without `colorMatched="YES"` (older ones) takes colours'
  components as they are instead of converting them: calibrated RGB is
  archived as sRGB with the document's values, calibrated white as sRGB
  grey (the grey as a float, the alpha not), gamma 2.2 grey and sRGB
  converted (sRGB 1, 0.5, 0.25 as 0.989, 0.413, 0.195), none named.
  Without it, calibrated colours, system colours and neutral black and
  white in the other spaces (which convert to themselves) are compiled;
  the rest are refused (18 probes; the corpus's 16 such storyboards).
- Equal colours are one object, compared after rounding to 10 digits
  (0.18039215686 and 0.1803921568627451 share), sRGB and grey alike; a
  named colour is shared only by name (`whiteColor` and a white
  `systemBackgroundColor` stay two). Equal fonts are one object too, system
  fonts and text styles included.
- A label's declared text colour, even `labelColor`, sets
  `UIDisableUpdateTextColorOnTraitCollectionChange`; a nil one is
  `labelColor` without it.
- A view gets `UIViewDoesNotTranslateAutoresizingMaskIntoConstraints` only
  when its element says `translatesAutoresizingMaskIntoConstraints="NO"`
  (labels, a text field, a button and a stack view, with it removed) and a
  constraint is on it: as an item, by its own constraints, or as a stack
  view's arranged subview. A label that says so but has no constraint
  doesn't get it.
- A font named in `<fontDescription name=...>` is resolved from a set of iOS
  fonts that is neither the Mac's nor the simulator's: of the 1,182
  PostScript names macOS 27 lists (CoreText) and the 304 the iOS 27
  simulator's `UIFont` lists, ibtool finds 301, including Rockwell (not in
  the simulator's list) and not NotoSerifMyanmar (in it) or AndaleMono (on
  the Mac). `family` is ignored. A found font is archived under its own
  name (`STHeitiSC-Light` as `PingFangSC-Light`), with the descriptor's
  `NSFontNameAttribute` and `NSFontSizeAttribute`, options 0x80000000,
  `UISystemFont` false, and its traits (italic 1, bold 2: `Avenir-Black` is
  bold); fonts.rs has them all, measured from one storyboard labelling
  each. A name it doesn't find is archived without the scaling keys and
  with options 0; since whether it finds a name can't be told from the name,
  names outside the table are refused. `--auto-activate-custom-fonts`
  doesn't find a font file beside the storyboard (Rubik-Bold).
- Legacy layout guides become hidden `_UILayoutGuide` subviews, after the
  others, each with a `_UILayoutSupportConstraint` (width, priority 999),
  `_UIViewControllerTop`/`Bottom` and three constraints pinning it (top:
  leading, top, height; bottom: leading, height, view bottom to guide
  bottom), which the root view's constraints list first.
- The document's constraints follow, whatever their order or ids in the
  document. They are grouped by item: a constraint with the root view by
  its other item, one between siblings (subviews, the safe-area guide) by
  its first item. Groups go by where their item is: its frame's origin by
  distance from the root view's origin, then by x, then by id (byte
  order), the safe-area guide counting as at the origin on every iPhone
  (seven devices, by bisection), so a subview at (0, 0) whose id sorts
  before the guide's comes first (two image views at different heights,
  labels at one frame told apart by id, and probes that tell the distance
  from x + y, the centre and y then x). In a group, the constraints with the
  root view come first, ordered by their two attributes as a pair (the
  lower rank, then the higher) in centerX, centerY, trailing, leading,
  width, bottom, top, height; then those between siblings, by first and
  then second attribute in centerX, trailing, leading, centerY, bottom,
  top, width, height. Measured with 176 constraints on two labels, the
  root view and the safe-area guide at once, in three document orders,
  eight random subsets and some 30 smaller probes; legacy guides as the
  second item of a sibling constraint fit too. Constraints with equal keys
  (the same items and the same pair of attributes either way round) come
  in an order that changes with the rest of the set, so they are refused,
  as are constraints from a legacy guide or between one and the root view
  (their frames aren't in the document), constraints between the root
  view and a nested view, between axes, and between a width and a
  height (Xcode drops some of those).
- The nib's objects: the placeholders, the root view, the safe-area guide,
  its document constraints, the legacy guides, then breadth first: each
  view in turn from a queue lists the constraints it holds, then its
  subviews, which join the queue (six probes nesting views two deep beside
  image views with their own constraints; depth first and level by level
  each fail one).
- A subview's constraints are its `UIViewAutolayoutConstraints`, after
  `UIViewDoesNotTranslateAutoresizingMaskIntoConstraints`: its own width
  and height constants (no second item), width before height whatever the
  document's order, then, for a container view, those between it and its
  subviews or among them, ordered as a root view's are, by its subviews'
  frames in it.
- A plain `<view>` subview archives its subviews, background colour, the
  flags, its autoresizing mask (36 without one), the autoresizing key and
  its constraints. With a custom class it is a `UIClassSwapper` with the
  same values, then `UIClassName` (the runtime name, as a controller's)
  and `UIOriginalClassName` `UIView`.
- With a `<device>`, the root view is laid out at the device's size,
  whatever its frame says: 26 device ids measured in both orientations
  (landscape swaps width and height; `retina3_5` and `retina6_72` are
  393x852 either way; `appleTV` is 1920x1080). Ids Xcode doesn't know
  (`retina5_4`, `ipad10_5`, `ipad12_9`, `ipad8_3rounded`,
  `ipad13_0rounded`, `appleTV4K`) take the frame; other ids are refused.
- An image view (`userInteractionEnabled="NO"`; scale to fill, aspect fit,
  aspect fill or centre) archives as a label's geometry and flags, then
  `UIContentMode` unless scale to fill, `UIClipsToBounds`, `UITintColor`,
  the autoresizing key, its own constraints, its hugging, and `UIImage`:
  a `UIImageNibPlaceholder` per image name (width and height 1, the name,
  nil symbol, configuration and catalog), shared by the views using it;
  no image, no key. snag's launch screen is byte-identical.
- A controller's `title` is its swapper's `UITitle`, first;
  `useStoryboardIdentifierAsRestorationIdentifier` adds
  `UIRestorationIdentifier` (the identifier's string) after
  `UIStoryboardIdentifier`; `userLabel` changes nothing. A label's
  `clipsSubviews` is `UIClipsToBounds` after `UIContentMode`; a text
  field's `placeholder` is `UIPlaceholder` after `UIFont`.
- The safe-area guide in the objects is an unowned `UILayoutGuide` with an
  empty identifier. When a constraint uses the guide, the root view also
  owns a second one, archived where a constraint first refers to it (in
  the order above): identifier `UIViewSafeAreaLayoutGuide`, locked,
  archived and allowing negative dimensions, with four system constraints
  pinning its top, left, bottom and right to the view's
  (`UIViewSafeAreaLayoutGuide-top`, ... as `NSLayoutIdentifier`, not
  archived), listed in the view's `UIViewLayoutGuides` after its
  constraints. Its connections: the
  controller's outlets, `view`, then control actions.
- A view nib that refers to its controller (an outlet into it, an action
  to it) has an `UpstreamPlaceholder-1` proxy, which the controller's nib
  maps to its class swapper in `UIExternalObjectsTableForViewLoading`.
- tvOS views have no `UIViewLargeContentStoredProperties`.

## WatchKit storyboards

Compile to `Interface.storyboardc/Interface.plist`, a binary property list
(`controllers`: per controller `controller-<id>`, its class and items;
`root`), which linking writes to the link directory itself. Each item is a
dictionary of its `type` and properties (a label's `text`, `numberOfLines`,
`fontScale` 1, a text style's WatchKit name as `fontSize` and `fontSize2`,
`color`, an outlet's `property`; a group's `width`, alignments, `layout`,
`items`). A calibrated white of 2/3 is recorded as `B8`; other colours are
not established.

The writer shares an object between equal strings and numbers only where
Interface Builder's writer does (by identity: everything here except the
controller name, written once as a key and once as `root`), and Info.plist
shares none. Dictionary keys enumerate in CoreFoundation's order for the
order they were inserted (foundation-hashing.md): for the compiled plist,
an order verified only for these key sets, so other items are refused;
linking reads the plist back and writes it again, each dictionary created
whole from its keys in the order read, which reproduces the linked file.
