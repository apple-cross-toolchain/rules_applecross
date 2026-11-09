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
5. `mtlb.py` writes the metallib. Given Xcode's own module, it rebuilds
   Xcode's `.metallib` byte for byte:
   - header: `MTLB`, six u16s, then nine u64s (total size, then offset and
     length of the function list, public metadata, private metadata and
     the module);
   - one entry per function: tags NAME, TYPE (2 = kernel), HASH (SHA-256
     of the wrapped module, padded to 16 bytes), MDSZ, OFFT, VERS;
   - then `ENDT` tags and the metadata sections.

On a `mac` slot (macOS 27.0.1, paravirtual GPU), `run.swift` loaded this
metallib with `makeLibrary(data:)`, listed `k`, and ran it. The output
buffer was identical to the one from Xcode's metallib of the same source.

## Next

The plan, sent to the Manager:
- **M0:** probe a shader suite and the corpus with Xcode on a macOS VM, to
  learn the metadata for each feature and what changes with the deployment
  target.
- **M1:** generate the metadata from the source instead of hard-coding it,
  plus a Rust writer.
- **M2:** a stdlib subset for compute.
- **M3:** graphics functions.
- **M4:** replace the relax_metal stub.
- **M5:** widen.
