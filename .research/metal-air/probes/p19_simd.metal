#include <metal_stdlib>
using namespace metal;
kernel void sd(device float *a [[buffer(0)]], threadgroup float *tg [[threadgroup(0)]], uint i [[thread_position_in_grid]], uint l [[thread_index_in_threadgroup]],
               uint lane [[thread_index_in_simdgroup]], uint sg [[simdgroup_index_in_threadgroup]], uint w [[threads_per_simdgroup]], uint tpg [[threads_per_threadgroup]],
               uint gid [[threadgroup_position_in_grid]], uint q [[thread_index_in_quadgroup]]) {
    float v = simd_sum(a[i]) + simd_shuffle_down(a[i], 1) + quad_broadcast(a[i], 0) + simd_prefix_exclusive_sum(a[i]);
    if (lane == 0) tg[sg] = v;
    threadgroup_barrier(mem_flags::mem_threadgroup);
    simdgroup_barrier(mem_flags::mem_none);
    a[i] = tg[l % w] + float(tpg + gid + q);
}
