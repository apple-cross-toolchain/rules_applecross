#include <metal_stdlib>
using namespace metal;
struct In { float4 pos [[position]]; float2 uv; float k [[flat]]; };
struct Out { float4 c0 [[color(0)]]; half4 c1 [[color(1)]]; float depth [[depth(any)]]; };
fragment Out fx(In in [[stage_in]], bool ff [[front_facing]], uint sid [[sample_id]], float4 prev [[color(0)]], constant float &t [[buffer(0)]]) {
    if (in.k < 0) discard_fragment();
    Out o; o.c0 = prev + float4(in.uv, t, ff ? 1 : 0); o.c1 = half4(float(sid)); o.depth = in.pos.z; return o;
}
fragment float4 fpt(float2 pc [[point_coord]], float4 pos [[position]]) { return float4(pc, pos.xy); }
