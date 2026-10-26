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
  smallest signed width, negative ones 8 bytes (a presentation style of
  -2); rects and points are data: 7 then doubles;
  booleans are type 4 (false) and 5 (true);
- one object per distinct string or number, encoded where first used (the
  hugging priority `{251, 251}`, a font name, `labelColor` are shared);
- classes are listed in first use, then the fallback classes not already
  listed (UIColor's NSColor, UIFont's NSFont), which the class entry
  references, in byte order: NSColor, NSFont, then UIView for a table
  cell's content view (a set's order gives NSFont first with all three).

The key table's order is a hash table's. Comparing Apple's key tables with
containers filled through public APIs in candidate orders, `NSMutableSet`
filled with keys in depth-first first-use order reproduced all ten nibs
examined (five storyboards); CoreFoundation's CFSet and CFDictionary did
not. foundation-hashing.md describes that set's layout.

## Supported elements

Scenes whose view controller (with or without a custom class) has a root
view with a frame and autoresizing mask, a background colour that is one of
40 iOS system colours or an sRGB colour, and either a safe-area layout guide or legacy top and bottom layout
guides (in a document using safe areas, a root view that lists neither
has the safe-area guide all the same: four probes; so does one with
legacy guides, beside them, whether it lists the safe area or not, the
safe-area guide following the root view in the objects as without them:
three probes, constraints on that guide beside legacy ones refused), alone or in a navigation, tab bar or split view controller (and table and collection view
controllers). Subviews: labels (the system font in any weight, bold or
italic, a font Xcode finds for iOS, or the iOS text styles and, on tvOS,
Title1; the default or an sRGB, grey or system text colour), text fields
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
  from x + y, the centre and y then x). A view nested in plain views, with
  the root view as its constraint's other item either way round, goes by
  its origin in the root view's coordinates (four probes; one whose root
  subview is resized where it is laid out is refused). In a group, constraints whose
  second item is deeper than the root's subviews come first, by first and
  then second attribute in the siblings' order below (eleven probes; a
  deeper view as the first item, which would group them by it, is refused),
  then the constraints with the root view, by kind and then by first and
  second attribute: those with a horizontal centre (either attribute,
  `centerXWithinMargins` before `centerX`), then a vertical centre, then
  between left and right edges (right margin, left margin, right, left),
  then horizontal edges with a leading or trailing one (trailing margin,
  leading margin, trailing, leading, each tied with its right or left
  counterpart; between a leading and a trailing edge, the one whose root
  view's edge is the trailing one first), then the rest (width, top margin,
  bottom margin, bottom, top, first baseline, height). Measured with all
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
  (the same first item and pair of attributes, their second items sibling
  views) follow no single rule. Some cases are settled for constraints
  with no constant, multiplier, relation or priority:
  - Between centres (centerX or centerY both ways), the second items'
    frames decide, whatever the ids, document and subview orders: the
    lower one first, then the one further right (19 probes, and seven of
    the corpus's storyboards).
  - Between the same attribute, second items with the same frame come in
    id order (three of the corpus's).

  Others are refused. With a constant, the ids play a part too (22
  probes). Top to bottom goes by the bottom edge from the top, then by id,
  and leading to trailing by the trailing edge from the right, then by id
  (ten probes, too few to settle either). Two constraints with the same
  items and attributes are refused as well,
  as are constraints from a legacy guide (but the bottom guide as the first
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
  attribute, before `NSMultiplier`; a `symbolic` constant is
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
- A storyboard in a `Base.lproj` directory has its translatable strings
  archived as `NSLocalizableString`s: the string inline (`NS.bytes`),
  `NSKey` `<id>.<property>` and `NSDev` the string (shared as strings
  are), for a label's `text`, a button's `normalTitle`, a text field's
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
  that order (twelve probes).
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
  place as a label's (five probes).
- A plain `<view>` subview archives its subviews, background colour, the
  flags, its autoresizing mask (36 without one), the autoresizing key and
  its constraints. With a custom class it is a `UIClassSwapper` with the
  same values, then `UIClassName` (the runtime name, as a controller's)
  and `UIOriginalClassName` `UIView`. A text field with a custom class is
  swapped in the same way, `UIOriginalClassName` `UITextField` (three
  probes: a module, none, and the target's). A button's
  `UIViewInsetsLayoutMarginsFromSafeArea` 0, when turned off, comes after
  its compression resistance, before its semantic content attribute (two
  probes).
- With a `<device>`, the root view is laid out at the device's size (or
  its freeform size) at its frame's origin (a view below an opaque bar is
  at y 64; eight probes, with simulated bars changing nothing),
  whatever its frame's size says: 26 device ids measured in both orientations
  (landscape swaps width and height; `retina3_5` and `retina6_72` are
  393x852 either way; `appleTV` is 1920x1080). Ids Xcode doesn't know
  (`retina5_4`, `ipad10_5`, `ipad12_9`, `ipad8_3rounded`,
  `ipad13_0rounded`, `appleTV4K`) take the frame; other ids are refused.
- Subviews keep their frames whatever size the root view is laid out at,
  except autoresizing ones (not `translatesAutoresizingMaskIntoConstraints="NO"`),
  which the root resizes by their masks (sizable width and height grow
  with it: 100x80 under a 375x667 frame laid out at 600x600 is 325x13).
  Where the root's size isn't its frame's, a subview whose mask has a
  flexible left or top margin or a sizable width or height is refused; the
  rest are compiled, with or without a device (eight probes).
- An image view (`UIUserInteractionDisabled` when `userInteractionEnabled="NO"`; scale to fill, aspect fit,
  aspect fill or centre) archives as a label's geometry and flags (a
  background colour first, hidden and tag after `UIOpaque`, alpha after
  the mask), then `UIContentMode` unless scale to fill, `UIClipsToBounds`,
  `UITintColor`, the autoresizing key, its own constraints, its hugging
  (none without priorities; a missing one is 250), its compression
  resistance if given (each 750 unless given), and `UIImage`:
  a `UIImageNibPlaceholder` per image name (width and height 1, the name,
  nil symbol, configuration and catalog), shared by the views using it;
  a system symbol (`catalog="system"`) is a placeholder of width and
  height 0 naming it as both resource and symbol, shared by name apart
  from an image of the document's of that name (five probes, image views
  and a button's normal state); no image, no key; then `UIAdjustsImageSizeForAccessibilityContentSizeCategory`
  when set (one probe). snag's launch screen is byte-identical.
- A controller's own `restorationIdentifier` is `UIRestorationIdentifier`
  after `UINibName` (one probe; beside a storyboard identifier, or in an
  embedded controller, it is refused).
- A root view's content mode other than scale to fill is `UIContentMode`
  after its mask (two probes), then `UIClipsToBounds` when it clips (one);
  beside `clearsContextBeforeDrawing` or a tint they are refused. One that
  doesn't autoresize its subviews leaves `UIAutoresizeSubviews` out (one
  probe). A table view controller's `autoresizesArchivedViewToFullSize="NO"`
  is `UIAutoresizesArchivedViewToFullSize` false after its identifiers,
  before its segues (two probes). A navigation controller's
  `extendedLayoutIncludesOpaqueBars` is `UIKeyExtendedLayoutIncludesOpaqueBars`
  1 before `UIViewControllers` (one probe). A text view's custom class
  swaps in for it as a text field's does, and a segmented control's mask is
  its document's (three probes).
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
  `labelColor` (two probes).
- A view's `UIBounds` is left out when its size is zero, and its
  `UICenter` when the centre is the origin. A label, button or plain view
  without a frame, or with an all-zero one, is 1000x1000 at the origin,
  or has no size when the document says `fixedFrame="YES"` (18 probes,
  three corpus storyboards); so is an image view or a segmented control
  (four probes, one corpus storyboard), an image view's `fixedFrame`
  otherwise changing nothing. An image view's `UIAutoresizingMask` is its
  document's mask, whether or not it translates it (three probes). An empty `<toolbarItems/>` on a view
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
  float, after the alignment (one probe; beside a scale factor or letter
  spacing it is refused).
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
- A text view (`UITextView`) is a scroll view: after its view values,
  `UIScrollViewIndicatorInsetAdjustmentBehavior` 1, its `UIIndicatorStyle`
  (black 1, white 2: three probes), then `UIBounceEnabled`,
  `UIShowsHorizontalScrollIndicator`, `UIShowsVerticalScrollIndicator`
  (each false when turned off), `UIScrollDisabled`, `UIBouncesZoom` (left
  out when off) and `UIScrollViewContentInsetAdjustmentBehavior` 0; one
  that doesn't scroll doesn't clip. Its text (a `<string key="text">` or
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
  refer to them. Then `UIAdjustsFontForContentSizeCategory`, `UIFont`,
  `UITextColor` (none for a nil colour), `UITextAlignment`, `UIEditable`
  and `UISelectable` (false when turned off), the text input traits,
  `UITextHighlightAttributes` (`NSColor` system blue,
  `NSTextHighlightOutlineColor` secondary system fill) and
  `UITextAllowsNumberPadPopover`. A text view's autocapitalization
  defaults to sentences: that is left out, and none (or no attribute) is
  written as 0. An opaque grey white background is
  `systemBackgroundColor`, as a plain table's is its own. Secure text
  views (which add more keys), `alwaysBounceVertical` and text in
  localizable storyboards are refused (32 probes).
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
  empty identifier. When a constraint uses the guide, the root view also
  owns a second one, archived where a constraint first refers to it (in
  the order above): identifier `UIViewSafeAreaLayoutGuide`, locked,
  archived and allowing negative dimensions, with four system constraints
  pinning its top, left, bottom and right to the view's
  (`UIViewSafeAreaLayoutGuide-top`, ... as `NSLayoutIdentifier`, not
  archived), listed in the view's `UIViewLayoutGuides` after its
  constraints. Its connections: the controller's outlets with `view`
  among them (a text field's `delegate` too, from the field to the
  controller's placeholder), then control actions, each in byte order of
  their names (`aButton`, `view`, `zButton`; `B:`, `a:`; `tap1:`, `tap:`,
  `tap_:`), whatever the document order, ids or positions; actions with
  the same selector stay in document order (19 probes). Actions on other
  events sort by event mask first (valueChanged 4096,
  primaryActionTriggered 8192, editingDidBegin 65536, editingChanged
  131072, editingDidEnd 262144, editingDidEndOnExit 524288), then by name
  (two probes).
- A view nib that refers to its controller (an outlet into it, an action
  to it) has an `UpstreamPlaceholder-1` proxy, which the controller's nib
  maps to its class swapper in `UIExternalObjectsTableForViewLoading`.
- tvOS views have no `UIViewLargeContentStoredProperties`.

## Stack views, content modes, named colours and gesture recognizers

- The document's tint colour (a `<color key="tintColor">` after its
  resources) is `UITintColor` on each controller's root view without one
  of its own (after its mask), on a root table or collection view (after
  its clipping) and on a navigation bar (after its mask); a root view's
  own tint goes in the same place, and its colours other than background
  and tint are refused. A segmented control beside it takes nothing of it
  (one probe); beside tab bars, toolbars, split and page view controllers
  and search bars the document's tint is refused, not yet probed (six
  probes).

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
  after them and after `UIViewLayoutMarginsFollowReadableWidth`. A
  background colour comes after its subviews, its own constraints after
  the not-translating key (26 probes).
- A view's or label's `contentMode` is `UIContentMode` after
  `UIAutoresizingMask`, UIViewContentMode's values, scaleToFill (0) left
  out; Xcode compiles `topLeft` as scaleToFill, so it is refused (18
  probes). A label is `UIUserInteractionDisabled` only when it says so.
- A named colour (`<color name=...>`, defined by a `<namedColor>` in the
  document's resources) is `UIDynamicCatalogName`,
  `UIDynamicCatalogUseNibBundle` and its definition's components, unrounded
  (sRGB and grey; three probes and one corpus storyboard). Colours in
  Display P3 and calibrated RGB are converted to extended sRGB in floats,
  through a matrix this ibtool doesn't reproduce exactly (it fits 24
  colours to 2e-7): refused. For calibrated RGB, 148 colours fit a
  linearisation of gamma 1.8 with a linear toe of slope 1/16 below 1/32,
  a matrix whose rows sum to 1, and the sRGB encoding (sign kept for
  values outside 0 to 1), to within a few float ulps (5e-7) but exactly
  for only a quarter of the components, which points at single-precision
  arithmetic or tables not established here.
- A scene's gesture recognizer (tap, pan or swipe) is in
  the controller's nib, after the storyboard placeholder, with
  `UIGestureRecognizer.allowedTouchTypes` 15 and
  `requiresExclusiveTouchType`, then a tap's `_imp` `UITapRecognizer`
  (taps or touches when not 1), a pan's minimum and maximum touches and
  `allowedScrollTypesMask` 0, a swipe's direction (left 2, up 4, down 8,
  right left out). The controller's external objects map
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
  controller's nib lists the recognizers in reverse document order,
  `UITopLevelObjectsToKeepAliveFromStoryboard` in document order, and
  their actions by selector, one selector's in document order (ten
  probes). A recognizer of two views is refused.
