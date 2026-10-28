# Clean-room Apple build tools

`tools/apple_tools/crates` (Rust, shipped with the ported tools, see
`tools/build-ported-tools.sh`) and `toolchain/stubs/` (Python, installed by
the toolchain) reimplement, for Linux, Apple tools that rules_apple runs
while bundling.

## How they were made

Every tool here was developed clean-room, as a black box. Each one was
derived from what an unmodified Xcode 27.0 tool does with inputs written for
the purpose, and from published material. None of it comes from Apple's
implementation.

- **Ruled out, for every tool:**
  - disassembling, decompiling or patching any Apple binary;
  - reading an Apple binary's code, symbols or strings to learn how it works;
  - copying Apple code.
- **Allowed sources:**
  - the tools' command lines, `--help` and Apple's documentation;
  - the outputs of Xcode 27.0's tools for constructed inputs, compared byte for byte;
  - Apple's public APIs, called from small programs;
  - published sources and specifications, read rather than copied:
    - the reference LZFSE encoder;
    - CoreFoundation's string hash and hash tables (CFString.c, CFBasicHash);
    - the former open-source `swift-stdlib-tool`;
    - the Security project's `libsecurity_codesigning`;
    - the Swift standard library's hashing (`Hasher`, SipHash-1-3), `Dictionary` layout and string escaping;
    - Apple's Core ML model specification and coremltools (which build test models).

All observations were made on an Apple silicon Mac with that Xcode. Where a
tool runs on Linux, its Linux build was compared against the Mac's output,
in a VM on that Mac reading the Xcode through a read-only mount, or as the
same pure-Python program.

Each tool's page has a **Provenance** paragraph (or **Method** paragraph)
giving:
- the experiments behind it, and how many inputs were compared;
- what it reproduces;
- what it refuses because the behaviour isn't established.

The tests keep a sample of those comparisons with Xcode's output, so a reader
can see the evidence for what each tool writes.

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
| [appintentsmetadataprocessor.md](appintentsmetadataprocessor.md) | `appintentsmetadataprocessor`: App Intents metadata from Swift constant values |
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
