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
guides, alone or in a navigation or tab bar controller. Subviews: labels (system font, a font Xcode finds for iOS, or the
Headline, Callout and, on tvOS, Title1 text styles; the default or an sRGB,
grey or system text colour), rounded-rect text fields,
rounded-rect, system and custom buttons with a normal state (title, title
and shadow colours) and touch-up-inside actions on the controller, vertical stack views, image views of named images,
plain views containing any of these.
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
  as 2/3), three probes.
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
  and `UIOriginalClassName` `UIView`.
- With a `<device>`, the root view is laid out at the device's size,
  whatever its frame says: 26 device ids measured in both orientations
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
  `UICenter` when the centre is the origin. A label or plain view without
  a frame, or with an all-zero one, is 1000x1000 at the origin; a button
  has no size (12 probes, one corpus storyboard).
- A label's `lineBreakMode` is `UILineBreakMode` after `UIMinimumFontSize`
  (or where it would be): wordWrap 0, characterWrap 1, clip 2,
  headTruncation 3, middleTruncation 5; tailTruncation, the default, is
  left out (eight probes).
- A label adjusts its font size to fit unless `adjustsFontSizeToFit="NO"`:
  without that attribute it has `UIAdjustsFontSizeToFit` after
  `UINumberOfLines`, then `UIMinimumFontSize` (a 32-bit float) before
  `UITextAlignment` and `UIMinimumScaleFactor` (also 32-bit) after it.
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
- Nothing in the output comes from a label's `fixedFrame`, from a label's
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
  among them, then control actions, each in byte order of their names
  (`aButton`, `view`, `zButton`; `B:`, `a:`; `tap1:`, `tap:`, `tap_:`),
  whatever the document order, ids or positions; actions with the same
  selector stay in document order (19 probes).
- A view nib that refers to its controller (an outlet into it, an action
  to it) has an `UpstreamPlaceholder-1` proxy, which the controller's nib
  maps to its class swapper in `UIExternalObjectsTableForViewLoading`.
- tvOS views have no `UIViewLargeContentStoredProperties`.

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
  string. A table view controller root is a `UITableViewController`, with
  `UIClearsSelectionOnViewWillAppear` last (four probes).
- The bar is laid out by Xcode, not the document: bounds 0x54 centred 27
  below its frame's top, mask 2, hugging `{250, 250}`, zero layout margins
  (directional), `UIViewInsetsLayoutMarginsFromSafeArea` 0 when the
  document turns it off, `UIIsLocked`, `UIItems` (the root's item),
  `UIDelegate` the controller, `UIBarPosition` 2, `UIBarTranslucence` 1.
  Other bar settings, other segues and controllers nothing loads are
  refused.

## Tab bar controllers

A `<tabBarController>` (its tab bar, children by `viewControllers`
relationship segues, empty toolbar items) compiles to
`UITabBarController-<id>.nib` (or its identifier's), which embeds every
child: view and table view controllers as a navigation controller embeds
its root, and navigation controllers with their bar and root. The
children's scenes add their view nibs, and their own nibs only when they
are identified or a segue reaches them. Observed with 13 probes and 19
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
  `UITitle`, `UIImage`, `UISelectedTemplateImage`. A view controller's own
  nib carries its declared item the same way, after its title, and lists it
  after the navigation item.
- The bar is laid out by Xcode: bounds 0x54 centred at (0, 27), its
  background colour if any, opaque, mask 10, hugging `{250, 250}`,
  `UIViewInsetsLayoutMarginsFromSafeArea` 0 when turned off, `UIDelegate`
  the controller, `UIItems` (an `NSArray` of the children's items),
  `UIBarTranslucence` 1, or 2 when `translucent="NO"`.
- One empty external objects table and one empty attributes dictionary (of
  navigation item titles) are shared by every object that has one.
- Simulated bottom bar metrics change nothing. Children with segues of
  their own, identified navigation children or ones loaded on their own
  too, tab bar items with insets, and tab bar controllers nothing loads are
  refused.

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
  `UIInsetsContentViewsToSafeArea`, `UISeparatorColor`, the prototypes,
  `UISectionIndexBackgroundColor`, `UISeparatorInsetReference` 0.
- Left out, the row height is 44, a plain table's section heights 22, and
  the estimates 0.
- The separator style: default and singleLine are `UISeparatorStyle` and
  `UISeparatorStyleIOS5AndLater` 1; none only
  `UISeparatorStyleIOS9AndLater` 0; singleLineEtched all three, 2.
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
  `UIDeepDrawRect`, `UIOpaque`, `UIAutoresizeSubviews`, `UIClipsToBounds`
  when clipping, `UIViewPreservesSuperviewMargins` and
  `UIViewPreservesSpecificSuperviewMargins` 15 when preserving margins,
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
  as its fallback class): geometry from the document, its subviews,
  `UIDeepDrawRect`, `UIMultipleTouchEnabled`, `UIAutoresizeSubviews`,
  `UIContentMode` 4, `UIClipsToBounds`, its constraints (ordered as a
  container's), the margins as the cell's,
  `UIViewInsetsLayoutMarginsFromSafeArea` 0 when it says NO, the semantic
  content attribute, large content.
- The separator is a point high at the cell's foot, from 8 points in to
  its right edge, whatever the table's separator style or the device.

Refused for now: static tables, cells with a style (Basic, Subtitle and
the rest), actions in cells, tables as subviews, header and footer views,
refresh controls, and other table and cell settings.

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
  refers to it (an outlet or an action), then one per segue in document
  order, and a button's segue is a `perform:` action (touch up inside,
  64) on its placeholder, sorted with the other actions. The controller's
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

Refused for now: unwind, embed, custom and popover segues, segues from
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
