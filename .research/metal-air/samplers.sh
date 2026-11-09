#!/bin/bash
# Write one kernel per constexpr sampler, each changing one parameter, to
# learn the sampler-state encoding (FORMAT.md, "Constexpr samplers").
set -eu
mkdir -p samplers; i=0
while IFS= read -r spec; do
  printf '#include <metal_stdlib>\nusing namespace metal;\nconstexpr sampler cs(%s);\nkernel void k(texture2d<float> t [[texture(0)]], device float4 *o [[buffer(0)]], uint i [[thread_position_in_grid]]) { o[i] = t.sample(cs, float2(0.5)); }\n' "$spec" > "samplers/$(printf 's%02d' $i).metal"
  i=$((i+1))
done <<'SPECS'
coord::normalized
coord::pixel
address::clamp_to_edge
address::repeat
address::mirrored_repeat
address::clamp_to_zero
address::clamp_to_border
s_address::repeat
t_address::repeat
r_address::repeat
filter::nearest
filter::linear
mag_filter::linear
min_filter::linear
mip_filter::none
mip_filter::nearest
mip_filter::linear
compare_func::never
compare_func::less
compare_func::always
compare_func::greater_equal
max_anisotropy(2)
max_anisotropy(16)
lod_clamp(0.0f, 1.0f)
lod_clamp(2.5f, 8.0f)
address::clamp_to_border, border_color::opaque_black
address::clamp_to_border, border_color::opaque_white
address::clamp_to_border, border_color::transparent_black
s_address::mirrored_repeat, t_address::clamp_to_zero, r_address::clamp_to_border
SPECS
