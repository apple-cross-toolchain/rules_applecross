#include <metal_stdlib>
using namespace metal;
struct Args { texture2d<float> tex [[id(0)]]; sampler smp [[id(1)]]; device float *data [[id(2)]]; constant uint &n [[id(3)]]; };
kernel void ab(constant Args &args [[buffer(0)]], device float *o [[buffer(1)]], uint i [[thread_position_in_grid]]) {
    o[i] = args.tex.sample(args.smp, float2(0.5)).x + args.data[i % args.n];
}
