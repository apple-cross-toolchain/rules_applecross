#include <metal_stdlib>
using namespace metal;
kernel void first(device float *a [[buffer(0)]], uint i [[thread_position_in_grid]]) { a[i] += 1; }
kernel void second(device float *a [[buffer(0)]], uint i [[thread_position_in_grid]]) { a[i] *= 3; }
static float helper(float x) { return x * x + 1; }
kernel void third(device float *a [[buffer(0)]], uint i [[thread_position_in_grid]]) { a[i] = helper(a[i]); }
