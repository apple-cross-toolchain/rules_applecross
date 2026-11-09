#define device __attribute__((address_space(1)))
extern "C" void k(device float *a, unsigned i) { a[i] *= 2; }
