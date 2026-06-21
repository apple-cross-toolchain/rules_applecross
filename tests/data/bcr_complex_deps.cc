#include <stddef.h>

#include <fmt/format.h>
#include <png.h>
#include <sqlite3.h>
#include <zlib.h>

int main() {
  sqlite3 *db = nullptr;
  if (sqlite3_open(":memory:", &db) != SQLITE_OK) {
    if (db != nullptr) {
      sqlite3_close(db);
    }
    return 1;
  }
  sqlite3_close(db);

  png_structp png = png_create_read_struct(PNG_LIBPNG_VER_STRING, nullptr,
                                           nullptr, nullptr);
  if (png == nullptr) {
    return 2;
  }
  png_destroy_read_struct(&png, nullptr, nullptr);

  auto summary = fmt::format("{} {} {}", zlibVersion(),
                             png_access_version_number(), sqlite3_libversion());

  return summary.empty() ? 3 : 0;
}
