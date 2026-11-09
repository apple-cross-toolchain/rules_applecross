#include <metal_stdlib>
using namespace metal;
kernel void invert(texture2d<float, access::read> src [[texture(0)]], texture2d<float, access::write> dst [[texture(1)]], uint2 gid [[thread_position_in_grid]]) {
    if (gid.x >= src.get_width() || gid.y >= src.get_height()) return;
    float4 c = src.read(gid);
    dst.write(float4(1.0 - c.rgb, c.a), gid);
}