- A navigation controller's root that nothing loads on its own keeps its
  recognizers in the navigation controller's nib: defined in the root's
  `UIExternalObjectsTableForViewLoading` in their placeholders' order,
  kept alive (`UITopLevelObjectsToKeepAliveFromStoryboard`, after
  `UIParentViewController`), top-level objects after the navigation
  controller (in reverse document order) and listed right after it, their
  actions after `sceneViewController` (two probes). Beside segues or bar
  button items, and in other containers, they are refused.

## Navigation controllers

A `<navigationController>` (its bar, a root view controller by a
relationship segue, empty toolbar items) compiles to
`UINavigationController-<id>.nib` (or its identifier's), which embeds the
root controller; the root's scene adds its view nib, and its own nib only
when it is initial or identified. Observed with 25 probes:

- Top-level objects: the root's scene's storyboard placeholder, the owner,
  the first responder, the navigation controller's storyboard placeholder
  and a class swapper for the controller (`UINavigationController`, or the
  custom class's runtime name): `UITitle`, `UIStoryboardIdentifier`,
  `UIRestorationIdentifier`, `UIChildViewControllers` (the root),
  `UIModalPresentationStyle` (UIKit's order: full screen 0, page sheet 1,
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
- The bar's appearance: a background colour before `UIDeepDrawRect`;
  `translucent="NO"` adds `UIOpaque` after it and makes
  `UIBarTranslucence` 2; its tint (or the document's) after the mask;
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
its tint (`UITintColor`; two probes). The items follow the controller's segue templates in its nib's
objects (its own nib's, or its navigation controller's).
- **Actions.** An item's action on its controller is an event connection
  without a mask.
- **Segues.** An item's segue is a template in the controller's nib, after
  its own and its view's in the templates array and objects, with a
  `perform:` event connection from the item.
- **Order.** These connections come after the templates' outlets, by
  selector: actions with one selector the right item's first, and the
  `perform:`s in the order of their templates' outlets.

A child navigation controller's root may have segues (Xcode's
master-detail template): its templates follow the children's parts in the
parent's objects, their `viewController` outlets to the root follow the
scene outlet. A child navigation controller a segue shows (the detail) has
a nib of its own too, as any controller a segue loads; one loaded
otherwise as well is refused. Its bar's `multipleTouchEnabled` is
`UIMultipleTouchEnabled` before the autoresizing keys and `clipsSubviews`
`UIClipsToBounds` after them (two corpus storyboards, Swift and
Objective-C).

Bar button items in a tab or split view controller's children, beside a
tab bar item or a gesture recognizer, and with images, are refused (46
probes).

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
(seven probes). Fractional spacing, an unidentified or initial one,
children and segues to one are refused.

## Tab bar controllers

A `<tabBarController>` (its tab bar, children by `viewControllers`
relationship segues, empty toolbar items) compiles to
`UITabBarController-<id>.nib` (or its identifier's), which embeds every
child: view and table view controllers as a navigation controller embeds
its root, and navigation controllers with their bar and root. The
children's scenes add their view nibs, and their own nibs only when they
are identified or a segue reaches them. Observed with 17 probes and 19
storyboards from the corpus that use one (4 of them compile today; the
rest are refused for other elements):

- Top-level objects: a storyboard placeholder per controller the nib holds,
  in reverse: the last child's first, a navigation child's root before the
  child itself. Then the owner, the first responder, the scene's own
  storyboard placeholder and the controller (`UITabBarController`, or a
  class swapper): `UITitle`, its identifiers, `UIChildViewControllers` (the
  children, when it has any), `UIKeyAutomaticallyAdjustsScrollViewInsets`,
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
- The bar is laid out by Xcode: bounds 0x54 centred 27 below its frame's
  top (at x 0; a mask of flexible width and top margin changes nothing:
  four probes and a corpus storyboard), its background colour if any, opaque, mask 10, hugging `{250, 250}`,
  `UIViewInsetsLayoutMarginsFromSafeArea` 0 when turned off, `UIDelegate`
  the controller, `UIItems` (an `NSArray` of the children's items),
  `UIBarTranslucence` 1, or 2 when `translucent="NO"`.
- One empty external objects table and one empty attributes dictionary (of
  navigation item titles) are shared by every object that has one.
- Simulated bottom bar metrics change nothing. Children with segues of
  their own, identified navigation children or ones loaded on their own
  too, tab bar items with insets, and tab bar controllers nothing loads are
  refused.

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
`view` (the upstream placeholder is in the nib for them). Observed with 94
probes:

- The table's values: geometry (the device's size, as a root view's),
  `UIBackgroundColor`, `UIDeepDrawRect`, `UIOpaque` (unless it says NO),
  `UIAutoresizeSubviews`, `UIAutoresizingMask`,
  `UIClearsContextBeforeDrawing` false when it says NO, `UIClipsToBounds`,
  the semantic content attribute, `UIViewLargeContentStoredProperties`,
  `UIScrollViewIndicatorInsetAdjustmentBehavior` 1,
  `UIShowsHorizontalScrollIndicator` false when it says NO (the vertical
  one changes nothing), `UIBouncesZoom`, `UIAlwaysBounceVertical` when
  YES, `UIKeyboardDismissMode` (onDrag 1, interactive 2),
  `UIScrollViewContentInsetAdjustmentBehavior` 0, `UITableViewStyle`
  (plain 0, grouped 1, insetGrouped 2), `UIStyle` (0 plain, else 1), the
  separator style, `UIPrefetchingEnabled`, the row, section header and
  footer heights and their three estimates as floats, `UIFillerRowHeight`
  0.0 and `UISectionHeaderTopPadding` -1.0 as doubles,
  `UIAllowsSelection` false, `UIAllowsMultipleSelection` or
  `UIAllowsSelectionDuringEditing` true when set,
  `UIInsetsContentViewsToSafeArea`, the header and footer views,
  `UISeparatorColor` (four probes with a header, a footer or both), the
  prototypes,
  `UISectionIndexBackgroundColor`, `UISeparatorInsetReference` 0.
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
  `tableBackgroundColor`; other colours, and every colour of a grouped
  table, are archived as usual.
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
  are none). Its objects are the cell, its content view and the content
  view's subviews; then, breadth first, each view's constraints and its
  subviews, the content view's not listed again.
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
  `UITableCellBackgroundColorSet` with a background colour,
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
  its right edge, whatever the table's separator style or the device.

Refused for now: static tables, cells with a style (Basic, Subtitle and
the rest), actions in cells, tables as subviews, header and footer views,
refresh controls, and other table and cell settings.

A table view can also be a subview of a controller's view: the same values
as a table view controller's (one probe), with a subview's mask (36) when
it gives none and its `UIViewDoesNotTranslateAutoresizingMaskIntoConstraints`
and own constraints after `UIClipsToBounds`; its outlets go to the
controller's placeholder. Its cells' segues are refused.

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
Without `colorMatched`, a grey white background stays as it is rather
than becoming the table's or text view's system background.

A table's header and footer views (`<view key="tableHeaderView">`,
`key="tableFooterView"`) are its subviews, the footer view first, and the
table's `UITableHeaderView` and `UITableFooterView`, after
`UIInsetsContentViewsToSafeArea`. Xcode lays them out itself: the header
view at the top, as wide as the table is laid out, whatever its frame
says; the footer view keeps its x, width and height and is right under the
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
sorted by name with the controller's; one between views uses no
placeholder. A plain view may be `opaque="NO"` (no `UIOpaque`) or clip its
subviews (`UIClipsToBounds`, after its content mode), and its
`fixedFrame` changes nothing (seven probes).

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
its first entry: the views in a mutable array in document order,
`runtimeCollectionClassName` the class and `addsContentToExistingCollection`
false. Those of table and collection view controllers, and beside a gesture
recognizer, are refused (six probes).

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

A scroll view without content and frame layout guides (those are refused)
is compiled as a plain view, `UIScrollView` (or swapped for its custom
class), then after its large content properties
`UIScrollViewIndicatorInsetAdjustmentBehavior` 1, `UIShowsHorizontalScrollIndicator`
and `UIShowsVerticalScrollIndicator` false when hidden, `UIBouncesZoom`,
`UIAlwaysBounceVertical`, `UIDirectionalLockEnabled`, the minimum and maximum
zoom scales (floats) and `UIScrollViewContentInsetAdjustmentBehavior` 0
(eight probes).

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
value; a stack view's standard spacing (`spacingType`) is archived as 0,
whatever its spacing (eight probes).

A button's states are its `UIButtonStatefulContent`, keyed by
UIControlState in that order whatever the document's (normal 0, which it
needs, highlighted 1, disabled 2, selected 4), each a `UIButtonContent` of
its title, image, background image (`UIBackgroundImage`) and colours (ten
probes; a localizable build's titles for other states than normal are
refused). A button may be hidden, tagged (both after `UIOpaque`) and
have an alpha (after its mask), as a plain view (two probes), and given
hugging (each 250 unless given) or compression resistance (each 750
unless given) priorities, written when given, before its semantic content
attribute (three probes). A selected button has `UISelected` before
`UIDisabled`, and a line break mode other than middle truncation is
`UILineBreakMode` (word wrap 0, tail truncation 4) after its stateful
content (four probes). A button may have no states at all, and then no
`UIButtonStatefulContent`, or only others than normal (two probes); a
content mode other than scale to fill is `UIContentMode` after its mask
(one probe).

A label's, plain view's, button's, text view's or control's (a switch or
slider: three probes) `<accessibility>` is a
`UIRuntimeAccessibilityConfiguration` in the nib's
`UINibAccessibilityConfigurationsKey`, several views' (the root view's
too, first: one probe) in the objects'
order (siblings either way round, a container and its subview with its
configuration before or after its subviews, a nested label after a
sibling: six probes):
`UIAccessibileObject` (sic), then those given of `UIAccessibilityHint`,
`UIAccessibilityIdentifier`, `UIAccessibilityLabel`, `UIAccessibilityTraits`
(an `NSNumber` of UIAccessibilityTraits' bits: button 1, link 2, image 4,
static text 64, not enabled 256, header 65536, none 0) and
`UIIsAccessibilityElement` (0 or 1); equal numbers are one object (14
probes). In `Base.lproj` the hint and label are `NSLocalizableString`s
keyed `<id>.accessibilityHint` and `<id>.accessibilityLabel`, as titles
are; the identifier stays a string (one probe). Runtime attributes, switches (their frame pair) and accessibility
settings inside cells and table headers are refused: those nibs aren't
built with them.

