# Clean-room Apple build tools

`tools/apple_tools` reimplements, for Linux, Apple tools that rules_apple runs
while bundling. They ship in the ported tools archive
(`tools/linux_vm/build-ported-tools.sh`).

No Apple binary was disassembled, decompiled or patched, and no Apple code
was copied. All observations were made on an Apple silicon Mac with an
unmodified Xcode 27.0; the Linux builds compared against it ran in a VM on
that Mac, reading that Xcode through a read-only mount. Sources:

- the tools' command lines and documentation;
- the outputs of Xcode 27.0's tools for constructed inputs;
- Apple's public APIs, called from small programs;
- published sources: the reference LZFSE encoder, CoreFoundation's string
  hash and hash tables, the former open-source `swift-stdlib-tool`.

Inputs outside what each page describes are refused rather than
approximated.

| Page | Tool |
| --- | --- |
| [actool.md](actool.md) | `actool`: app icons, CgBI PNGs, Assets.car |
| [codesign.md](codesign.md) | `codesign`: ad hoc signatures and resource seals |
| [ibtool.md](ibtool.md) | `ibtool`: launch storyboards, NIB archives |
| [swift-stdlib-tool.md](swift-stdlib-tool.md) | `swift-stdlib-tool`: the Swift runtime libraries copied into apps |
| [swiftui-macros.md](swiftui-macros.md) | `SwiftUIMacros`: SwiftUI's `@State` macro for the Swift compiler on Linux |
| [lzfse.md](lzfse.md) | The streaming LZFSE encoder CoreUI compresses renditions with |
| [foundation-hashing.md](foundation-hashing.md) | Hash-table orders NIB and property-list writers follow |

## Result

`tools/compare_ipa.sh` builds `examples/ios/CleanRoom` (SwiftUI with `@State`,
an app icon, a launch storyboard, ad hoc signed for arm64 devices) on a Mac
with Xcode's tools and in the Linux VM with these. The two `.ipa` files are
identical.
