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
view with a frame and autoresizing mask, a background colour that is a
document system colour (white/alpha, generic gamma 2.2 grey) or an sRGB
colour, and either a safe-area layout guide or legacy top and bottom layout
guides. Subviews: labels (system font or the Headline, Callout and, on
tvOS, Title1 text styles; default colours), rounded-rect text fields,
rounded-rect buttons with a normal-state title and touch-up-inside actions
on the controller, vertical stack views. Constraints between those, with
constants; outlets from the controller into its view. Everything else, and
other values of the attributes these use, is refused. Observed:

- Without a `<device>`, the root view is laid out at 600x600 whatever its
  frame says (`resources/Launch`).
- Legacy layout guides become hidden `_UILayoutGuide` subviews, after the
  others, each with a `_UILayoutSupportConstraint` (width, priority 999),
  `_UIViewControllerTop`/`Bottom` and three constraints pinning it (top:
  leading, top, height; bottom: leading, height, view bottom to guide
  bottom), which the root view's constraints list first.
- The document's constraints follow, by first attribute: centerX, centerY,
  trailing, leading, width, bottom, top, height, whatever their order or
  ids in the document (measured with all eight on one view, in either
  document order and with the items swapped).
- The nib's objects: the placeholders, the root view, its document
  constraints, the guides, the subviews depth first. Its connections: the
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
