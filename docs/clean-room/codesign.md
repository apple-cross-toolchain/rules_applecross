# codesign

Reproduced: `codesign -v --sign - [--generate-entitlement-der --entitlements
F] --force --timestamp=none PATH`, as rules_apple runs it, for iOS-style
and macOS bundles and Mach-O files. Output was byte-identical to Xcode 27.0's codesign
on the first comparison, and Apple's `codesign --verify --strict` accepts it.

Also reproduced: signing with an identity (below), with and without
`--options runtime`. Observed: two ad hoc signatures of the same input are
identical; two identity signatures differ only in the signing time their CMS
blob records. This codesign takes that time from `SOURCE_DATE_EPOCH` when it
is set, so an identity-signed `.ipa` can be reproduced byte for byte too.

## The signature

A SuperBlob (big-endian) with, in order: the code directory (slot 0), an
empty requirement set (2), the entitlements as given (5), their DER form
(7), an empty CMS wrapper (0x10000). The code directory is version 0x20400:
flags adhoc, SHA-256, identifier from `CFBundleIdentifier`, special slots
-1 Info.plist, -2 requirements, -3
CodeResources, -5 entitlements, -7 DER entitlements (blob hashes for 5 and
7), the executable segment from `__TEXT` with main-binary and
get-task-allow flags. Space reserved: the SuperBlob's length plus 8,
rounded up to 16 (sweeping identifier lengths 1-48), the length counted with
an 88-byte code directory header whatever the version written. Like
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
`UUID` and of the image's `LC_UUID` (a universal file's preferred slice's,
below) appended (`libFoo.dylib` with UUID
1A12DDC6-... is `libFoo-555549441a12ddc6...`; `some.name.v2` is
`some.name`). A file without `LC_UUID` gets some other suffix, not
established, and is refused.

Entitlements are embedded in DER as well as XML with or without
`--generate-entitlement-der`, as codesign's manual says of macOS 12 and later.

DER entitlements: `[APPLICATION 16] { INTEGER 1, [CONTEXT 16] { SEQUENCE {
UTF8String key, value } ... } }`, keys in byte order.

## The hardened runtime

`--options runtime` sets flag 0x10000 and writes code directory version
0x20500, whose header adds `runtime`, the SDK version from
`LC_BUILD_VERSION` (0x1b0000 for the 27.0 SDK), and a zero
`preEncryptOffset`: 96 bytes instead of 88. The space reserved is still
reckoned with 88, 8 bytes short of the header written (476 bytes of
SuperBlob reserve 480). Other `--options` flags are refused.

## Identity signatures

Observed by signing, in a VM, with a throwaway RSA identity: a leaf
certificate (code signing extended key usage) issued by a self-signed test
CA, both trusted for code signing (Apple-issued identities: below). Inputs: rules_applecross's iOS and macOS examples, a
macOS app holding a universal dylib in `Contents/Frameworks` (signed first,
as rules_apple does), `/bin/echo` (three slices), and a sweep of
identifier lengths 1-48 with and without `--options runtime`. All were
reproduced byte for byte, given codesign's signing time.

This codesign reads the identity from `RULES_APPLECROSS_CODESIGN_KEY` (the
private key, PEM) and `RULES_APPLECROSS_CODESIGN_CERTIFICATES` (its
certificate and those it chains to, PEM), and matches `-s NAME` as codesign
does: the certificate's SHA-1 in hex, or its common name or a prefix of it.

Differences from an ad hoc signature:

- The code directory has no adhoc flag (0 without the runtime) and no team
  identifier: codesign takes that from Apple-issued certificates only.
- The requirement set holds the designated requirement (type 3):
  `identifier ID and certificate root = H"SHA-1 of the chain's root"`,
  compiled as and(ident, anchor hash with slot -1). In text, a string of
  letters, digits and `_` only is left unquoted (`identifier echo1`).
- A Mach-O file named by its file name keeps the bare name; the `-UUID…`
  suffix is for ad hoc signatures only.
