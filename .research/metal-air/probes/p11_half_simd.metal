#include <metal_stdlib>
using namespace metal;
kernel void hs(device half4 *a [[buffer(0)]], device float *sums [[buffer(1)]], uint i [[thread_position_in_grid]], uint lane [[thread_index_in_simdgroup]]) {
    half4 h = a[i] * half(0.5h) + half4(1);
    a[i] = h;
    float s = simd_sum(float(h.x));
    if (lane == 0) sums[i / 32] = s;
}
