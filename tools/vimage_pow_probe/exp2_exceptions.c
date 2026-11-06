#include <Accelerate/Accelerate.h>
#include <stdio.h>
#include <math.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
// eall LO_BITS HI_BITS OUT: every float f in [LO, HI) (as bits): vImage's 2^f against the exp2 model; writes the misses (f bits, vImage bits).
static const uint32_t ci[7] = {0x3f800000, 0x3f317218, 0x3e75fe20, 0x3d634f9e, 0x3c1e2639, 0x3aa649f5, 0x395a8346};
int main(int argc, char **argv) {
  uint32_t a = strtoul(argv[1], 0, 0), b = strtoul(argv[2], 0, 0); float c[7]; memcpy(c, ci, 28);
  FILE *o = fopen(argv[3], "wb"); long miss = 0, n = 0;
  enum { CH = 4096 }; float in[CH], out[CH];
  for (uint32_t u = a; u < b; ) {
    // vImage takes one gamma per call: base 2, gamma f, one pixel per call.
    float f; memcpy(&f, &u, 4);
    float base = 2.0f, y; float e[3] = {1, 0, 0}, l[2] = {0, 0}; vImage_Buffer s = {&base, 1, 1, 4}, d = {&y, 1, 1, 4};
    vImagePiecewiseGamma_PlanarF(&s, &d, e, f, l, 0, kvImageNoFlags);
    float x = f, q = c[6]; for (int k = 5; k >= 1; k--) q = fmaf(q, x, c[k]); volatile float pr = q * x; float p = pr + 1.0f;
    n++; if (p != y) { miss++; uint32_t rec[2]; memcpy(&rec[0], &f, 4); memcpy(&rec[1], &y, 4); fwrite(rec, 4, 2, o); }
    u++;
  }
  fclose(o); printf("%s: %ld of %ld differ\n", argv[3], miss, n);
}
