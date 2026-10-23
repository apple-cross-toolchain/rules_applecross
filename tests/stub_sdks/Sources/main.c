#include "tests/stub_sdks/Sources/greeting.h"

int main(void) {
  const char *text = "Hello from C\n";
  greeting_print(text, greeting_length(text));
  return 0;
}
