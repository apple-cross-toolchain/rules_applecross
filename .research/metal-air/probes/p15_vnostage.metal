#include <metal_stdlib>
using namespace metal;
struct V { packed_float3 p; float s; };
struct VOut { float4 position [[position]]; float size [[point_size]]; float c [[flat]]; float d [[center_no_perspective]]; };
vertex VOut vb(const device V *vs [[buffer(0)]], constant float4x4 &mvp [[buffer(1)]], uint vid [[vertex_id]], uint iid [[instance_id]], uint bv [[base_vertex]], uint bi [[base_instance]]) {
    VOut o; o.position = mvp * float4(float3(vs[vid].p), 1); o.size = vs[vid].s; o.c = float(iid + bv); o.d = float(bi); return o;
}
