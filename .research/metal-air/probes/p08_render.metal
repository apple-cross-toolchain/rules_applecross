#include <metal_stdlib>
using namespace metal;
struct VIn { float2 pos [[attribute(0)]]; float2 uv [[attribute(1)]]; };
struct VOut { float4 position [[position]]; float2 uv; };
vertex VOut vmain(VIn in [[stage_in]], constant float2 &offset [[buffer(1)]]) {
    VOut o; o.position = float4(in.pos + offset, 0, 1); o.uv = in.uv; return o;
}
fragment float4 fmain(VOut in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler s [[sampler(0)]]) {
    return tex.sample(s, in.uv);
}
