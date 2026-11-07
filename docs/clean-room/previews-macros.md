# SwiftUI's #Preview

`#Preview` is declared by SwiftUI as a macro of Xcode's `PreviewsMacros`
plugin, which a Swift compiler on Linux cannot load. `tools/apple_tools/
swiftui_macros` builds a clean-room `PreviewsMacros` executable, passed to
Linux compiles as `-load-plugin-executable .../PreviewsMacros#PreviewsMacros`,
written from the expansions Xcode's compiler prints with `-Xfrontend
-dump-macro-expansions`.

`#Preview(_ name: String? = nil, body:)` of a SwiftUI view (`SwiftUIView`,
two probes: with and without a name) becomes a type named by the compiler's
unique name for `PreviewRegistry`, available from iOS 17, macOS 14, tvOS 17,
visionOS 1 and watchOS 10, conforming to `DeveloperToolsSupport.PreviewRegistry`:
its `fileID`, `line` and `column` are the macro's, and `makePreview()` makes a
`DeveloperToolsSupport.Preview`, with the name if given, of the body's
statements built by a nested `@SwiftUI.ViewBuilder` function. The body may be
the trailing closure or a `body:` argument.

Previews with traits, and those of UIKit views and view controllers or of
widgets (the plugin's other macro types), are refused.
