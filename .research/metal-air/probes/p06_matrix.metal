#include <metal_stdlib>
using namespace metal;
kernel void matmul(device float4 *out [[buffer(0)]], constant float4x4 &m [[buffer(1)]], const device float4 *in [[buffer(2)]], uint i [[thread_position_in_grid]]) {
    out[i] = (m * in[i]).wzyx + float4(in[i].xy, 1, 0);
}
