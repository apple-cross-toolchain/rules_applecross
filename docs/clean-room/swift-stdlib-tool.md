# swift-stdlib-tool

Reproduced: `swift-stdlib-tool --copy --platform P --destination D
[--source-libraries DIR]... [--scan-executable F]... [--scan-folder DIR]...`
as rules_apple's `swift_stdlib_tool` wrapper runs it (through `xcrun`), and
`--print`. The behaviour comes from the tool's `--help` and from its former
source in the Swift repository (`tools/swift-stdlib-tool`, Apache 2.0):

- every Mach-O file given or found under a scanned folder is read for the
  dylibs it links (`LC_LOAD_DYLIB`, weak, re-export, lazy and upward);
- names `@rpath/libswift*.dylib` are looked up in the source directories,
  in order; one found there is needed, and its own references are followed;
  one not found is the OS's and is skipped;
- the needed libraries are copied, unmodified, to each destination
  (`--print` lists their paths instead, as Xcode's does);
- without `--source-libraries`, the default is the one `--help` documents,
  `<tool>/../../lib/swift/<platform>`.

rules_apple passes `DEVELOPER_DIR/Toolchains/XcodeDefault.xctoolchain/usr/
lib/swift-*/<platform>` and afterwards thins the copies to the app's
architectures. The toolchain tree keeps the back-deployment runtimes there
(`swift-5.5`: `libswift_Concurrency`; `swift-6.2`:
`libswiftCompatibilitySpan`) as Xcode ships them; `swift-5.0`, holding the
runtime itself for OSes before iOS 12.2, has only text stubs, so apps
deploying that far back are not supported.

For Visor (iOS 18) both tools find only `libswiftCompatibilitySpan.dylib`.
Signing (`--sign`) is refused: rules_apple signs the bundle afterwards.
