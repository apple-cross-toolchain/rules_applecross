#!/bin/bash

llvm_strip="${BASH_SOURCE[0]%/*}/llvm-strip"
args=("$@")
flags=()
files=()
output=""
swift=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    -o)
      if [[ $# -lt 2 ]]; then
        exec "$llvm_strip" "${args[@]}"
      fi
      output="$2"
      shift
      ;;
    -[A-Za-z]*)
      if [[ ! "$1" =~ ^-[A-Za-z]+$ ]]; then
        exec "$llvm_strip" "${args[@]}"
      fi
      if [[ "$1" == *T* ]]; then
        swift=true
      fi
      if [[ "${1//T/}" != "-" ]]; then
        flags+=("${1//T/}")
      fi
      ;;
    -*)
      exec "$llvm_strip" "${args[@]}"
      ;;
    *)
      files+=("$1")
      ;;
  esac
  shift
done

if ! $swift || [[ ${#flags[@]} -eq 0 ]]; then
  exec "$llvm_strip" "${args[@]}"
fi

if [[ -n "$output" ]]; then
  "$llvm_strip" "${flags[@]}" -o "$output" "${files[@]}" || exit
  exec "$llvm_strip" -T "$output"
fi
"$llvm_strip" "${flags[@]}" "${files[@]}" || exit
exec "$llvm_strip" -T "${files[@]}"
