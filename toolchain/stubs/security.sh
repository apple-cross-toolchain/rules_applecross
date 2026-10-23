#!/bin/bash

if [[ "$1" == "cms" ]]; then
  shift
  decode=false
  input=""
  output=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      -D) decode=true ;;
      -i) input="$2"; shift ;;
      -o) output="$2"; shift ;;
    esac
    shift
  done
  if ! $decode || [[ -z "$input" ]]; then
    exit 0
  fi
  extract() {
    local LC_ALL=C chunk
    while IFS= read -r -d '' chunk || [[ -n "$chunk" ]]; do
      if [[ "$chunk" == *"<?xml"*"</plist>"* ]]; then
        chunk="<?xml${chunk#*"<?xml"}"
        printf '%s\n' "${chunk%%"</plist>"*}</plist>"
        return
      fi
    done < "$input"
  }
  if [[ -n "$output" ]]; then
    extract > "$output"
  else
    extract
  fi
  exit 0
fi

if [[ "$1" == "find-identity" ]]; then
  echo '  1) AABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDD "Apple Development: Cross Compilation"'
  echo '     1 valid identities found'
fi

exit 0
