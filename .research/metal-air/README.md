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
- **M1, in progress:** `msl2air.py` generates the metadata from the source.
  - **How it works:**
    1. A textual rewrite turns `[[attributes]]` into annotations, and vector
       constructors, half literals, function constants and constexpr
       samplers into C++. Our own `include/metal_stdlib` turns stage
       keywords into macros.
    2. **The AST pass** (clang 15, with address spaces as attributes) gives
       the signatures, struct members and `[[id(n)]]`s;
       `-fdump-record-layouts` gives the offsets.
    3. **Codegen compiles flat**, with no address spaces. A generated
       `extern "C"` entry point gives each function AIR's signature, with
       its address spaces, and casts the arguments to flat. Textures and
       samplers become AIR's opaque pointers.
    4. **IR passes:**
       - every function is forced inline;
       - AIR calls that return structs are made direct (spir64 uses sret);
       - program-scope constants go to address space 2;
       - atomics resolve to `air.atomic.global/local` by tracing their
         pointer's address space;
       - `opt -O2`, LLVM's `infer-address-spaces` (address space 0 as
         flat), then `-O2` again, put every load and store in its real
         address space;
       - function constants get AIR's globals and static init.
    5. The final pass adds the AIR metadata, and `mtlb.py` writes the
       metallib with its public metadata (CNST).
  - **Why flat:** MSL member functions and operators must work on objects
    in device and constant memory, e.g. `uniforms.mvp[3]` or
    `args.tex.sample()` in an argument buffer. Plain C++ can't call a
    method on an object in another address space.
    - C++ for OpenCL without the generic address space comes closest to
      MSL's model, but clang declares implicit constructors with a generic
      `this`, so ordinary structs break.
    - With the generic space kept, generic pointers would reach the AIR.
  - **Results (probes the stdlib covers so far):**
    - **Coverage:**
      - buffers, built-ins, threadgroup memory, atomics, math and
        matrices;
      - textures of each kind (read, write, sample, read_write);
      - sampler arguments and constexpr samplers;
      - simdgroup and quadgroup functions, function constants;
      - argument buffers (textures, arrays of textures, samplers,
        pointers, references, plain values);
      - structs with nested structs, arrays, packed vectors and matrices;
      - several kernels per file, `max_total_threads_per_threadgroup`.
    - **Metadata:** AIR metadata matches Xcode's node for node, and the
      metallib's header, function list and public metadata match too. That
      holds on iOS 12, 16, 18, 27.2 and macOS 11, for 73 probe files plus
      82 constexpr-sampler files (`runall.sh`).
    - **Runtime:** on a Mac slot's GPU, every kernel from our Linux-built
      macOS 27 metallibs gives bit-identical output to Xcode's metallib
      for the same source: 19 of 19 runs (`run/diffrun.swift` with
      `run/specs/`).
  - **M3, vertex and fragment functions:**
    - `[[stage_in]]` structs are flattened into AIR's parameters.
    - The output struct is returned as a packed literal struct. spir64's
      sret is undone on `instnamer`'d IR.
    - Vertex outputs and fragment inputs get their `generated(...)` keys and
      interpolation.
    - Fragment outputs become render targets and depth;
      `[[color(n)]]` inputs, `discard_fragment`, VATT/VATY.
    - Version rules: `arg_unused` only from AIR 2.5, and
      `generated(__air_placeholder__)` on vertex inputs at AIR 2.1. The
      2.2 and 2.4 boundaries are unprobed.
  - **Results now:**
    - **Metadata:** all 20 compiling probes match on iOS 12, 16, 18, 27.2
      and macOS 11, at both the metadata and the metallib level (98 files),
      plus the sampler sweep and p21 at the metadata level.
    - **GPU:** 22 of 22 runs are bit-identical to Xcode's metallib. That
      includes three draws: stage_in attributes through a vertex
      descriptor, sampling with a sampler argument, and a constexpr
      sampler on a half texture.
  - **Front end, real-world code:**
    - The driver preprocesses first, as MSL does, with `__METAL_VERSION__`
      and the platform macro defined. Macros can then spell types and
      attributes (FFmpeg's `#define T float2` … `T(…)`), and the rewrite
      runs on the expanded text.
    - Unsuffixed floating literals become float, since MSL has no double.
    - MSL's other standard header names and `<simd/simd.h>` (its vector
      and matrix names) map onto our stdlib.
  - **Stdlib breadth:**
    - min/max/min3/max3, floor/fract, derivatives, `smoothstep`, `abs`;
    - matrix constructors (columns, diagonal, scalars, nested braces) and
      vector×matrix and matrix×matrix products;
    - uint texture reads, gradient sampling.
    - Names come from Xcode's output for the corpus; semantics from the
      MSL specification.
  - **Corpus (measurement only, not committed):** the rules_apple samples,
    Agora's iOS and macOS shaders, SDL's render and blit shaders, FFmpeg's
    yadif, and a SwiftPM fixture.
    - All of them now match Xcode at both the metadata and the metallib
      level, on all five targets.
    - Together with the probes, that's 143 files at both levels, plus 83
      at the metadata level.
    - The metallib orders functions by stage: kernels, then vertex, then
      fragment.
    - GPU: still 22 of 22 identical after these changes.
  - **Still to do:**
    - a Rust port;
    - runtime checks for argument buffers, framebuffer fetch and MRT;
    - unprobed intrinsic variants (int simd, threadgroup atomics, more
      math, other texture forms), from the next macOS VM batch;
    - the public corpus.
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
