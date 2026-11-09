#include <metal_stdlib>
using namespace metal;
constant int c_int [[function_constant(0)]];
constant uint c_uint [[function_constant(3)]];
constant half c_half [[function_constant(2)]];
constant float4 c_f4 [[function_constant(5)]];
constant short c_short [[function_constant(6)]];
constant int2 c_i2 [[function_constant(7)]];
constant bool c_opt [[function_constant(8)]];
constant bool c_unused [[function_constant(9)]];
constant bool has_opt = is_function_constant_defined(c_opt);
constant uchar c_uchar [[function_constant(10)]];
constant float c_f [[function_constant(11)]];
kernel void fc2(device float4 *a [[buffer(0)]], uint i [[thread_position_in_grid]]) {
    float4 v = a[i] * float(c_int) + float(c_uint) + float(c_half) + c_f4 + float(c_short) + float4(float2(c_i2), 0, 0) + float(c_uchar);
    if (has_opt && c_opt) v += 1;
    a[i] = v;
}
kernel void fc3(device float *a [[buffer(0)]], uint i [[thread_position_in_grid]]) {
    a[i] = a[i] + c_f;
}
