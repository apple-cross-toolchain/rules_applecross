#!/bin/bash
# Stub for intentbuilderc on Linux.

OUTPUT_DIR=""
LANGUAGE=""
PREV=""
for arg in "$@"; do
  if [[ "$PREV" == "-output" ]]; then
    OUTPUT_DIR="$arg"
  fi
  if [[ "$PREV" == "-language" ]]; then
    LANGUAGE="$arg"
  fi
  PREV="$arg"
done

if [[ -n "$OUTPUT_DIR" ]]; then
  mkdir -p "$OUTPUT_DIR"
  if [[ "$LANGUAGE" == "Swift" ]]; then
    touch "$OUTPUT_DIR/Intents.swift"
  fi
fi
