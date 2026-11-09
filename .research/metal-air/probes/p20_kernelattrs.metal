#include <metal_stdlib>
using namespace metal;
[[max_total_threads_per_threadgroup(256)]]
kernel void ka(device atomic_uint *c [[buffer(0)]], device const int *d [[buffer(1)]], uint i [[thread_position_in_grid]]) {
    if (d[i] > 0) atomic_fetch_add_explicit(c, 1, memory_order_relaxed);
}
kernel void kb(device float *a [[buffer(0)]], constant float *b [[buffer(1)]], device half *h [[buffer(2)]], uint2 i [[thread_position_in_grid]], uint2 n [[threads_per_grid]]) {
    a[i.y * n.x + i.x] = b[i.x] + float(h[i.y]);
}
