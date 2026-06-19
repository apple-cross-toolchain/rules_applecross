# Use rules_swift Hermetic Swift Tools in Applecross

## Context

`rules_applecross` currently downloads a Linux Swift archive inside the
`apple_cross_toolchain` repository rule and copies Swift binaries and host
runtime files into the generated fake Xcode layout:

```text
Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/swift*
Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/**
```

This makes Apple Swift actions work on Linux, but it duplicates new upstream
`rules_swift` functionality and makes the fake Xcode repository a hybrid of:

- Xcode-derived Apple SDK material.
- Linux Swift.org host toolchain material.
- LLVM/ported Apple tool material.

Upstream `rules_swift` now has a Bzlmod `swift.toolchain` extension and
`swift_tools` provider. That support downloads Swift.org distributions, exposes
Swift executables as labels, and carries required host runtime inputs into
Swift actions.

Applecross still needs to own Apple-specific target semantics:

- Xcode-derived Apple SDKs.
- Apple target Swift resources under `usr/lib/swift/{iphoneos,...}`.
- Apple target triples.
- `-sdk` and `-resource-dir` wiring.
- Apple Swift link options.
- `rules_apple` fake-Xcode compatibility.

The simplification target is therefore:

```text
rules_swift owns Linux-host Swift compiler payload.
rules_applecross owns Apple target SDK/resource/link semantics.
```

## Goals

- Stop downloading Swift inside `apple_cross_toolchain.bzl`.
- Stop copying Linux Swift binaries into fake Xcode.
- Stop copying Linux Swift host runtime files into fake Xcode.
- Use upstream `rules_swift` `swift.toolchain` extension to fetch the Linux
  Swift distribution.
- Use upstream `SwiftToolsInfo` in the Applecross Swift toolchain.
- Keep fake Xcode Apple SDK and Apple target Swift resource files unchanged.
- Keep the first implementation conservative: keep Applecross's custom Swift
  target toolchain, only replace its host Swift executable source.

## Non-goals

- Do not replace Applecross fake Xcode materialization.
- Do not replace Apple SDK packaging.
- Do not replace Apple C/C++/ObjC toolchains.
- Do not implement asset catalogs, Metal, or real codesigning.
- Do not try to register upstream standalone `swift_toolchain` as an iOS target
  Swift toolchain; it is not an Apple app target toolchain.

## Implementation Steps

1. Bump the `rules_swift` git override to a commit that includes PR #1630 and
   the `swift_tools` provider.

2. Add an upstream `rules_swift` Bzlmod extension instance:

   ```starlark
   applecross_swift = use_extension(
       "@build_bazel_rules_swift//swift:extensions.bzl",
       "swift",
   )
   applecross_swift.toolchain(
       name = "applecross_swift",
       swift_version = "6.2.1",
       platform_sha256 = {
           "ubuntu24.04": "...",
       },
   )
   use_repo(
       applecross_swift,
       "applecross_swift",
       "applecross_swift_ubuntu24.04",
   )
   ```

3. Pass the generated Swift tools target into Applecross:

   ```starlark
   apple_cross_toolchain.configure(
       name = "apple_cross_toolchain",
       apple_sdk_path = "...",
       swift_tools = "@applecross_swift_ubuntu24.04//:tools",
   )
   ```

4. Remove `swift_path`, `swift_urls`, `swift_sha256`, and `swift_strip_prefix`
   from the Applecross module extension and repository rule.

5. Remove Swift archive extraction and copy logic from
   `toolchain/apple_cross_toolchain.bzl`.

6. Remove the generated Linux Swift toolchain rooted at fake Xcode from
   `toolchain/BUILD.template.bzl`.

7. Add a `swift_tools` label attribute to `toolchain/swift_toolchain.bzl`.

8. Change Applecross Swift `ToolConfigInfo` generation:

   - compile actions use `swift_tools.swift_driver`.
   - symbol graph actions use `swift_tools.swift_symbolgraph_extract`.
   - additional action inputs include `swift_tools.additional_inputs` and the
     existing Applecross SDK/toolchain files.

9. Keep Apple resource behavior unchanged:

   ```text
   -resource-dir external/.../XcodeDefault.xctoolchain/usr/lib/swift
   -sdk external/.../Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS*.sdk
   ```

10. Build verification:

    ```sh
    bazel --nohome_rc build //tests/data:hello_swift
    bazel --nohome_rc build --config=remote --platforms=@build_bazel_apple_support//platforms:ios_arm64 //tests/data:hello_swift
    ```

    If available, also verify the tiny iOS app target.

## Expected Simplification

Repository rule no longer owns Swift.org distribution logic:

- no Swift URL defaults.
- no Swift archive extraction.
- no copying `swift*` host binaries into fake Xcode.
- no overlay deletion for Linux Swift modules.
- no generated fake-Xcode Linux Swift toolchain.

Applecross still keeps:

- Xcode-derived Apple target Swift resources.
- fake Xcode layout.
- LLVM/ported tools under fake Xcode.
- Apple-specific Swift target toolchain.

## Risks

- Swift.org 6.2.1 host compiler must remain compatible with Xcode 26.1.1 Swift
  target resources.
- Swift macros/plugins may need additional work because Xcode's host plugin
  dylibs are macOS binaries; simple Swift compilation should avoid this path.
- Action input size remains high until `toolchain_files` is split by action and
  platform.
- Upstream `rules_swift` main may have API movement after PR #1630; pinning a
  known commit keeps this deterministic.
