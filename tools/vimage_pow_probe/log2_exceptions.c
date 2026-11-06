#include <stdio.h>
#include <math.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
// lexc: log2 models (positive: lm62_pos, all fused; negative: ncur, pattern 55) against the exact recovered values; writes exceptions.
static float FM(int fused, float a, float b, float cc) { if (fused) return fmaf(a, b, cc); volatile float m = a * b; return m + cc; }
static float model(const float *c, int mask, float r) {
  volatile float x2v = r * r; float x2 = x2v;
  float ev = FM(mask & 4, FM(mask & 2, FM(mask & 1, c[6], x2, c[4]), x2, c[2]), x2, c[0]);
  float od = FM(mask & 16, FM(mask & 8, c[5], x2, c[3]), x2, c[1]);
  volatile float L = r * FM(mask & 32, od, r, ev); return L;
}
static void run(const char *data, const char *coef, int mask, const char *out) {
  float c[7]; FILE *f = fopen(coef, "rb"); fread(c, 4, 7, f); fclose(f);
  f = fopen(data, "rb"); fseek(f, 0, SEEK_END); size_t n = ftell(f) / 12; rewind(f); float *d = malloc(n * 12); fread(d, 12, n, f); fclose(f);
  FILE *o = fopen(out, "wb"); long miss = 0;
  for (size_t i = 0; i < n; i++) { float m = d[3 * i], L = d[3 * i + 1]; float r = m - 1.0f; if (model(c, mask, r) != L) { miss++; fwrite(&d[3 * i], 4, 2, o); } }
  fclose(o); printf("%s: %ld exceptions of %zu\n", data, miss, n);
}
int main(void) { run("lpos2.bin", "lfin_pos.bin", 63, "lexc_pos.bin"); run("lneg2.bin", "lfin_neg.bin", 63, "lexc_neg.bin"); }
