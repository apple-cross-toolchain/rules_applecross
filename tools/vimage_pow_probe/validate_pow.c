#include <Accelerate/Accelerate.h>
#include <stdio.h>
#include "vpow.h"
static float vp(float b, float g) { float e[3] = {1, 0, 0}, l[2] = {0, 0}, o; vImage_Buffer s = {&b, 1, 1, 4}, d = {&o, 1, 1, 4}; vImagePiecewiseGamma_PlanarF(&s, &d, e, g, l, 0, kvImageNoFlags); return o; }
int main(void) {
  srand48(1);
  float gs[] = {1.8f, 2.4000000953674316f, 0.4166666567325592f, 1.0f, 0.5f, 3.0f, 0.1f};
  for (int gi = -1; gi < 7; gi++) {
    long n = 0, m = 0; int shown = 0;
    for (int i = 0; i < 2000000; i++) {
      float b = (float)(drand48() * (drand48() < 0.5 ? 1.0 : 1.5)); if (drand48() < 0.1) b = (float)ldexp(drand48(), -(int)(drand48() * 40));
      float g = gi < 0 ? (float)(drand48() * 4) : gs[gi];
      float a = POW(b, g), v = vp(b, g); n++;
      if (a == v || (isnan(a) && isnan(v))) m++; else if (shown++ < 3) printf("   b=%.9g g=%.9g ours=%.9g vImage=%.9g\n", b, g, a, v);
    }
    printf("g=%s: %ld/%ld (%.6f)\n", gi < 0 ? "random" : "fixed", m, n, m / (double)n); fflush(stdout);
  }
}
