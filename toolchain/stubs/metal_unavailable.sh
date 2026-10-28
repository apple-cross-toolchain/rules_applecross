#!/bin/bash
# metal and metallib off a Mac: Apple's Metal compiler has no Linux version.
# Strict (the default), compiling a shader fails the build. Relaxed
# (RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed, --config=relaxed), it writes
# an empty output, so the app builds and its shaders fail when it runs.

tool="${0##*/}"
if [[ "${RULES_APPLECROSS_RESOURCE_OUTPUT_MODE:-}" != "relaxed" ]]; then
  echo "error: $tool: Apple's Metal compiler has no Linux version, so Metal shaders can't be compiled off a Mac; build them on a Mac, or with --config=relaxed (RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed) to write an empty $tool output" >&2
  exit 1
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
  echo "warning: $tool: relaxed: writing an empty $OUTPUT; its shaders will fail at run time" >&2
  touch "$OUTPUT"
fi
