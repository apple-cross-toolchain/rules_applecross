#include <metal_stdlib>
using namespace metal;
struct Params { float scale; uint count; };
kernel void buffers(device float *out [[buffer(0)]], const device float *in [[buffer(1)]],
                    constant Params &p [[buffer(2)]], uint i [[thread_position_in_grid]]) {
    if (i < p.count) out[i] = in[i] * p.scale;
}
