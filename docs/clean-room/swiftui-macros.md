# SwiftUI's macros

Since the iOS 27 SDK, SwiftUICore declares `@State` as a macro,
`#externalMacro(module: "SwiftUIMacros", type: "StateMacro")`. Xcode
implements it in `Platforms/*/Developer/usr/lib/swift/host/plugins/
libSwiftUIMacros.dylib`, which a Swift compiler on Linux cannot load:
without it, every `@State` fails with `plugin for module 'SwiftUIMacros' not
found`.

`tools/apple_tools/swiftui_macros` is a macro plugin executable, built on
swift-syntax, that the toolchain passes to Linux compiles as
`-load-plugin-executable .../SwiftUIMacros#SwiftUIMacros`. It was written
from the macro declarations in SwiftUICore's `.swiftinterface` and from the
expansions Xcode's compiler prints with `-Xfrontend
-dump-macro-expansions`. Apple's plugin was not disassembled.

## What is implemented

| Macro | Type |
| --- | --- |
| `@State`, `@State(initialValue:)`, `@State(wrappedValue:)` | `StateMacro` (peer, accessor) |
| `@_StatePropertyWrapperStorage(initialValue:)` | `StatePropertyWrapperStorageMacro` |
| `@_StateProjectedValue` | `StateProjectedValueMacro` |
| `@_StateInitialStoredValue(_:)` | `StateInitialStoredValueMacro` |
| `@_PropertyWrapperProjectedValue` | `ProjectedValueMacro` |
| `@Entry` | `EntryMacro` (accessor, peer) |
| `@__EntryDefaultValue` | `EntryDefaultValueMacro` |
| `#_SwiftUIEntryTypeCheck` (value and class overloads) | `EntryTypeCheckMacro`, `EntryTypeCheckClassMacro` |

Not implemented, and refused when used: `@Animatable` and its helpers,
`_SwiftUIState`, and `@_StatePropertyWrapperStorage()` without an argument.

The observers and `@Entry` came back from Isomer's port of this plugin
(AttilaTheFun/isomer, `swift_ui_macros`), derived the same way from
Xcode's printed expansions; the layout of their text was matched here.

## The expansions

Observed for properties of every access level, with and without a type and
an initial value, optional (`T?`, `T!`, `Optional<T>`), static, and with the
attribute's arguments:

- A private or fileprivate instance property with an initial value takes
  lazy storage. An initial value counts if it is written or is the implicit
  `nil` of `T?` or `T!` (but not of `Optional<T>`). The value is kept in
  `__name = State._makeStorage(...)`, built from a closure so that the
  initial value is evaluated once. `_name` (via
  `_StatePropertyWrapperStorage`) and `$name` (via `_StateProjectedValue`)
  are computed from that storage. A static property named by
  `makeUniqueName("_initialStoredValue_")` holds the storage to start from:
  `_name`'s argument is that name in base64, and the static property's
  argument is the base64 of the expression that computes it.
- Any other property is stored as `_name = State(initialValue:)`, and
  `$name` comes from `_PropertyWrapperProjectedValue`. Without an initial
  value, `_name` is declared `State<T>` and `$name` is a computed
  `Binding<T>`.
- `$name` takes the property's access modifiers, without `(set)` ones, and
  `static`. `T!` is written `T?`.
- The property gets `get` and `nonmutating set` accessors, plus an init
  accessor (`@storageRestrictions(initializes:)`). A static property has
  neither the init accessor nor `nonmutating`. With lazy storage, only an
  implicit `nil` gets the init accessor.

With `willSet` or `didSet`, the setter runs the observers' bodies as
closures around the write: `({ newValue in ... })(newValue)` before it,
and after it `({ ... })()`, or `({ oldValue in ... })(<old>)` when the
didSet has a parameter or uses `oldValue`, `<old>` being
`makeUniqueName("_oldValue_")`, read into a local first. Apple's text is
laid out exactly: the setter's statements at 4, an observer's first body
line at 4 when its closure is the setter's first statement and 8
otherwise, later body lines at their source column shifted by that less 4,
and each closure's end at 8.

`@Entry` adds `get`, `set` and `_modify` through a peer key,
`private struct __Key_<name>: SwiftUICore.EnvironmentKey` (or
`TransactionKey`, `ContainerValueKey`, or `SwiftUI.FocusedValueKey` with
`typealias Value` for the unwrapped type). The key's `defaultValue` is
marked `@SwiftUICore.__EntryDefaultValue`, which makes it a getter; a
default made by a call adds a `#sourceLocation`-wrapped `_typeCheck` whose
class overload warns, and a closure-typed entry warns too. Its text is
written as is, not reformatted (an empty closure default stays `{}`).

Apple's private `@State` expansion needs Swift 6.4: with 6.3.2 the init
accessor on `_name` makes a view's memberwise initializer private. The
Linux toolchain here is 6.4.0 (`MODULE.bazel`).

For the probe files covering these cases (`probes/`), the expansions Xcode's
compiler prints with this plugin loaded in place of Apple's are identical to
the ones it prints with Apple's; `compare-with-xcode.sh` repeats that check.
`examples/ios/CleanRoom` uses both expansions, and its app compiled on Linux
with this plugin is byte-identical to the one compiled on a Mac with Apple's
(`tools/compare_ipa.sh`).
