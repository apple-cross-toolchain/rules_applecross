# What Xcode emits: metallib and AIR metadata (M0, 2026-11-09)

Learned black-box from Xcode 27.2's Metal toolchain (metalfe 32023.922) on
a leased macOS VM. The inputs were our own probe shaders (`probes/`; the
samplers come from `samplers.sh`), plus a local measurement corpus of
public shader projects, which is not committed. The flags were rules_apple's:
`metal -c -target air64-apple-<os><min>[-simulator] -ffast-math`, then
`metallib`.

## The metallib container

`rebuild.py` takes every reference metallib apart into its modules and
metadata blocks, then rebuilds it with `mtlb.py`. All 358 reference files
come out byte for byte: 12 targets, single- and multi-function, vertex and
fragment, function constants, and the files linked from many `.air`s.

- **Header:** `MTLB`, then six u16s, then nine u64s. The u64s are: the
  total size; the offset and length of the function list (without its
  count); the public metadata; the private metadata; and the modules.
- **Function list:** a u32 count, then one entry per function.
  - Each entry is a u32 size (including itself), then tags: NAME, TYPE,
    HASH, MDSZ, OFFT, VERS, ENDT.
  - Each tag is four letters and a u16 length, except ENDT, which has no
    length.
  - TYPE: 0 vertex, 1 fragment, 2 kernel.
  - HASH: SHA-256 of the function's module.
  - MDSZ: the module's size.
  - OFFT: three u64s, the offsets of the function's public metadata,
    private metadata and module within their sections.
  - VERS: AIR major and minor, then language major and minor, as u16s.
- **After the list:** one `ENDT`, then the public metadata blocks, the
  private blocks and the modules, one of each per function. Each block is
  a u32 size (including itself), its tags, then `ENDT`.
- **Modules:** every function gets its own module. It is bitcode in the
  `0B17C0DE` wrapper, padded with zeros to 16 bytes. `metallib` splits a
  multi-function `.air` this way, so a three-kernel file holds three
  modules.

### Header u16s

| u16 | Meaning | Values seen |
| --- | --- | --- |
| 0 | Platform family | 1 iOS, tvOS, xrOS; 0x8001 macOS |
| 1 | Always 2 | |
| 2 | File version | 3 for iOS 12; 5 for macOS 11; 7 for iOS 16 and 17, macOS 14, tvOS 17; 8 for iOS 18, xrOS 2; 9 for 26 and 27 |
| 3 | Platform | `0x80\|p` in the high byte (0x82 iOS, 0x81 macOS, 0x83 tvOS, 0x87 iOS Simulator, 0x8B xrOS); 0 for the oldest targets |
| 4, 5 | Deployment target | OS major and minor; 0, 0 for the oldest targets |

### Public metadata

Private blocks were always empty (`ENDT`). Public blocks are empty except
for these:

- **VATT** (vertex functions with `[[stage_in]]`): a u16 count, then for
  each attribute, in declaration order:
  - the field name, NUL-terminated;
  - a u16 holding the attribute index, with bit 15 set when the function
    uses the attribute.
- **VATY:** a u16 count, then one byte per attribute: its `MTLDataType`
  (the public enum: float 3, float2 4, float3 5, float4 6, half4 19, int
  29, uint 33, uint2 34, short2 38, uchar4 52, bool 53).
- **CNST** (functions that use function constants): a u16 count, then for
  each constant the function references, in index order:
  - the name, NUL-terminated;
  - its `MTLDataType` byte;
  - a u16 index;
  - a byte that is 1 when the constant is required, and 0 when the
    function tests it with `is_function_constant_defined`.

  Constants the function doesn't reference are left out.

## The module: target, versions, flags

| Target | Triple | AIR | Language |
| --- | --- | --- | --- |
| iOS 11 | air64_v20 | 2.0 | Metal 2.0 |
| iOS 12, macOS 10.14 | air64_v21 | 2.1 | 2.1 |
| iOS 13, macOS 10.15 | air64_v22 | 2.2 | 2.2 |
| iOS 14, macOS 11 | air64_v23 | 2.3 | 2.3 |
| iOS 15, macOS 12 | air64_v24 | 2.4 | 2.4 |
| iOS 16, macOS 13 | air64_v25 | 2.5 | 3.0 |
| iOS 17, macOS 14, tvOS 17 | air64_v26 | 2.6 | 3.1 |
| iOS 18, xrOS 2 | air64_v27 | 2.7 | 3.2 |
| iOS 26 | air64_v28 | 2.8 | 4.0 |
| iOS 27.2, macOS 27 | air64_v29 | 2.9 | 4.1 |

- The triple's OS part is the target as given, e.g.
  `air64_v29-apple-ios27.2.0`.
- Module flags: `SDK Version` (27.2 for every target with this Xcode),
  `wchar_size` 4 and `frame-pointer` 2.
- From AIR 2.3 (iOS 14, macOS 11) the module also carries the `air.max_*` limits:
  31 device, constant and threadgroup buffers; 128 textures; 8
  read-write textures; 16 samplers.
- `!air.compile_options`: `denorms_disable`, `fast_math_enable`,
  `framebuffer_fetch_enable`.
- `!air.version`, `!air.language_version` and `!air.source_file_name`.
- The datalayout is the same for every target.

