// The simd vector and matrix names shared C/MSL headers use, as the MSL
// types they correspond to (Apple's public simd documentation).
#pragma once
#include <metal_stdlib>
#define __MSL_SIMD_NAMES(T) \
  typedef T##2 vector_##T##2; typedef T##3 vector_##T##3; typedef T##4 vector_##T##4; \
  typedef T##2 simd_##T##2; typedef T##3 simd_##T##3; typedef T##4 simd_##T##4; typedef T simd_##T;
__MSL_SIMD_NAMES(float) __MSL_SIMD_NAMES(half) __MSL_SIMD_NAMES(int) __MSL_SIMD_NAMES(uint)
__MSL_SIMD_NAMES(short) __MSL_SIMD_NAMES(ushort) __MSL_SIMD_NAMES(char) __MSL_SIMD_NAMES(uchar)
typedef metal::float2x2 matrix_float2x2; typedef metal::float3x3 matrix_float3x3; typedef metal::float4x4 matrix_float4x4;
typedef metal::float2x2 simd_float2x2; typedef metal::float3x3 simd_float3x3; typedef metal::float4x4 simd_float4x4;
typedef metal::float4x3 matrix_float4x3; typedef metal::float3x4 matrix_float3x4;
