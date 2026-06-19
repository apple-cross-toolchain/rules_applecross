# Xcode iOS App Baseline

This benchmark builds the same tiny UIKit app fixture as `//tests/data:hello_ios_app`,
but with local `xcodebuild` on macOS.

Run:

```sh
benchmarks/xcode/benchmark.sh clean
benchmarks/xcode/benchmark.sh warm
benchmarks/xcode/benchmark.sh incremental
```

Modes:

- `clean`: uses a fresh DerivedData directory.
- `warm`: primes the stable DerivedData directory, then times a no-change build.
- `incremental`: primes the stable DerivedData directory, touches `tests/data/main.swift`,
  then times the rebuild.

Comparable Bazel command:

```sh
bazel --nohome_rc build --config=remote \
  --platforms=@apple_support//platforms:ios_arm64 \
  --ios_multi_cpus=arm64 \
  --apple_platform_type=ios \
  //tests/data:hello_ios_app
```
