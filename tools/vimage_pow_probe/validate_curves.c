#include <Accelerate/Accelerate.h>
#include <stdio.h>
#include "vpow.h"
// curveall CURVE LO_BITS HI_BITS: every float in the range, ours (fused a*x+b, x >= d) against vImage's symmetric piecewise curve; also its negative.
typedef struct { float g, a, b, e, c, f, d; } Curve;
static const Curve CV[3] = {
  {1.7999999523162842f, 1, 0, 0, 0.0625f, 0, 0.03125f},
  {2.4000000953674316f, 0.94786727428436279f, 0.052132699638605118f, 0, 0.077399380505084991f, 0, 0.040449999272823334f},
  {0.4166666567325592f, 1.1371190547943115f, 0, -0.054999999701976776f, 12.92f, 0, 0.003130804980173707f},
};
static float ours(const Curve *k, float x) {
  float s = x < 0 ? -1.0f : 1.0f, ax = fabsf(x), y;
  if (ax >= k->d) { float base = fmaf(k->a, ax, k->b); volatile float p = POW(base, k->g); y = p + k->e; } else { volatile float m = k->c * ax; y = m + k->f; }
  return s * y;
}
int main(int argc, char **argv) {
  int ci = atoi(argv[1]); uint32_t a = strtoul(argv[2], 0, 0), b = strtoul(argv[3], 0, 0); const Curve *k = &CV[ci];
  enum { CH = 1024 }; float in[CH], out[CH]; long n = 0, miss = 0; int shown = 0;
  float e[3] = {k->a, k->b, k->e}, l[2] = {k->c, k->f};
  for (uint32_t u = a; u < b; u += CH) {
    int cnt = b - u < CH ? b - u : CH;
    for (int j = 0; j < cnt; j++) { uint32_t w = u + j; memcpy(&in[j], &w, 4); }
    vImage_Buffer s = {in, 1, cnt, cnt * 4}, d = {out, 1, cnt, cnt * 4};
    vImageSymmetricPiecewiseGamma_PlanarF(&s, &d, e, k->g, l, k->d, kvImageNoFlags);
    for (int j = 0; j < cnt; j++) { n++; float o = ours(k, in[j]); if (o != out[j] && !(o == 0 && out[j] == 0)) { miss++; if (shown++ < 3) printf("  curve %d x=%.9g ours=%.9g vImage=%.9g\n", ci, in[j], o, out[j]); } }
    for (int j = 0; j < cnt; j++) in[j] = -in[j];
    vImageSymmetricPiecewiseGamma_PlanarF(&s, &d, e, k->g, l, k->d, kvImageNoFlags);
    for (int j = 0; j < cnt; j++) { n++; float o = ours(k, in[j]); if (o != out[j] && !(o == 0 && out[j] == 0)) { miss++; if (shown++ < 3) printf("  curve %d x=%.9g ours=%.9g vImage=%.9g\n", ci, in[j], o, out[j]); } }
  }
  printf("curve %d [0x%08x,0x%08x): %ld misses of %ld\n", ci, a, b, miss, n);
}
