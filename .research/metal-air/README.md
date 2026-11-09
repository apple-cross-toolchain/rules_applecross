# Metal shaders on Linux (research, 2026-11-09)

The goal is for `--config=relax_metal` to produce working `.metallib`s
instead of today's empty file, which fails at run time. Byte-exact Metal
stays Mac-only. The approach is clean-room: rewrite the MSL subset into
C++17 for clang 15, the last clang with typed pointers (which AIR uses);
add the AIR metadata ourselves; and write the metallib container in our
own code. Everything is learned black-box from Xcode's output for our own
probe shaders, on leased VMs. This is not for merging.

## First end-to-end result

The kernel `kernel void k(device float *a [[buffer(0)]], uint i
[[thread_position_in_grid]]) { a[i] *= 2; }` was built on Linux like this:

1. Rewrite by hand into `k.cpp`: `device` becomes `address_space(1)`, the
   function is `extern "C"`, and the attributes go (their meaning moves
   into metadata).
2. Compile with clang 15.0.6 (LLVM's aarch64 Linux release):
   `clang++ -std=c++17 -target spir64 -O2 -ffast-math -fno-builtin
   -Xclang -no-opaque-pointers -S -emit-llvm`. The function body and its
   attributes come out the same as in Xcode's AIR for the same shader.
3. `toair.py` sets the AIR triple and datalayout, drops `dso_local
   spir_func`, marks the buffer argument `"air-buffer-no-alias"`, and adds
   the AIR metadata: module flags (SDK version, buffer, texture and sampler
   limits), `!air.kernel` with argument descriptors, compile options, and
   the AIR and language versions. It is hard-coded for this kernel so far.
4. `llvm-as` from LLVM 15 writes the bitcode. For an `apple` triple it
   adds the `0B17C0DE` wrapper itself.
5. `mtlb.py` writes the metallib (the format is in [FORMAT.md](FORMAT.md)).

On a `mac` slot (macOS 27.0.1, paravirtual GPU), `run.swift` loaded this
metallib with `makeLibrary(data:)`, listed `k`, and ran it. The output
buffer was identical to the one from Xcode's metallib of the same source.

## Status

The plan, sent to the Manager:

- **M0, done:** probed our own shaders (`probes/`, `samplers.sh`) and a
  local corpus with Xcode on a leased macOS VM. [FORMAT.md](FORMAT.md) has
  the results:
  - the metallib container (multi-function, vertex and fragment,
    function-constant metadata), with all 358 reference files rebuilt
    byte for byte by `mtlb.py` (`rebuild.py`);
  - the AIR versions per target;
  - the argument descriptors for each feature;
  - function constants;
  - the constexpr-sampler encoding;
  - what changes with the deployment target.
- **M1, next:** generate the metadata from the source instead of
  hard-coding it, plus a Rust writer.
- **M2:** a stdlib subset for compute.
- **M3:** graphics functions.
- **M4:** replace the relax_metal stub.
- **M5:** widen.

## Clean-room provenance

- The approach (rewrite MSL into C++, compile with clang 15 for typed
  pointers, add the AIR metadata) comes from a friend of Logan's who built
  this for a subset he needs. His code was not shared; he answered
  questions only.
- He learned the AIR metadata black-box (probe shaders, `llvm-dis`, `xcrun
  metal`) and by "observing Apple's libraries", which he says meant mostly
  the `metal_stdlib` headers.
- This work does not read or copy those headers. Its stdlib subset is
  written from observed behaviour (the AIR Xcode emits for our own probe
  shaders) and Apple's public Metal Shading Language Specification.
- The metallib format is learned from Xcode's output for our own probe
  shaders only.
- Measurement corpus: Logan's projects, plus public, permissively licensed
  shader projects downloaded to /tmp for measurement and never committed.
