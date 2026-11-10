#include <metal_stdlib>
using namespace metal;

constant bool use_square [[function_constant(0)]];

kernel void scale(device float *data [[buffer(0)]], constant float &factor [[buffer(1)]], uint i [[thread_position_in_grid]]) {
    data[i] = (use_square ? data[i] * data[i] : data[i]) * factor;
}

struct VertexIn {
    float2 position [[attribute(0)]];
    float2 uv [[attribute(1)]];
};

struct VertexOut {
    float4 position [[position]];
    float2 uv;
};

vertex VertexOut quad(VertexIn in [[stage_in]]) {
    VertexOut out;
    out.position = float4(in.position, 0, 1);
    out.uv = in.uv;
    return out;
}

fragment float4 textured(VertexOut in [[stage_in]], texture2d<float> image [[texture(0)]], sampler s [[sampler(0)]]) {
    return image.sample(s, in.uv);
}
