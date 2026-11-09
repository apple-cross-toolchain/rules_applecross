#include <metal_stdlib>
using namespace metal;
constexpr sampler cs(coord::normalized, address::repeat, filter::linear, mip_filter::linear);
kernel void tx(texture1d<float> t1 [[texture(0)]], texture2d_array<half> ta [[texture(1)]], texture3d<float, access::read> t3 [[texture(2)]],
               texturecube<float> tc [[texture(3)]], depth2d<float> dt [[texture(4)]], texture_buffer<float, access::write> tb [[texture(5)]],
               texture2d<float, access::read_write> rw [[texture(6)]], sampler s [[sampler(0)]], device float4 *o [[buffer(0)]], uint2 g [[thread_position_in_grid]]) {
    float4 v = t1.sample(s, 0.5) + float4(ta.sample(cs, float2(0.5), 1)) + t3.read(uint3(g, 0)) + tc.sample(s, float3(1, 0, 0)) + dt.sample(s, float2(0.5));
    tb.write(v, g.x); rw.write(rw.read(g) + v, g); o[g.x] = v;
}
