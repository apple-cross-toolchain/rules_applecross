#include <metal_stdlib>
using namespace metal;
struct Params { float scale; uint count; };
// Order around branches and loops: one shape per kernel.
kernel void c01(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant float &s [[buffer(2)]], constant uint &n [[buffer(3)]], uint i [[thread_position_in_grid]]) { if (i < n) out[i] = in[i] * s; }
kernel void c02(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant Params &p [[buffer(2)]], uint i [[thread_position_in_grid]]) { if (i < p.count) out[i] = in[i] * p.scale; }
kernel void c03(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant uint &n [[buffer(2)]], uint i [[thread_position_in_grid]]) { if (i < n) out[i] = in[i]; }
kernel void c04(device float *out [[buffer(0)]], const device float *a [[buffer(1)]], const device float *b [[buffer(2)]], constant uint &n [[buffer(3)]], uint i [[thread_position_in_grid]]) { if (i < n) out[i] = a[i] + b[i]; }
kernel void c05(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant float &s [[buffer(2)]], constant uint &n [[buffer(3)]], uint i [[thread_position_in_grid]]) { if (i >= n) return; out[i] = in[i] * s; }
kernel void c06(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant uint &n [[buffer(2)]], uint i [[thread_position_in_grid]]) { out[i] = (i < n) ? in[i] : 0.0f; }
kernel void c07(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], uint i [[thread_position_in_grid]]) { if (in[i] > 0) out[i] = 1; }
kernel void c08(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant uint &n [[buffer(2)]], uint i [[thread_position_in_grid]]) { float s = 0; for (uint k = 0; k < n; k++) s += in[k]; out[i] = s; }
kernel void c09(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], uint i [[thread_position_in_grid]]) { for (int k = 0; k < 4; k++) out[i * 4 + k] = in[i * 4 + k] * 2; }
kernel void c10(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant Params &p [[buffer(2)]], uint i [[thread_position_in_grid]]) { if (i < p.count) out[i] = p.scale * in[i]; }
kernel void c11(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant float &s [[buffer(2)]], constant uint &n [[buffer(3)]], uint i [[thread_position_in_grid]]) { if (i < n) { float v = in[i]; out[i] = v * s; } }
kernel void c12(device float *out [[buffer(0)]], constant float &s [[buffer(1)]], constant uint &n [[buffer(2)]], uint i [[thread_position_in_grid]]) { if (i < n) out[i] = s; }
kernel void c13(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], constant uint &n [[buffer(2)]], uint i [[thread_position_in_grid]]) { if (i < n) out[i] = in[i] * 3.0f; }
kernel void c14(device float *out [[buffer(0)]], const device float *in [[buffer(1)]], device float *out2 [[buffer(2)]], constant uint &n [[buffer(3)]], uint i [[thread_position_in_grid]]) { if (i < n) { out[i] = in[i]; out2[i] = in[i] * 2; } }
