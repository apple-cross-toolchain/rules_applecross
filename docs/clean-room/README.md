# Clean-room Apple build tools

`tools/apple_tools/crates` reimplements, for Linux, Apple tools that
rules_apple runs while bundling. They ship with the ported tools
(`tools/build-ported-tools.sh`).

No Apple binary was disassembled, decompiled or patched, and no Apple code
was copied. All observations were made on an Apple silicon Mac with an
unmodified Xcode 27.0; the Linux builds compared against it ran in a VM on
that Mac, reading that Xcode through a read-only mount. Sources:

- the tools' command lines and documentation;
- the outputs of Xcode 27.0's tools for constructed inputs;
- Apple's public APIs, called from small programs;
- published sources: the reference LZFSE encoder, CoreFoundation's string
  hash and hash tables, the former open-source `swift-stdlib-tool`, and the
  Security project's `libsecurity_codesigning` (read, not copied).

The tools are strict by default: their output is byte-identical to Xcode's,
or the build fails on the input. Known gaps are on each page; the largest is
tvOS's radiosity image ([actool.md](actool.md)), which fails tvOS app icons.
Relaxed, `--config=relaxed` (in `.bazelrc`: `--action_env` of
`RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed`), accepts a near match
instead: `actool` writes its closest output, or hands what it doesn't
support to xcbuild's actool (`tools/apple_tools/xcbuild`), and `ibtool`
leaves what it can't compile uncompiled, as the stub it replaced did. The
other tools refuse such inputs either way.

| Page | Tool |
| --- | --- |
| [actool.md](actool.md) | `actool`: app icons, CgBI PNGs, Assets.car |
| [codesign.md](codesign.md) | `codesign`: ad hoc signatures and resource seals |
| [ibtool.md](ibtool.md) | `ibtool`: launch storyboards, NIB archives |
| [xcstringstool.md](xcstringstool.md) | `xcstringstool`: String Catalogs to `.strings` and `.stringsdict` |
| [swift-stdlib-tool.md](swift-stdlib-tool.md) | `swift-stdlib-tool`: the Swift runtime libraries copied into apps |
| [swiftui-macros.md](swiftui-macros.md) | `SwiftUIMacros`: SwiftUI's `@State` macro for the Swift compiler on Linux |
| [lzfse.md](lzfse.md) | The streaming LZFSE encoder CoreUI compresses renditions with |
| [foundation-hashing.md](foundation-hashing.md) | Hash-table orders NIB and property-list writers follow |

## Result

`tools/compare_ipa.sh` builds `examples/ios/CleanRoom` (SwiftUI with `@State`,
an app icon, a launch storyboard, ad hoc signed for arm64 devices) on a Mac
with Xcode's tools and in the Linux VM with these. The two `.ipa` files are
identical.