A button with a plain `<buttonConfiguration>` and a title has no
`UIButtonStatefulContent` or `UIFont`; its `UIButtonConfiguration` is
UIKit's default configuration (style 0, a `UIBackgroundConfiguration` with
a clear background and stroke, corner radius 17 whatever the button's
size, stroke width 1; image placement 2, image padding 0, title padding
1), its title an `NSAttributedString` with the empty attributes,
localizable as `<id>.configuration.title`, and an empty subtitle (four
probes and one in `Base.lproj`; other styles and settings are refused).

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
(mask 4096) after the outlets; in a static table, beside header and footer
views, a keyboard dismiss mode or with an attributed title it is refused
(two probes).

A collection view in a view is compiled as a collection view controller's
(mask 36 when none is given, its constraint keys after its clipping), its
outlets the controller's, its flow layout listed as its child (after the
collection view's siblings) and its automatic estimated item size a pair
of the layout's; its cells' segues and constraints it holds are refused
(three probes).

A search bar, in a view or as a table's header view (laid out as a header
view is), is a `UISearchBar`: `UIOpaque` when not translucent, its mask
(36 by default), content mode 3 (redraw), hugging `{250, 250}`, then after
its large content properties `UIBarStyle` 1 for black,
`UISearchBarTranslucence` (1, or 2 when not translucent), `UIText` (empty
when it has none), `UIPlaceholder`, `UIEnablesReturnKeyAutomatically` and
`UISecureTextEntry` false and `UISearchBarStyle` 2 for minimal; its delegate
outlet is an outlet like any other (six probes).

