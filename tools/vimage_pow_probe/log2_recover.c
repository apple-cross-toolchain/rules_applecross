#include <Accelerate/Accelerate.h>
#include <stdio.h>
#include "vpow.h"
// lrec2 LO HI OUT: vImage's log2 for every float x in [LO, HI), recovered with the exact exp2 (unique at two k's).
static float vp(float b, float g) { float e[3] = {1, 0, 0}, l[2] = {0, 0}, o; vImage_Buffer s = {&b, 1, 1, 4}, d = {&o, 1, 1, 4}; vImagePiecewiseGamma_PlanarF(&s, &d, e, g, l, 0, kvImageNoFlags); return o; }
static int recover(float x, int k, float *L) {
  float t0 = ldexpf(log2f(x), k), v = vp(x, ldexpf(1, k)); int found = 0; float t = t0;
  for (int s = 0; s < 200; s++) t = nextafterf(t, -INFINITY);
  for (int s = 0; s < 400; s++, t = nextafterf(t, INFINITY)) if (E2(t) == v) { if (found) return 2; *L = ldexpf(t, -k); found = 1; }
  return found;
}
int main(int argc, char **argv) {
  float lo = atof(argv[1]), hi = atof(argv[2]); uint32_t a, b; memcpy(&a, &lo, 4); memcpy(&b, &hi, 4);
  FILE *o = fopen(argv[3], "wb"); long st[3] = {0};
  for (uint32_t u = a; u < b; u++) {
    float x; memcpy(&x, &u, 4); if (log2f(x) == 0) continue;
    int k = 1 - ilogbf(log2f(x)); float L1, L2; int s = 0;
    if (recover(x, k, &L1) == 1 && recover(x, k + 1, &L2) == 1 && L1 == L2) s = 1; else if (recover(x, k + 2, &L1) == 1 && recover(x, k + 3, &L2) == 1 && L1 == L2) s = 1;
    st[s]++; float rec[3] = {x, s ? L1 : 0, (float)s}; fwrite(rec, 4, 3, o);
  }
  fclose(o); printf("%s: confirmed %ld, unresolved %ld\n", argv[3], st[1], st[0]);
}
