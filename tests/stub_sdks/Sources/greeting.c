#include "tests/stub_sdks/Sources/greeting.h"

long write(int fd, const void *buffer, size_t length);

size_t greeting_length(const char *text) {
  size_t length = 0;
  while (text[length]) {
    length++;
  }
  return length;
}

void greeting_print(const void *text, size_t length) { write(1, text, length); }
