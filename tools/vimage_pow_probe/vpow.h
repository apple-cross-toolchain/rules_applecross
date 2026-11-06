// vpow.h: vImage's float pow model under study: exp2 (model + exceptions), log2 (two models + exceptions to come).
#include <stdint.h>
#include <string.h>
#include <math.h>
#include <stdlib.h>
#include "eexc.h"
static const uint32_t EC[7] = {0x3f800000, 0x3f317218, 0x3e75fe20, 0x3d634f9e, 0x3c1e2639, 0x3aa649f5, 0x395a8346};
static int cmpu(const void *a, const void *b) { uint32_t x = *(const uint32_t *)a, y = *(const uint32_t *)b; return x < y ? -1 : x > y; }
static float P2(float x) {   // 2^x for x in [0, 1)
  uint32_t u; memcpy(&u, &x, 4);
  const uint32_t *hit = bsearch(&u, EXC_X, EXC_N, 4, cmpu);
  if (hit) { float y; memcpy(&y, &EXC_Y[hit - EXC_X], 4); return y; }
  float c[7]; memcpy(c, EC, 28); float q = c[6]; for (int k = 5; k >= 1; k--) q = fmaf(q, x, c[k]); volatile float pr = q * x; return pr + 1.0f;
}
static float E2(float t) { float i = floorf(t), x = t - i; return ldexpf(P2(x), (int)i); }
#include "lexc.h"
static float LPC[7], LNC[7]; static int lc_loaded;
static void lc_load(void) { if (lc_loaded) return; FILE *f = fopen("lfin_pos.bin", "rb"); fread(LPC, 4, 7, f); fclose(f); f = fopen("lfin_neg.bin", "rb"); fread(LNC, 4, 7, f); fclose(f); lc_loaded = 1; }
static float LP(const float *c, float r) { volatile float x2v = r * r; float x2 = x2v; float ev = fmaf(fmaf(fmaf(c[6], x2, c[4]), x2, c[2]), x2, c[0]); float od = fmaf(fmaf(c[5], x2, c[3]), x2, c[1]); volatile float L = r * fmaf(od, r, ev); return L; }
static float LM(float m) {  // log2 for m in [sqrt2/2, sqrt2)
  lc_load(); uint32_t u; memcpy(&u, &m, 4);
  const uint32_t *X = m < 1.0f ? LNEG_X : LPOS_X, *Y = m < 1.0f ? LNEG_Y : LPOS_Y; uint32_t N = m < 1.0f ? LNEG_N : LPOS_N;
  const uint32_t *hit = bsearch(&u, X, N, 4, cmpu); if (hit) { float y; memcpy(&y, &Y[hit - X], 4); return y; }
  return LP(m < 1.0f ? LNC : LPC, m - 1.0f);
}
static float SQRT2_EDGE = 1.41421354f;   // m >= this goes down a binade (to be confirmed)
static float L2(float b) { int e; float m = frexpf(b, &e); m *= 2; e -= 1; if (m >= SQRT2_EDGE) { m *= 0.5f; e += 1; } volatile float l = LM(m); volatile float s = (float)e + l; return s; }
static float POW(float b, float g) { volatile float t = g * L2(b); float y = E2(t); return fabsf(y) < 1.17549435e-38f ? 0.0f : y; }