A segmented control (plain style, left and top content alignment) is a
`UISegmentedControl`: its segments are its `UISubviews`, in order but for
the selected one, which comes last; each a `UISegment` with bounds 0x32 at
(0, 16) whatever the control's size, `UIDeepDrawRect`,
`UIAutoresizeSubviews`, the semantic and large content keys, its title as
`UISegmentInfo` (localizable as `<id>.segmentTitles[i]`),
`UISegmentStyle` 5 and `UISegmentPosition` by index: none for the first,
3 for the second, 1 for the rest. Then the control's mask 36, constraint
keys, hugging `{250, 250}`, the alignments (vertical 1 and 1, horizontal
1), `UIPointerInteractionEnabled`, `UISegments` (the segments in order)
and `UISelectedSegmentIndex`; value-changed actions as other controls'
(24 probes, every selection of two to five segments, three heights, and
one in `Base.lproj`). The segments aren't in the objects list, and
`fixedFrame` beside a frame changes nothing (one probe).

A date picker's date isn't archived: a `UIDatePicker` has the view keys,
then `UIDatePickerMode` unless date and time (time 0, date 1, count-down
timer 3), `UIPickerStyle` 3 for inline, `UIMinuteInterval` (0 unless
given) and, for a count-down timer, `UIDatePickerUseCurrentDateDuringDecoding`
false (eight probes; Xcode lays the wheels style out at its own width, so
it is refused).

