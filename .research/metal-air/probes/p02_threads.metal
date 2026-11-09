#include <metal_stdlib>
using namespace metal;
kernel void threads(device uint4 *out [[buffer(0)]], uint2 gid [[thread_position_in_grid]],
                    uint2 tg [[threadgroup_position_in_grid]], uint2 tid [[thread_position_in_threadgroup]],
                    uint2 tpg [[threads_per_threadgroup]], uint2 grid [[threads_per_grid]]) {
    out[gid.y * grid.x + gid.x] = uint4(tg.x, tid.x, tpg.x, gid.y);
}
