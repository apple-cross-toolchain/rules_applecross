#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/.real-sdk-tests"
FLAGS=(
  --repo_env=RULES_APPLECROSS_LOCAL_XCODE=1
  --ios_multi_cpus=sim_arm64
  --macos_cpus=arm64
  --tvos_cpus=sim_arm64
  --watchos_cpus=arm64
)
TOOLS=(CommandLine CommandLineSwift)

fail() {
  echo "error: $*" >&2
  exit 1
}

bundle_id() {
  case "$1" in
    CommandLine) echo com.example.command-line ;;
    CommandLineSwift) echo com.example.command-line-swift ;;
  esac
}

host_bazel() {
  bazel ${RULES_APPLECROSS_BAZEL_ROOT:+--output_user_root="$RULES_APPLECROSS_BAZEL_ROOT"} "$@"
}

run_tools() {
  local tool output
  for tool in "${TOOLS[@]}"; do
    output="$("$1/$tool" 2>&1)" || fail "$1/$tool exited with $?: $output"
    echo "$tool: $output"
    grep -q "Hello World from $(bundle_id "$tool")\$" <<<"$output" || fail "unexpected output from $1/$tool"
  done
}

simulator_runtimes() {
  xcrun simctl list runtimes 2>/dev/null | awk '/^[A-Za-z]+ [0-9]/ { print $1 }' | sort -u
}

macos() {
  [ "$(uname)" = Darwin ] || fail "the macos leg runs on a Mac"
  local runtimes platform excluded=() volume
  runtimes="$(simulator_runtimes)"
  for platform in iOS:ios tvOS:tvos watchOS:watchos visionOS:visionos; do
    if ! grep -qx "${platform%%:*}" <<<"$runtimes" && ! { [ "${platform%%:*}" = visionOS ] && grep -qx xrOS <<<"$runtimes"; }; then
      echo "::warning::No ${platform%%:*} simulator runtime, so Xcode's actool cannot build //examples/${platform#*:}/... here"
      excluded+=("-//examples/${platform#*:}/...")
    fi
  done
  cd "$ROOT"
  FLAGS+=(--repo_env=BAZEL_NO_APPLE_CPP_TOOLCHAIN=0)
  volume="$(df -P "$(host_bazel info output_base)" | awk 'NR == 2 { print $6 }')"
  if [ -d "$volume/.TemporaryItems" ]; then
    FLAGS+=(--sandbox_writable_path="$volume/.TemporaryItems")
  fi
  host_bazel build "${FLAGS[@]}" -- //examples/... ${excluded[@]+"${excluded[@]}"}
  mkdir -p "$OUT/macos"
  for tool in "${TOOLS[@]}"; do
    /bin/cp -fL "$(host_bazel cquery "${FLAGS[@]}" --output=files "//examples/macos/$tool")" "$OUT/macos/$tool"
  done
  run_tools "$OUT/macos"
}

linux_vm() {
  [ "$(uname)" = Darwin ] || fail "the linux-vm leg runs on a Mac"
  local engine="${RULES_APPLECROSS_ENGINE:-}" developer_dir
  if [ -z "$engine" ]; then
    if command -v podman >/dev/null; then engine=podman; else engine=docker; fi
  fi
  export RULES_APPLECROSS_ENGINE="$engine"
  export RULES_APPLECROSS_IMAGE="${RULES_APPLECROSS_IMAGE:-localhost/rules-applecross-linux}"
  export RULES_APPLECROSS_WORKDIR="$ROOT"
  if [ "$engine" = docker ] && [ -z "${RULES_APPLECROSS_VM_XCODE:-}" ]; then
    developer_dir="${DEVELOPER_DIR:-$(xcode-select -p)}"
    export RULES_APPLECROSS_VM_XCODE="${developer_dir%/Contents/Developer}"
  fi
  "$engine" image inspect "$RULES_APPLECROSS_IMAGE" >/dev/null 2>&1 ||
    "$engine" build -t "$RULES_APPLECROSS_IMAGE" "$ROOT/tools/linux_vm"
  mkdir -p "$OUT/linux-vm"
  "$ROOT/tools/linux_vm/run.sh" bash -c '
    set -euo pipefail
    out="$1"
    shift
    flags=("$@")
    bazel build --symlink_prefix=/ "${flags[@]}" //examples/...
    execroot="$(bazel info "${flags[@]}" execution_root)"
    for tool in CommandLine CommandLineSwift; do
      cp -fL "$execroot/$(bazel cquery "${flags[@]}" --output=files "//examples/macos/$tool")" "$out/$tool"
    done
  ' bash "$OUT/linux-vm" "${FLAGS[@]}"
  run_tools "$OUT/linux-vm"
}

case "${1:-}" in
  macos) macos ;;
  linux-vm) linux_vm ;;
  *) fail "usage: tools/real-sdk-tests.sh macos|linux-vm" ;;
esac
