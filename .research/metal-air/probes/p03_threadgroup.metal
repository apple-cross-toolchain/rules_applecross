#include <metal_stdlib>
using namespace metal;
kernel void tgsum(device float *out [[buffer(0)]], const device float *in [[buffer(1)]],
                  threadgroup float *scratch [[threadgroup(0)]], uint tid [[thread_index_in_threadgroup]],
                  uint gid [[thread_position_in_grid]]) {
    scratch[tid] = in[gid];
    threadgroup_barrier(mem_flags::mem_threadgroup);
    if (tid == 0) { float s = 0; for (uint k = 0; k < 64; k++) s += scratch[k]; out[gid / 64] = s; }
}
