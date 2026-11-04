#!/bin/bash
# docc off a Mac: swift.org's docc, of the Swift that built the SDKs (Xcode's
# docc is that open-source docc), beside its Swift libraries in
# ../libexec/docc. Its archives are Xcode's when they're given Xcode's render
# template, which the SDK archive carries in ../share/docc/render
# (tools/package-sdks.sh), and their JSON is written in one order, where
# Xcode's docc orders it differently every run (SWIFT_DETERMINISTIC_HASHING).
# Without that template, strict (the default) fails; relaxed
# (--config=relax_docc, RULES_APPLECROSS_RELAX_DOCC=1, or all of
# --config=relaxed) uses swift.org's own, whose pages work the same but whose
# HTML, JavaScript and styles differ from Xcode's.

here="$(cd "$(dirname "$0")/.." && pwd)"
render="$here/share/docc/render"
if [[ ! -f "$render/index.html" ]]; then
  if [[ "${RULES_APPLECROSS_RELAX_DOCC:-}" != "1" && "${RULES_APPLECROSS_RESOURCE_OUTPUT_MODE:-}" != "relaxed" ]]; then
    echo "error: docc: the SDK archive has no DocC render template (Xcode's usr/share/docc/render), so the archive can't be Xcode's; package the SDKs again with tools/package-sdks.sh, or build with --config=relax_docc to use swift.org's template" >&2
    exit 1
  fi
  echo "note: docc: relaxed: swift.org's render template, not Xcode's" >&2
  render="$here/libexec/docc/share/docc/render"
fi
# A catalog that is a link to a directory (rules_apple's docc_archive runs
# outside the sandbox, where the execroot links the workspace's top-level
# directories) is taken for none by docc on Linux, so it gets the directory.
args=()
for arg in "$@"; do
  if [[ -L "$arg" && -d "$arg" ]]; then
    arg="$(readlink -f "$arg")"
  fi
  args+=("$arg")
done
export DOCC_HTML_DIR="$render"
export SWIFT_DETERMINISTIC_HASHING=1
exec "$here/libexec/docc/bin/docc" "${args[@]}"
