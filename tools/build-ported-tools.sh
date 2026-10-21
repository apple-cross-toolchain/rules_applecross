#!/bin/bash
# Builds apple-cross-toolchain/ci's ported tools (same revisions) for this
# host's CPU, as ported-tools-linux-<cpu>.tar.xz in the workspace root.
# Ubuntu 24.04 dependencies:
#   apt-get install build-essential clang cmake curl git libpng-dev \
#     libxml2-dev ninja-build patchelf python3 python-is-python3 xz-utils \
#     zlib1g-dev cargo
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARCH="$(uname -m)"
WORK="$(mktemp -d)"
OUT="$WORK/out"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$OUT/lib" "$OUT/bin"
cd "$WORK"

git clone -q https://github.com/apple-cross-toolchain/xcbuild-rust.git
(cd xcbuild-rust && git checkout -q aeef77013f26d89d7b19247ccc3f74b6bdb29929 \
  && cargo build -q --release \
  && find target/release -maxdepth 1 -type f -executable -exec cp {} "$OUT/bin" \;)

curl -fsSL https://sourceforge.net/projects/pmt/files/pngcrush/1.8.13/pngcrush-1.8.13.tar.gz/download -o pngcrush.tar.gz
tar -xf pngcrush.tar.gz
(cd pngcrush-1.8.13 && make -s -j"$(nproc)" && mv pngcrush "$OUT/bin")

git clone -q https://github.com/tpoechtrager/apple-libtapi.git
(cd apple-libtapi && git checkout -q 664b8414f89612f2dfd35a9b679c345aa5389026 \
  && INSTALLPREFIX="$OUT" ./build.sh >/dev/null && ./install.sh >/dev/null)

git clone -q https://github.com/tpoechtrager/cctools-port.git
(cd cctools-port/cctools && git checkout -q faa1f24cb7e31be132f98f503cc447c90ce2fd87 \
  && ./configure -q --prefix="$OUT" --with-libtapi="$OUT" \
  && make -s -j"$(nproc)" && make -s install)
patchelf --set-rpath '$ORIGIN/../lib' "$OUT/bin/ld"
rm -f "$OUT/bin/dsymutil" "$OUT/bin/nm"

# Placeholders, as in ci's archive.
for tool in codesign ibtool swift-stdlib-tool; do
  printf '#!/bin/bash\n' > "$OUT/bin/$tool"
  chmod +x "$OUT/bin/$tool"
done

archive="$ROOT/ported-tools-linux-$ARCH.tar.xz"
tar -C "$OUT" -Jcf "$archive" bin lib
sha256sum "$archive"
