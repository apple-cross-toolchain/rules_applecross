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

Not implemented, and refused when used: `@Entry`, `@Animatable` and their
helpers, `_SwiftUIState`, `@_StatePropertyWrapperStorage()` without an
argument, and `@State` on a property with `willSet` or `didSet`.

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

For the probe files covering these cases (`probes/`), the expansions Xcode's
compiler prints with this plugin loaded in place of Apple's are identical to
the ones it prints with Apple's; `compare-with-xcode.sh` repeats that check.
`examples/ios/CleanRoom` uses both expansions, and its app compiled on Linux
with this plugin is byte-identical to the one compiled on a Mac with Apple's
(`tools/compare_ipa.sh`).