A map view is an `MKMapView`: opaque, multiple touch, mask 36, clipping,
its constraint keys, zero `UIViewLayoutMargins`, then MapKit's keys
(`MKDelegate` nil, `MKMapType` 0, zoom, scroll, pitch and rotation on,
user location and traffic off, buildings on, `MKShowsCompass` unless
`showsCompass="NO"`, `MKShowsScaleKey` with `showsScale`, points of
interest on, an `MKStandardMapConfiguration`, `MKPitchButtonVisibility` 1,
no user tracking button); its delegate outlet is an outlet like others
(six probes; other map types and settings are refused).

A picker view is a `UIPickerView`: the view keys (opaque, hidden, mask 36),
hugging `{250, 250}` and after its large content properties
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
(seven probes; material styles are refused). Constraints it holds
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

Switches, sliders, steppers and activity indicators (29 probes):
- **Common.** Each is a non-opaque view: `UIDeepDrawRect`, hidden and tag,
  `UIAutoresizeSubviews`, mask 36, alpha, the constraint keys, then its
  hugging (each priority 250 unless given; a switch's, stepper's or
  activity indicator's left out when it is 750 both ways, a slider's
  always written), its compression resistance if either is given (each
  750 unless given), `UIViewSemanticContentAttribute` and the large
  content properties, before its own values (nine probes). Their
  actions are event connections with their UIControlEvents mask, as a
  button's (value changed 4096; a slider's touch up inside 64: one probe).
