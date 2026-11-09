#include <metal_stdlib>
using namespace metal;
kernel void count(device atomic_uint *counter [[buffer(0)]], const device float *in [[buffer(1)]], uint i [[thread_position_in_grid]]) {
    if (in[i] > 0.5f) atomic_fetch_add_explicit(counter, 1u, memory_order_relaxed);
}
