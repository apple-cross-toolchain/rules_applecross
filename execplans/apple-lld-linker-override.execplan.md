# Evaluate ld64.lld Linker Override

## Goal

Determine whether rules_applecross can link Apple/Mach-O binaries on Linux RBE with `ld64.lld` instead of the current ported cctools `ld`, using a composable linker override and without rewriting the apple_support toolchain.

## Context

The current remote iOS app path reaches Swift compilation and then links through the Linux-hosted cctools `ld` from `ported-tools-linux-x86_64.tar.xz`. That linker is not identical to modern Xcode `ld`; at least `-no_warn_duplicate_libraries` is rejected. LLVM's Mach-O linker may be a better fit if an `ld64.lld` binary is available in the existing LLVM prebuilt, or if a small additional linker artifact is acceptable.

`keith/rules_apple_linker` proves the integration model: inject `--ld-path=<linker>` via `CcInfo` / Objc provider linkopts and add the linker binary as an input. Prefer that style over replacing the whole cc toolchain.

## Non-Goals

- Do not mutate or redistribute the private Xcode SDK archive.
- Do not replace apple_support's rule-based toolchain wholesale.
- Do not import `rules_apple_linker` as a dependency before proving the minimal local path.
- Do not make asset catalog, provisioning, or signing behavior part of this experiment.

## Plan

1. Verify linker availability.
   - Inspect `@llvm_prebuilt` for `ld64.lld`, `ld.lld`, and `lld`.
   - If `ld64.lld` is absent, check whether the `llvm` module can expose/build it.
   - If not, record the exact external artifact option, such as `keith/ld64.lld`, before adding any dependency.

2. Add a minimal local linker override target.
   - Model it after `rules_apple_linker`: a small rule returns `CcInfo` and Objc provider linkopts.
   - Inject only `--ld-path=<execpath-to-ld64.lld>` at first.
   - Make the linker binary an action input.
   - Wire the override into only the tiny `ios_application` target or its app library.

3. Keep apple_support features composable.
   - Keep official apple_support labels in known features.
   - Do not remove upstream feature definitions.
   - Disable only applecross-incompatible default features when needed.
   - Retest `-no_warn_duplicate_libraries` and `-reproducible` separately with `ld64.lld`; enable them only if the selected linker accepts them.

4. Run a focused target ladder.
   - `bazel --nohome_rc build --config=remote --platforms=@apple_support//platforms:ios_arm64 //tests/data:hello_swift`
   - `bazel --nohome_rc build --config=remote --platforms=@apple_support//platforms:ios_arm64 //tests/data:hello_ios_app`
   - If `ios_application` still transitions to simulator, add a follow-up target or flags to force generic device arm64 and verify that path too.

5. Capture linker compatibility failures.
   - Save the exact failing link command params file.
   - Classify each failure as unsupported flag, missing SDK input, missing TAPI behavior, Swift runtime link issue, or lld correctness gap.
   - Prefer targeted linkopts/feature toggles over wrapper string rewriting.

6. Compare performance only after correctness.
   - Run warm-cache comparisons for cctools `ld` versus `ld64.lld`.
   - Record BuildBuddy invocation URLs.
   - Compare link action wall time, critical path, and total elapsed time.

## Acceptance Criteria

- The tiny Swift library still builds remotely on Linux.
- The tiny `ios_application` links with `ld64.lld` on Linux RBE, or the plan records the precise blocker and the smallest next patch.
- The implementation uses provider-based linker override wiring, not a full copied toolchain config.
- Any apple_support divergence is explicitly documented as applecross-only linker compatibility behavior.

## Rollback

Remove the linker override dependency from the tiny app target and rebuild with the ported cctools linker. No SDK archive changes should be required.