- **A switch.** `UISwitchEnabled` (or `UIDisabled`), `UISwitchOn` when on,
  `UISwitchOnTintColor`. Its frame is also a key-value pair, `frame`
  (`NS.special` 3, `{{x, y}, {w, h}}`), however it is laid out.
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
  whiteLarge 0 and 0, white 1 and 1, gray 2 and 2), `UIAnimating`,
  `UIHidesWhenStopped` false unless set, and `UIColor`. A colour is also
  `UITintColor`, with `UIDisableUpdateColorOnTraitCollectionChange`;
  without one, a modern style's tint is the secondary label colour. One
  hidden when stopped and not animating is `UIHidden`, once if it is
  hidden as well (two probes).

User-defined runtime attributes of a view, the root view or a controller
are its nib's `UINibKeyValuePair`s (`UIObject`, `UIValue`, `UIKeyPath`), in
document order; a controller's are in its own nib. Those of more than one
view in a nib, or beside a switch's frame pair, are refused: probes put
them in the objects' order (siblings, a container and its subview with
its list before or after its subviews, the root view and a subview,
custom classes, a stack view or another view between, outlets to either:
fifteen probes), but one corpus storyboard's (TreasureHunt's claim
button) follow the document's order of their lists instead, for reasons
not established. A number is an `NSNumber` (`NS.intval` or `NS.dblval`), a
boolean one of 0 or 1, a string the string, a colour the colour, an image the
`UIImageNibPlaceholder` an image view's image would be, declared or not (two
probes), and a size an
`NSValue` (`NS.special` 2, `NS.sizeval` `{w, h}`). Those of an embedded
controller and of other kinds of view are refused (13 probes).

