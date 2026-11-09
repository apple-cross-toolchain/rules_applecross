#include <metal_stdlib>
using namespace metal;
struct Inner { float2 a; uchar b; };
struct S { float arr[4]; Inner in; packed_float3 p; half2 h; float2x2 m; Inner ins[2]; int3 i3; };
kernel void st(constant S &s [[buffer(0)]], device S *d [[buffer(1)]], device float *o [[buffer(2)]], uint i [[thread_position_in_grid]]) {
    o[i] = s.arr[i % 4] + s.in.a.x + float(s.in.b) + s.p[0] + float(s.h.x) + s.m[0][0] + s.ins[1].a.y + float(s.i3.z) + d[i].arr[0];
}
struct AB { device float *p [[id(0)]]; array<texture2d<float>, 3> ts [[id(1)]]; float k [[id(4)]]; constant S *cs [[id(5)]]; };
kernel void ab2(device AB &ab [[buffer(0)]], device float *o [[buffer(1)]], uint i [[thread_position_in_grid]]) {
    o[i] = ab.p[i] + ab.ts[1].read(uint2(0)).x + ab.k + ab.cs->arr[0];
}
