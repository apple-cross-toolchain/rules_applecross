# iOS VM end-to-end test

This directory is a standalone consumer module containing the real
`//:UnitTests` target exercised by the Linux VM CI job. Its local override points
back to this source checkout, while the rest of its setup is what an external
`rules_applecross` consumer needs. The target is tagged `manual` because its
inputs contain Apple firmware and an Xcode XCTest runtime and therefore cannot
be committed here.

Stage these files in `assets/` before invoking the target. The `ios_vm`
extension in `MODULE.bazel` selects that flat directory as the local iPhone
12/iOS 26.5 firmware runtime named `iphone12_ios26_5`, selects the QEMU
executable separately, and exposes the combined runtime as
`@ios_vm_runtime`:

- `qemu-system-aarch64`: native Linux binary built from darwin-vm's QEMU fork
- `bootkc` and `dtree`: matching non-SPTM device firmware
- `ramdisk.dmg`, `ramdisk.tc`, and `slot.json`: output from darwin-vm's
  `xctest/prepare_ramdisk.sh`

The extension is the sole source of the named runtime's device type and OS
version and materializes it as the typed target
`@ios_vm_runtime//:iphone12_ios26_5`. The runner selects that label; the shared
QEMU implementation remains private inside the runtime target. The manifest
binds its firmware and prepared-image hashes, so mixing files from different
asset sets fails before QEMU starts.

## CI pipeline

The workflow never stores VM inputs in git:

1. `package_ios_vm_assets` runs on macOS 26, clones the pinned
   `apple-cross-toolchain/darwin-vm` revision, extracts the non-SPTM iPhone 12
   firmware, and prepares the XCTest ramdisk with Xcode 26.6.
2. The job packages the five firmware/image files into a one-run transfer
   artifact.
3. `ios_vm_test` runs on `ubuntu-24.04-arm`, clones the same darwin-vm
   revision with its QEMU submodule, builds QEMU, extracts the artifact into
   the ignored `assets/` directory, verifies every manifest hash, and runs
   Bazel from this consumer workspace using the extension's local paths.
4. GitHub discards the hosted runners after their jobs. The transfer artifacts
   are deleted through the GitHub API; one-day retention is the cancellation
   fallback.

Configure the `BUILDBUDDY_API_KEY` repository secret already used by the other
remote-execution jobs. The jobs run for pushes and same-repository pull
requests; fork pull requests have no access to that secret and skip the
pipeline.

The Linux job executes from this directory:

```sh
bazel test \
  --config=remote \
  --remote_header=x-buildbuddy-api-key="$BUILDBUDDY_API_KEY" \
  --apple_platform_type=ios \
  --ios_multi_cpus=arm64 \
  //:UnitTests
```
