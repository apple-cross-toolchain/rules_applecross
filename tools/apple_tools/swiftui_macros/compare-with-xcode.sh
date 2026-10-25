#!/bin/bash
# Compares macro expansions of probes/ with Apple's plugin and this one.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
swift build -q -c release
plugin="$PWD/.build/release/SwiftUIMacros#SwiftUIMacros"
out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT
status=0
for probe in probes/*.swift; do
  name="$(basename "$probe" .swift)"
  check=(xcrun --sdk iphonesimulator swiftc -target arm64-apple-ios18.0-simulator
         -typecheck -module-name Probe -Xfrontend -dump-macro-expansions "$probe")
  # Probes may fail to type-check on purpose (warnings and errors are
  # compared too).
  "${check[@]}" > "$out/$name.xcode" 2>&1 || true
  "${check[@]}" -load-plugin-executable "$plugin" > "$out/$name.clean-room" 2>&1 || true
  if cmp -s "$out/$name.xcode" "$out/$name.clean-room"; then
    echo "identical: $probe"
  else
    echo "DIFFERENT: $probe"
    diff "$out/$name.xcode" "$out/$name.clean-room" || true
    status=1
  fi
done
exit $status
