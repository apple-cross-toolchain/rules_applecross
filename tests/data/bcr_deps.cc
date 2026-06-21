#include <stddef.h>

#include <fmt/format.h>
#include <zlib.h>

int main() {
  const char input[] = "rules_applecross bcr dependency fixture";
  unsigned char compressed[128];
  uLongf compressed_len = sizeof(compressed);

  int result = compress2(compressed, &compressed_len,
                         reinterpret_cast<const Bytef *>(input),
                         sizeof(input) - 1, Z_BEST_SPEED);
  if (result != Z_OK) {
    return result;
  }

  auto summary = fmt::format("zlib {} compressed {} bytes", zlibVersion(),
                             static_cast<size_t>(compressed_len));
  return summary.empty() ? 1 : 0;
}