A label that translates its autoresizing mask has the document's mask,
left out when empty. A system font with `size="button"` (older
documents) is 18 points (one probe). A system font of size 0, regular or bold, is 12
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
  view: nine probes), opaque, multiple
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
  one, and its `customSize` changes nothing (three probes). A control's
  action on the cell itself is an event connection after the cell's
  outlets, the cell its destination (one corpus storyboard).
- An old-format content view (`<view key="contentView">`, no id) is not
  the document's: an empty one compiles to a plain opaque view the
  objects leave out. One with subviews keeps its constraints on the cell,
  and is refused.
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
  `UIStoryboardPresentationSegueTemplate` or
  `UIStoryboardPushSegueTemplate`. Its values: `UIIdentifier` (the
  segue's identifier, if any), `UIDestinationViewControllerIdentifier`
  (the destination's nib's name), then for show `UIAnimates` false when it
  doesn't animate and `UIActionName`; for presentation
  `UIModalPresentationStyle` (fullScreen 0, pageSheet 1, formSheet 2,
  currentContext 3, overFullScreen 5, overCurrentContext 6, popover 7,
  automatic -2), `UIModalTransitionStyle` (crossDissolve 2) or
  `UIAnimates` false; for push `UIDestinationContainmentContext` and
  `UISplitViewControllerIndex`, both 0.
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

An unwind segue leads to its scene's `<exit>` and loads nothing. Its
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
says `opaque="NO"`. Its embed segue gets a placeholder like a control's,
in document order with the others, and instead of an action the
placeholder has a `containerView` outlet to the view, sorted by label
with the view nib's outlets. The template is a
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
in its nib's objects; anything else inside a reference is refused. With
references, the order Xcode adds the Info.plist tables' entries in isn't
established (RxSwift's example, 15 references, fits none tried): where
their hashes collide so that the order would matter, it is refused.

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
