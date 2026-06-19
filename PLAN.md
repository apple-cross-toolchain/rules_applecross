# PLAN

- [ ] Evaluate `ld64.lld` as an optional Apple linker for rules_applecross remote builds. See [execplans/apple-lld-linker-override.execplan.md](execplans/apple-lld-linker-override.execplan.md).

## RBE Performance Plan

- [ ] Keep benchmark harness: Xcode local baseline plus Bazel remote profile and `aquery` measurements. Every performance change should report wall time, critical path, configured target count, and action input count.
- [ ] Add a canonical `remote_ios_arm64` config with explicit `--platforms`, `--ios_multi_cpus=arm64`, and `--apple_platform_type=ios` so measurements do not accidentally include simulator or multi-platform transitions.
- [ ] Configure BuildBuddy auth/profile upload so remote profile data and cache behavior are visible and reliable.
- [ ] Remove full `toolchain_files` from `environment_plist` runfiles. Replace it with minimal declared plist/SDK metadata inputs or a generated environment plist.
- [ ] Split `toolchain_files` by platform and action. Swift compile, C/ObjC compile, archive, link, bundle, and plist actions should each receive only the files they actually need.
- [ ] Normalize and minimize SDK archive generation: no AppleDouble/xattrs, pre-normalized modulemaps, and ideally platform-sliced archives or filegroups.
- [ ] Register and generate only the Apple toolchains needed by the requested platforms, starting with `ios_arm64` for the RBE demo path.
- [ ] Avoid cold-building musl/LLVM/rules_swift exec tools by using prebuilt static helpers or an executor image with compatible host Swift/glibc dependencies.
- [ ] Decide whether to standardize on a custom BuildBuddy executor image or hermetically bundle Swift host runtime libraries.
- [ ] Upstream or otherwise harden the needed `rules_apple`/`apple_support` integration points so the implementation is maintainable.
