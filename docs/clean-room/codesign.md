# codesign

Reproduced: `codesign -v --sign - [--generate-entitlement-der --entitlements
F] --force --timestamp=none PATH`, as rules_apple runs it, for iOS-style
bundles and Mach-O files. Output was byte-identical to Xcode 27.0's codesign
on the first comparison, and Apple's `codesign --verify --strict` accepts it.

Observed: two ad hoc signatures of the same input are identical; two
identity signatures are not (their CMS blob records the signing time), so a
signed `.ipa` can only be reproduced byte for byte when signed ad hoc.

## The signature

A SuperBlob (big-endian) with, in order: the code directory (slot 0), an
empty requirement set (2), the entitlements as given (5), their DER form
(7), an empty CMS wrapper (0x10000). The code directory is version 0x20400:
flags adhoc, SHA-256, identifier from `CFBundleIdentifier`, special slots
-1 Info.plist, -2 requirements, -3
CodeResources, -5 entitlements, -7 DER entitlements (blob hashes for 5 and
7), the executable segment from `__TEXT` with main-binary and
get-task-allow flags. Space reserved: the SuperBlob's length plus 8,
rounded up to 16 (sweeping identifier lengths 1-48). Like
`codesign_allocate`, a missing `LC_CODE_SIGNATURE` is added after the load
commands and the signature placed 16-byte aligned after `__LINKEDIT`, which
grows to cover it. An existing signature is written over in place: bytes of
it beyond the new SuperBlob are left as they were (re-signing Xcode's
identity-signed Visor app ad hoc leaves the tail of the old CMS blob behind).

Page size, from unsigned dylibs built with `clang -target` for each
platform and deployment target and signed ad hoc: 16 KiB for arm64, except
iOS devices below iOS 16.0 (15.99 and earlier) and tvOS devices at every
deployment target tried (16.0-26.0), which get 4 KiB, as does x86_64. The
iOS simulator, watchOS, visionOS and macOS on arm64 get 16 KiB. Executables
and dylibs follow the same rule.

A lone Mach-O file (a dylib in `Frameworks`) is identified, without `-i`, by
the `CFBundleIdentifier` of the Info.plist embedded in its
`__TEXT,__info_plist`, which is also bound in slot -1. Without one: the
file name without its extension, and when that has no dot, `-`, the hex of
`UUID` and of the image's `LC_UUID` appended (`libFoo.dylib` with UUID
1A12DDC6-... is `libFoo-555549441a12ddc6...`; `some.name.v2` is
`some.name`). A file without `LC_UUID` gets some other suffix, not
established, and is refused.

DER entitlements: `[APPLICATION 16] { INTEGER 1, [CONTEXT 16] { SEQUENCE {
UTF8String key, value } ... } }`, keys in byte order.

## CodeResources

An XML property list as CoreFoundation writes it (tabs, keys in UTF-16
order, base64 at 76 columns counting a tab as 8, reals `%.17g`), with
`files` (SHA-1 under the v1 rules), `files2` (SHA-256 `hash2` under the v2
rules) and the iOS default rule sets. Those mark nothing as nested code, so
the files of a nested bundle (`PlugIns/visor_widget.appex/...`) and nested
libraries (`Frameworks/libswiftCompatibilitySpan.dylib`) are sealed by
their hashes like any other file, as Xcode's codesign seals Visor's.
Signing Visor's library, widget and app ad hoc, in that order, gives the
same bytes as Apple's codesign.
