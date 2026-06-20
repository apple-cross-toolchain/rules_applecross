#!/bin/bash
# Stub for Apple tools that are unavailable on Linux and only need an output file.

OUTPUT=""
PREV=""
for arg in "$@"; do
  if [[ "$PREV" == "-o" ]]; then
    OUTPUT="$arg"
  fi
  PREV="$arg"
done

if [[ -n "$OUTPUT" ]]; then
  touch "$OUTPUT"
fi
