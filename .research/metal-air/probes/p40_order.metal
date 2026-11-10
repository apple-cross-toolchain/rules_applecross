#include <metal_stdlib>
using namespace metal;
struct Params { float scale; uint count; };
// Evaluation and operand order: one shape per kernel.
kernel void o01(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant float &s [[buffer(2)]], uint i [[thread_position_in_grid]]) { out[i] = in[i] * s; }
kernel void o02(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], const device float *b [[buffer(2)]], uint i [[thread_position_in_grid]]) { out[i] = a[i] + b[i]; }
kernel void o03(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], const device float *b [[buffer(2)]], const device float *c [[buffer(3)]], uint i [[thread_position_in_grid]]) { out[i] = a[i] * b[i] + c[i]; }
kernel void o04(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], const device float *b [[buffer(2)]], uint i [[thread_position_in_grid]]) { float x = a[i]; float y = b[i]; out[i] = y - x; }
kernel void o05(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], const device float *b [[buffer(2)]], uint i [[thread_position_in_grid]]) { out[i] = b[i] - a[i]; }
kernel void o06(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant Params &p [[buffer(2)]], uint i [[thread_position_in_grid]]) { out[i] = in[i] * p.scale; }
kernel void o07(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], uint i [[thread_position_in_grid]]) { out[i] = in[i]; }
kernel void o08(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant Params &p [[buffer(2)]], uint i [[thread_position_in_grid]]) { out[i] = p.scale * in[i]; }
kernel void o09(device float *out [[buffer(0)]], device float *out2 [[buffer(1)]], const device float *a [[buffer(2)]], const device float *b [[buffer(3)]], uint i [[thread_position_in_grid]]) { out[i] = a[i]; out2[i] = b[i]; }
kernel void o10(device float4 *out [[buffer(0)]], const device float *a [[buffer(1)]], const device float *b [[buffer(2)]], uint i [[thread_position_in_grid]]) { out[i] = float4(a[i], b[i], 0, 1); }
kernel void o11(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], const device float *b [[buffer(2)]], const device float *c [[buffer(3)]], const device float *d [[buffer(4)]], uint i [[thread_position_in_grid]]) { out[i] = a[i] + b[i] + c[i] + d[i]; }
kernel void o12(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], uint i [[thread_position_in_grid]]) { out[i] = a[i] * 2.0 + 1.0; }
kernel void o13(device float4 *out [[buffer(0)]], const device float4 *v [[buffer(1)]], constant float &s1 [[buffer(2)]], constant float &s2 [[buffer(3)]], constant float &s3 [[buffer(4)]], uint i [[thread_position_in_grid]]) { out[i] = v[i] * s1 + s2 + s3; }
kernel void o14(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], uint i [[thread_position_in_grid]]) { out[i] = out[i] + a[i]; }
kernel void o15(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], uint i [[thread_position_in_grid]]) { out[i] += a[i]; }
kernel void o16(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], const device float *b [[buffer(2)]], uint i [[thread_position_in_grid]]) { out[i] = a[i] * b[i]; }
kernel void o17(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], const device float *b [[buffer(2)]], uint i [[thread_position_in_grid]]) { out[i] = b[i] * a[i]; }
kernel void o18(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], uint i [[thread_position_in_grid]]) { out[i] = a[i] * a[i + 1]; }
kernel void o19(device int *out [[buffer(0)]], const device int *a [[buffer(1)]], const device int *b [[buffer(2)]], uint i [[thread_position_in_grid]]) { out[i] = a[i] + b[i]; }
kernel void o20(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], const device float *b [[buffer(2)]], uint i [[thread_position_in_grid]]) { float t = a[i]; out[i] = t * b[i] + t; }