- The CMS blob is a SignedData, BER with indefinite lengths for the
  ContentInfo, its `[0]`, the SignedData and the encapsulated content info,
  definite DER within: version 1, digest SHA-256, detached content (the code
  directory), the whole chain root included as a DER SET OF, and one
  SignerInfo (issuer and serial, SHA-256, sha256WithRSAEncryption) whose
  signed attributes, in DER order, are contentType (data), signingTime
  (UTCTime), messageDigest (the code directory's SHA-256),
  1.2.840.113635.100.9.2 (a SEQUENCE of the SHA-256 OID and the code
  directory's SHA-256) and 1.2.840.113635.100.9.1 (an XML property list,
  `cdhashes`, of that hash's first 20 bytes). PKCS#1 v1.5 signatures are
  deterministic, so the same key and time give the same bytes.
- No secure timestamp is added unless asked for (`--timestamp[=URL]`; below).
- Space reserved: everything before the CMS blob, plus 18000 bytes for it,
  plus 8, rounded up to 16.
- A bundle's seal records nested code signed by an identity with its
  designated requirement in text, e.g. `identifier "com.apple.dt.runtime.swiftCompatibilitySpan"
  and certificate root = H"4b76…"` (hex in lowercase), beside its cdhash.

### Apple-issued identities

Observed from apps codesign had signed with Apple Development and Developer
ID identities (no private key needed to read them), and checked by signing
those apps' inputs again with their certificate chains and another key:
everything matched but the 256 bytes of the RSA signature value.

- The chain in the CMS is the leaf, its Apple intermediate and the Apple
  Root CA.
- The team identifier, the leaf's organizational unit, follows the
  identifier in the code directory (`teamOffset`), and the hash slots follow
  it.
- Which designated requirement depends on the intermediate's marker
  extension, compiled as right-nested `and`s:
  - WWDR (1.2.840.113635.100.6.2.1; Apple Development and Distribution):
    `identifier ID and anchor apple generic and certificate leaf[subject.CN]
    = "CN" and certificate 1[field.1.2.840.113635.100.6.2.1] /* exists */`;
  - Developer ID (1.2.840.113635.100.6.2.6, leaf 1.2.840.113635.100.6.1.13):
    `identifier ID and anchor apple generic and certificate
    1[field.1.2.840.113635.100.6.2.6] /* exists */ and certificate
    leaf[field.1.2.840.113635.100.6.1.13] /* exists */ and certificate
    leaf[subject.OU] = TEAM`.
  Other Apple-issued identities are refused.
- Apple Development signatures carried no timestamp. Developer ID ones are
  timestamped unless `--timestamp=none` is given.
- The text of these forms in a bundle's seal matched the 38 requirements
  codesign recorded for Developer ID nested code in two apps.

### Secure timestamps

`--timestamp` asks Apple's authority, `http://timestamp.apple.com/ts01`, and
`--timestamp=URL` another, by RFC 3161 over HTTP: a TimeStampReq of version
1, the SHA-256 (with a NULL parameter) of the RSA signature value, a 64-bit
nonce and certReq true. The token returned, with the authority's
certificates, is the SignerInfo's one unsigned attribute,
id-aa-timeStampToken (1.2.840.113549.1.9.16.2.14), in a definite-length [1]
after the signature. Compared with codesign's for the same input, the CMS
had the same structure throughout; tokens differ from request to request
(time, serial, nonce, and which of Apple's signers answered), and Apple's
codesign showed `Timestamp=` for and verified the one this codesign got. The
space reserved is unchanged: 18000 bytes cover a timestamp.

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

A macOS bundle (`Contents/Info.plist`) is sealed from `Contents`: the
executable is `Contents/MacOS/<CFBundleExecutable>` and `_CodeSignature`
goes in `Contents`. Its rule sets differ: v1 seals `^Resources/` (with the
same `.lproj` rules under it) and `^version.plist$`; v2 adds `^Resources/`
at weight 20 and two `nested` rules at weight 10,
`^(Frameworks|SharedFrameworks|PlugIns|Plug-ins|XPCServices|Helpers|MacOS|Library/(Automator|Spotlight|LoginItems))/`
and `^[^/]+$`.

What those rules match is nested code, which must already be signed (rules_apple
signs it first). Apple's published code-signing sources (`libsecurity_codesigning`
in the Security project, APSL 2.0) describe the scan, and Xcode 27.0's codesign
confirms it on a constructed app:

- a file there is sealed as `{cdhash, requirement}`;
- a directory there whose name has a dot is a nested bundle, sealed the same
  way as one entry, its contents not listed; one without a dot is entered;
- a symlink stays a symlink entry;
- unsigned code, or a file there that isn't code, fails with "code object is
  not signed at all", naming it as the subcomponent.

`cdhash` is the code directory's digest by its own hash type (SHA-256, or
SHA-1), its first 20 bytes. `requirement` is the ad hoc designated
requirement: `cdhash H"<hex>"` for each architecture, joined with ` or `,
the preferred architecture's first (and its hash is `cdhash`), then the
others in file order. A nested bundle's code is its main executable: in
`Contents/MacOS`, beside a framework version's `Resources/Info.plist`, or
beside `Info.plist`. Nested code with alternate code directories, or a
universal file without an arm64 slice, is refused.

The preferred architecture is the one the signing Mac runs: arm64e, else
arm64. Xcode 27 runs on macOS 27, which Apple silicon Macs alone run, so
that is the same for every Xcode 27. It also names a lone universal file
(its `LC_UUID`, above).

A versioned framework (`Versions/Current/Resources/Info.plist`) is sealed
from its current version with the macOS rules: its executable is at the
version's root, its Info.plist is bound and also sealed as
`Resources/Info.plist`, and `_CodeSignature` goes in the version.

## Universal files and `__LINKEDIT`

`__LINKEDIT`'s size in memory is rounded up to 16 KiB, whatever the
architecture or the page size code is hashed in (x86_64 for macOS and the
iOS simulator included). A universal file is laid out again with every
slice aligned to at least 16 KiB (2^14), from the first offset after the
fat header.

A constructed macOS app (a dylib, a two- and a three-architecture dylib, a
helper, a versioned framework with symlinks, a symlink and a resource),
signed inner code first as rules_apple does, is byte-identical to Xcode
27.0's codesign, and `codesign --verify --deep --strict` accepts it; an iOS
app with a nested dylib and app extension still is.
