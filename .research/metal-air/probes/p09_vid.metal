#include <metal_stdlib>
using namespace metal;
struct VOut { float4 position [[position]]; float4 color; };
vertex VOut tri(uint vid [[vertex_id]], const device float4 *pos [[buffer(0)]], const device float4 *col [[buffer(1)]]) {
    VOut o; o.position = pos[vid]; o.color = col[vid]; return o;
}
fragment float4 solid(VOut in [[stage_in]]) { return in.color; }
constexpr sampler linear_clamp(coord::normalized, address::clamp_to_edge, filter::linear);
fragment half4 sampled(VOut in [[stage_in]], texture2d<half> t [[texture(0)]]) { return t.sample(linear_clamp, in.color.xy); }
