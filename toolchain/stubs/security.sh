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
    LC_ALL=C sed -n '/<?xml/,/<\/plist>/p' "$input" |
      LC_ALL=C sed -e '1s/^.*<?xml/<?xml/' -e '$s/<\/plist>.*$/<\/plist>/'
  }
  if [[ -n "$output" ]]; then
    extract > "$output"
  else
    extract
  fi
  exit 0
fi

if [[ "$1" == "find-identity" ]]; then
  # The certificates of the identity configured for codesign, by SHA-1 and
  # common name as security lists them, for rules_apple to match; without one,
  # a placeholder for ad hoc signing.
  certificates="%{certificates}"
  if [[ -n "$certificates" ]]; then
    n=0
    while IFS= read -r -d '' pem; do
      [[ -z "$pem" ]] && continue
      sha1=$(printf '%s\n' "$pem" | openssl x509 -noout -fingerprint -sha1 | sed -e 's/^.*=//' -e 's/://g')
      cn=$(printf '%s\n' "$pem" | openssl x509 -noout -subject -nameopt multiline | sed -n 's/^ *commonName *= *//p')
      n=$((n + 1))
      echo "  $n) $sha1 \"$cn\""
    done < <(awk '/-----BEGIN CERTIFICATE-----/{c=""} {c=c $0 "\n"} /-----END CERTIFICATE-----/{printf "%s%c", c, 0}' "$certificates")
    echo "     $n valid identities found"
  else
    echo '  1) AABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDD "Apple Development: Cross Compilation"'
    echo '     1 valid identities found'
  fi
fi

exit 0
