#include <metal_stdlib>
using namespace metal;
struct VIn {
    float3 pos [[attribute(0)]];
    half4 col [[attribute(3)]];
    uchar4 bytes [[attribute(2)]];
    int ix [[attribute(7)]];
    uint2 pair [[attribute(5)]];
    float unused [[attribute(1)]];
    short2 sh [[attribute(4)]];
    float4 w [[attribute(6)]];
};
struct VOut { float4 position [[position]]; half4 col; };
vertex VOut va(VIn in [[stage_in]]) {
    VOut o;
    o.position = float4(in.pos, 1) + float4(in.bytes) + float(in.ix) + float4(float2(in.pair), float2(in.sh)) + in.w;
    o.col = in.col;
    return o;
}
