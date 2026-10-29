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
- Apple's public CreateML framework, called from small programs, to make test
  models of the types Create ML exports.

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

## Deviations

The tools are strict by default: their output is byte-identical to Xcode's,
or the build fails on the input. Known gaps are on each page; the largest are
tvOS's radiosity image and Icon Composer documents ([actool.md](actool.md)).
Each deviation from Xcode's output that a tool can make instead is accepted
on its own, by a `--config=relax_*` in `.bazelrc` (an `--action_env` of
`RULES_APPLECROSS_RELAX_<DEVIATION>=1`), so a project accepts only the ones
it can live with; `--config=relaxed` accepts them all. The table is in the
top-level [README.md](../../README.md#byte-identical-output):
- `actool`: Icon Composer documents rendered flat; tvOS radiosity images
  that may round apart from Xcode's; catalogs it can't reproduce handed to
  xcbuild's actool (`tools/apple_tools/xcbuild`);
- `ibtool`: files it can't compile left uncompiled, as the stub it replaced
  did;
- Metal: an empty library;
- `xcstringstool`: entries not yet reproduced, in their nearest form;
- `appintentsmetadataprocessor`: intents not yet reproduced, left out, and
  Flexible Matching's assets left out.

`coremlc` and the other tools have no near match to write, and refuse such
inputs either way.

## Determinism

These tools write the same bytes every time for the same input. Not all of
Xcode's do. Where an Xcode tool varies its output from run to run, ours keep
one order, and comparisons with Xcode's output allow exactly that variation.
`tools/xcode_equivalence.py` lists each one and puts both sides in a single
order before comparing them byte for byte. The tests and
`tools/compare_examples.sh` use it.

Known variations:
- `coremlc` writes a model's user-defined metadata (a protobuf map) in a
  randomly seeded order, in `coremldata.bin` and `metadata.json`, and so the
  other maps of a specification it copies into `coremldata.bin` (a custom
  model's parameters, a categorical mapping's map)
  ([coremlc.md](coremlc.md#xcodes-own-output-varies)). Ours keeps the
  model's own order. A package's Metadata overlay adds its creator-defined
  entries to that map; packages compiled twice varied in nothing else.
- `actool` writes the blocks of `Assets.car`'s `APPEARANCEKEYS` tree in a
  random order when a catalog has more than one appearance
  ([actool.md](actool.md)). The comparison requires the same entries, the
  same blocks and every other byte the same. With an Icon Composer `.icon`,
  Xcode also names renditions with a fresh UUID and varies a facet's part;
  strict mode refuses `.icon` documents, so that isn't compared.
- `appintentsmetadataprocessor` varies the order of some arrays unless
  `SWIFT_DETERMINISTIC_HASHING=1` is set, as rules_apple sets it
  ([appintentsmetadataprocessor.md](appintentsmetadataprocessor.md)).

An audit ran Xcode 27.0's `actool`, `ibtool`, `xcstringstool`,
`appintentsmetadataprocessor`, `momc`, `mapc`, `swift-stdlib-tool` and ad hoc
`codesign` at least five times on each of several inputs, from different
input and output paths. Apart from the variations above, every output was
the same on every run. (`actool --export-dependency-info` names absolute
paths, and `xcstringstool` writes different partial output for an input it
fails on.)

| Page | Tool |
| --- | --- |
| [actool.md](actool.md) | `actool`: app icons, CgBI PNGs, Assets.car |
| [codesign.md](codesign.md) | `codesign`: ad hoc signatures and resource seals |
| [ibtool.md](ibtool.md) | `ibtool`: launch storyboards, NIB archives |
| [coremlc.md](coremlc.md) | `coremlc`: ML Programs and the other Core ML model types but neural networks, Create ML's included, compiled to `.mlmodelc`, and their Swift and Objective-C classes |
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
