#include <metal_stdlib>
using namespace metal;
kernel void mathf(device float4 *out [[buffer(0)]], const device float4 *in [[buffer(1)]], uint i [[thread_position_in_grid]]) {
    float4 v = in[i];
    out[i] = float4(sin(v.x) + cos(v.y), sqrt(fabs(v.z)), fma(v.x, v.y, v.z), clamp(v.w, 0.0f, 1.0f))
           + mix(v, normalize(v), 0.25f) * dot(v, v) + exp(v) - log(fabs(v) + 1.0f) + pow(fabs(v), 2.5f);
}
