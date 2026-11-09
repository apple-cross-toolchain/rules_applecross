#include <metal_stdlib>
using namespace metal;
constant bool use_square [[function_constant(0)]];
constant float offset_c [[function_constant(1)]];
kernel void fc(device float *a [[buffer(0)]], uint i [[thread_position_in_grid]]) {
    a[i] = (use_square ? a[i] * a[i] : a[i]) + offset_c;
}
