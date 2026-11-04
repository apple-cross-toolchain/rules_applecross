# docc

Not reimplemented: Xcode's `docc` is swift.org's open-source docc
(swiftlang/swift-docc), so off a Mac `xcrun docc` runs swift.org's, of the
Swift that built the SDKs (`toolchain/stubs/docc.sh`, installed as
`usr/bin/docc` with the docc binary and its Swift libraries in
`usr/libexec/docc`, linked from that Swift's toolchain repository).
rules_apple's `docc_archive` runs it as on a Mac.

What differs is the render template docc copies into every archive (its
HTML, JavaScript, styles and images): Xcode's is its own build of
docc-render, not the swift.org toolchain's, and matches no published
swift-docc-render-artifact build (31 of its 34 scripts at best). So the SDK
archive carries Xcode's (`usr/share/docc`, from `tools/package-sdks.sh`, an
Xcode `.xip` or a mounted Xcode), and the stub gives it to docc
(`DOCC_HTML_DIR`). An SDK archive packaged before that has none: strict, the
build fails; `--config=relax_docc` uses swift.org's template, whose pages
work the same but whose files differ from Xcode's.

## Equivalence

Xcode's docc writes each JSON object's keys in a randomly seeded order
(Swift dictionaries), so two of its runs differ; ours sets
`SWIFT_DETERMINISTIC_HASHING=1` and writes one order.
`tools/xcode_equivalence.py` compares an archive's JSON with its keys
sorted, and every other file byte for byte.

swift.org 6.4.2's docc on Linux, with Xcode 27.2's template, wrote archives
equivalent to Xcode 27.2's for a small Swift module with a catalog
(articles, an extension file, `@Metadata`, a table, images with 2x and dark
variants, a tutorial table of contents and a tutorial), with and without
`--transform-for-static-hosting`; built with `docc_archive` on Linux too.

## Notes

- On Linux docc takes a catalog that is a link to a directory for none, and
  rules_apple runs `docc_archive` outside the sandbox, where the execroot
  links the workspace's top-level directories; the stub passes such a path
  resolved.
- docc needs the libraries swift.org's toolchain needs on the host,
  libcurl and libxml2 among them (Foundation's networking and XML).
