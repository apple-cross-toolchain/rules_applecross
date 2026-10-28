#!/bin/bash
# Builds every example's bundles and binaries and compares them across hosts:
# `collect DIR` on a Mac with Xcode and on Linux, then `compare MAC LINUX`.
#
#   tools/compare_examples.sh collect .compare/mac
#   tools/compare_examples.sh compare .compare/mac .compare/linux
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"

FLAGS=(
  --ios_multi_cpus=sim_arm64
  --macos_cpus=arm64
  --tvos_cpus=sim_arm64
  --watchos_cpus=arm64
)
KINDS='(_application|_extension|_framework|_bundle|_test|^xcarchive) rule'
# Manual targets (macOS-only by design) are left out, as with //examples/....
TARGETS="kind('$KINDS', //examples/...) except attr(tags, manual, //examples/...)"

collect() {
  local out="$1"
  if [ "$(uname)" = Darwin ]; then FLAGS+=(--repo_env=BAZEL_NO_APPLE_CPP_TOOLCHAIN=0); fi
  local targets
  targets="$(bazel query "$TARGETS")"
  bazel build --keep_going --build_event_json_file="$out.bep" "${FLAGS[@]}" -- $targets > "$out.log" 2>&1 || true
  # Every top-level output's absolute path, from the build's event stream.
  python3 -c '
import json, sys
for line in open(sys.argv[1]):
    e = json.loads(line)
    for f in e.get("namedSetOfFiles", {}).get("files", []):
        if f.get("uri", "").startswith("file://"):
            print(f["uri"][len("file://"):])
' "$out.bep" | sort -u > "$out.files"
  rm -f "$out.bep"
  if [ -e "$out" ]; then chmod -R u+w "$out" && rm -rf "$out"; fi
  mkdir -p "$out"
  while read -r f; do
    # .../bazel-out/<configuration>/bin/<path>, without the configuration's hash.
    local rel="${f#*/bazel-out/}"
    local cfg="${rel%%/*}"; cfg="${cfg%-ST-*}"
    # Test runners and other outputs for the host itself.
    case "$cfg" in aarch64-*|k8-*|darwin_arm64-fastbuild|darwin_arm64-opt) cfg="host-${cfg#*-}" ;; esac
    rel="${rel#*/bin/}"
    mkdir -p "$out/$cfg/$(dirname "$rel")"
    cp -RL "$f" "$out/$cfg/$rel"
  done < "$out.files"
  chmod -R u+w "$out"
  rm -f "$out.files"
}

compare() {
  local a="$1" b="$2" status=0 work
  work="$(mktemp -d)"
  trap 'rm -rf "$work"' RETURN
  while read -r f; do
    if [ ! -e "$b/$f" ]; then
      echo "only in $a: $f"; status=1; continue
    fi
    if diff -rq "$a/$f" "$b/$f" >/dev/null; then
      echo "identical: $f"; continue
    fi
    # Xcode varies some outputs from run to run (docs/clean-room/README.md#determinism).
    if python3 "$here/xcode_equivalence.py" "$a/$f" "$b/$f" >/dev/null; then
      echo "equivalent (up to Xcode's own variation): $f"; continue
    fi
    status=1
    case "$f" in
      *.ipa|*.zip|*.xcarchive.zip)
        rm -rf "$work/a" "$work/b"
        mkdir -p "$work/a" "$work/b"
        (cd "$work/a" && unzip -qo "$a/$f")
        (cd "$work/b" && unzip -qo "$b/$f")
        if diff -rq "$work/a" "$work/b" >/dev/null; then
          echo "DIFFERENT (zip metadata only): $f"
        elif python3 "$here/xcode_equivalence.py" "$work/a" "$work/b" >/dev/null; then
          echo "DIFFERENT (zip metadata, and Xcode's own variation, only): $f"
        else
          echo "DIFFERENT: $f"
          diff -rq "$work/a" "$work/b" | sed "s|$work/[ab]/||g; s/^/    /" || true
        fi
        ;;
      *)
        echo "DIFFERENT: $f"
        diff -rq "$a/$f" "$b/$f" | sed "s|$a/||g; s|$b/||g; s/^/    /" || true
        ;;
    esac
  done < <(cd "$a" && find . -type f | sed 's|^\./||' | sort)
  while read -r f; do
    [ -e "$a/$f" ] || { echo "only in $b: $f"; status=1; }
  done < <(cd "$b" && find . -type f | sed 's|^\./||' | sort)
  return $status
}

cmd="${1:-}"; shift || true
case "$cmd" in
  collect) collect "$(mkdir -p "$1" && cd "$1" && pwd)" ;;
  compare) compare "$(cd "$1" && pwd)" "$(cd "$2" && pwd)" ;;
  *) echo "usage: $0 collect DIR | compare MAC LINUX" >&2; exit 2 ;;
esac
