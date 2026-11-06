// sha256_log: logs the input of CommonCrypto's SHA-256 functions in the process it is inserted into (README.md).
#include <CommonCrypto/CommonDigest.h>
#include <stdio.h>
#define INTERPOSE(r, o) __attribute__((used)) static struct { const void *a, *b; } _i_##o __attribute__((section("__DATA,__interpose"))) = { (const void *)r, (const void *)o };
static void dump(const char *what, const void *data, CC_LONG len) {
    fprintf(stderr, "%s %u ", what, len);
    const unsigned char *p = data;
    for (CC_LONG i = 0; i < len; i++) fprintf(stderr, "%02x", p[i]);
    fprintf(stderr, "\n");
}
int my_update(CC_SHA256_CTX *c, const void *data, CC_LONG len) { dump("UPDATE", data, len); return CC_SHA256_Update(c, data, len); }
unsigned char *my_sha256(const void *data, CC_LONG len, unsigned char *md) { dump("SHA256", data, len); return CC_SHA256(data, len, md); }
int my_init(CC_SHA256_CTX *c) { fprintf(stderr, "INIT\n"); return CC_SHA256_Init(c); }
int my_final(unsigned char *md, CC_SHA256_CTX *c) { int r = CC_SHA256_Final(md, c); dump("FINAL", md, 32); return r; }
INTERPOSE(my_update, CC_SHA256_Update)
INTERPOSE(my_sha256, CC_SHA256)
INTERPOSE(my_init, CC_SHA256_Init)
INTERPOSE(my_final, CC_SHA256_Final)
