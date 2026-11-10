#!/bin/bash
# metal and metallib off a Mac. Apple's Metal compiler has no Linux version.
# Strict (the default), compiling a shader fails the build. Relaxed
# (--config=relax_metal: --@rules_applecross//toolchain:relax_metal and
# RULES_APPLECROSS_RELAX_METAL=1; or all of --config=relaxed),
# this repository's metal and metallib compile it with LLVM 15
# (tools/apple_tools/crates/metal): working shaders, not Xcode's bytes. Without
# them, or without LLVM 15 among the action's inputs, it writes an empty
# output, so the app builds and its shaders fail when it runs.

tool="${0##*/}"
# LLVM 15's tools, relative to the execution root (the action's directory):
# among the inputs only with --@rules_applecross//toolchain:relax_metal.
llvm_bin="%{llvm_bin}"
ported="${BASH_SOURCE[0]}.ported"
if [[ "${RULES_APPLECROSS_RELAX_METAL:-}" != "1" && "${RULES_APPLECROSS_RESOURCE_OUTPUT_MODE:-}" != "relaxed" && ! -x "$llvm_bin/clang" ]]; then
  echo "error: $tool: Apple's Metal compiler has no Linux version, so Metal shaders can't be compiled off a Mac as Xcode compiles them; bundle on a macOS execution platform (--@rules_applecross//toolchain:apple_bundling=macos), where Xcode's metal compiles them, or build with --config=relax_metal to compile them with this repository's metal" >&2
  exit 1
fi

if [[ -x "$ported" && -x "$llvm_bin/clang" ]]; then
  exec "$ported" --llvm-bin "$llvm_bin" "$@"
fi

OUTPUT=""
PREV=""
for arg in "$@"; do
  if [[ "$PREV" == "-o" ]]; then
    OUTPUT="$arg"
  fi
  PREV="$arg"
done

if [[ -n "$OUTPUT" ]]; then
  if [[ -x "$ported" ]]; then
    why="LLVM 15 isn't among this action's inputs (build with --@rules_applecross//toolchain:relax_metal, which --config=relax_metal sets)"
  else
    why="these ported tools have no $tool"
  fi
  echo "warning: $tool: relaxed: $why; writing an empty $OUTPUT, whose shaders will fail at run time" >&2
  touch "$OUTPUT"
fi
