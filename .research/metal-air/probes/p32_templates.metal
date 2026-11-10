#include <metal_stdlib>
using namespace metal;
enum order { ASC, DESC };
typedef void (sort_t)(device float *p [[buffer(0)]], uint i [[thread_position_in_grid]]);
template <order O>
kernel void ksort(device float *p [[buffer(0)]], uint i [[thread_position_in_grid]]) { p[i] = O == ASC ? p[i] : -p[i]; }
template [[host_name("ksort_desc")]] kernel sort_t ksort<DESC>;
template [[host_name("ksort_asc")]] kernel sort_t ksort<ASC>;
template <typename T>
kernel void kscale(device T *p [[buffer(0)]], constant float &s [[buffer(1)]], uint i [[thread_position_in_grid]]) { p[i] = T(float(p[i]) * s); }
typedef decltype(kscale<float>) kscale_t;
template [[host_name("kscale_f32")]] kernel kscale_t kscale<float>;