## Argument descriptors

`!air.kernel`, `!air.vertex` and `!air.fragment` each list
`{function, outputs, arguments[, extras]}`. The extras seen so far are
`air.max_work_group_size` (from `max_total_threads_per_threadgroup`).

Each argument is `{i32 index, kind, ...}`, ending with
`air.arg_type_name` and `air.arg_name`. When the optimised function no
longer uses the argument, `air.arg_unused` follows.

- **Buffers:**
  - `air.buffer`, then `air.buffer_size` N, but only for `constant T &`
    references.
  - Then `air.location_index` slot 1, then `air.read` (const or constant)
    or `air.read_write`.
  - Then `air.address_space` (1 device, 2 constant, 3 threadgroup), from
    AIR 2.5 on.
  - Then `air.struct_type_info` for struct pointees, then
    `air.arg_type_size` and `air.arg_type_align_size`.
  - Threadgroup arguments are `air.buffer` too, with address space 3 and
    their `[[threadgroup(n)]]` slot.
- **Argument buffers:** a struct with `[[id(n)]]` members is an
  `air.indirect_buffer`, with the same fields as a buffer. Its struct info
  pairs each member with an `air.indirect_argument` descriptor: a buffer, a
  texture, a sampler, or an `air.indirect_constant` for a plain value.
  Arrays put their count in the second `location_index` operand.
- **Struct type info:** a flat list of (offset, element size, array count
  or 0, type name, field name) for each field. A struct-typed field is
  preceded by `air.struct_type_info` and its own list; an argument-buffer
  member is followed by `air.indirect_argument`. Atomics show as
  `metal::_atomic` with one field.
- **Textures:** `air.texture`, `location_index` slot and count, then the
  access (`air.sample`, `air.read`, `air.write` or `air.read_write`).
  The type name spells the access out, e.g. `texture2d<float, sample>` or
  `array<texture2d<float, sample>, 3>`.
- **Samplers:** `air.sampler`, `location_index`.
- **Built-ins:** the attribute name with `air.` in front:
  - `thread_position_in_grid`, `threads_per_grid`,
    `thread_index_in_threadgroup` and the other thread and threadgroup
    positions;
  - `thread_index_in_simdgroup`, `simdgroup_index_in_threadgroup`,
    `threads_per_simdgroup`;
  - `vertex_id`, `instance_id`, `base_vertex`, `base_instance`;
  - `front_facing`, `sample_id`, `point_coord`.
- **Vertex input** (`[[stage_in]]`): one `air.vertex_input` per field, in
  declaration order, with `location_index` (attribute, 1).
- **Vertex output:** `air.position`, `air.point_size`, and
  `air.vertex_output` with a `generated(<len><name><mangled type>)` key.
  For example, `generated(2uvDv2_f)` is `uv`, a float2. Only AIR 2.0–2.1
  add interpolation qualifiers here.
- **Fragment input:** `air.fragment_input` with the same `generated` key,
  then `air.center`/`air.centroid`/`air.sample` and
  `air.perspective`/`air.no_perspective`, or `air.flat`. `[[position]]` is
  `air.position` with `air.center, air.no_perspective`.
- **Fragment output:** `air.render_target` (index, 0) for each `color(n)`,
  and `air.depth` with `air.depth_qualifier` (`air.any` and the others).
  `[[color(n)]]` as an input is `air.render_target` too (framebuffer
  fetch).
- **Rules the front end enforces:** thread-position arguments must all be
  scalars or all vectors of one width.

## Function constants

- Each constant is a pair of globals:
  - an internal `@_ZL<name>` (`undef`);
  - a `@_Z<name>.MTL_FC_INIT_<index>_<mangled type>` in section
    `air.fc_initializer`.
- Each gets an `!air.function_constants` entry `{initializer, type name,
  name, index}`.
- A static initializer (`air.static_init`) copies the constants into
  their uses. `is_function_constant_defined` reads the initializer
  directly.

## Constexpr samplers

A constexpr sampler is an internal constant `@__air_sampler_state` in
address space 2, listed in `!air.sampler_states`.

- **Form by target:**
  - Up to AIR 2.6 (iOS 17, macOS 14, tvOS 17): a single `i64`, with bit
    63 set.
  - From AIR 2.7 (iOS 18, macOS 27, xrOS 2): `[2 x i64]`, with bit 63
    clear and the second word 0.
- **Bits in the first word:**

  | Bits | Field | Values |
  | --- | --- | --- |
  | 0–2, 3–5, 6–8 | s, t, r address | clamp_to_zero/clamp_to_border 0, clamp_to_edge 1, repeat 2, mirrored_repeat 3 |
  | 9–10, 11–12 | mag, min filter | nearest 0, linear 1 |
  | 13–14 | mip filter | none 0, nearest 1, linear 2 |
  | 15 | coord | normalized 0, pixel 1 |
  | 16–19 | compare_func | the MSL enum order: none 0, less 1, … greater_equal 4, … always 7, never 8 (the default) |
  | 20–23 | max_anisotropy | value − 1 |
  | 24–39 | lod_clamp min | IEEE half |
  | 40–55 | lod_clamp max | IEEE half (default 65504) |
  | 56–57 | border_color | transparent_black 0, opaque_black 1, opaque_white 2 |
