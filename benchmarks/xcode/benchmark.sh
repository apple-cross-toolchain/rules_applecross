#!/usr/bin/env bash
set -euo pipefail

mode="${1:-clean}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
project="${repo_root}/benchmarks/xcode/HelloIOSApp.xcodeproj"
scheme="HelloIOSApp"
configuration="${CONFIGURATION:-Debug}"
sdk="${SDKROOT:-iphoneos}"
destination="${DESTINATION:-}"

case "${mode}" in
  clean)
    derived_data="${DERIVED_DATA_DIR:-$(mktemp -d "${TMPDIR:-/tmp}/rules_applecross-xcode-clean.XXXXXX")}"
    ;;
  warm)
    derived_data="${DERIVED_DATA_DIR:-${TMPDIR:-/tmp}/rules_applecross-xcode-warm}"
    ;;
  incremental)
    derived_data="${DERIVED_DATA_DIR:-${TMPDIR:-/tmp}/rules_applecross-xcode-incremental}"
    ;;
  *)
    echo "usage: $0 [clean|warm|incremental]" >&2
    exit 2
    ;;
esac

build_args=(
  -project "${project}"
  -scheme "${scheme}"
  -configuration "${configuration}"
  -sdk "${sdk}"
  -derivedDataPath "${derived_data}"
  CODE_SIGNING_ALLOWED=NO
  CODE_SIGNING_REQUIRED=NO
  CODE_SIGN_IDENTITY=
  build
  -showBuildTimingSummary
)

if [[ -n "${destination}" ]]; then
  build_args=(-destination "${destination}" "${build_args[@]}")
fi

if [[ "${XCODEBUILD_QUIET:-0}" == "1" ]]; then
  build_args=(-quiet "${build_args[@]}")
fi

if [[ "${mode}" == "warm" ]]; then
  xcodebuild "${build_args[@]}" >/dev/null
elif [[ "${mode}" == "incremental" ]]; then
  xcodebuild "${build_args[@]}" >/dev/null
  touch "${repo_root}/tests/data/main.swift"
fi

echo "mode=${mode}"
echo "derived_data=${derived_data}"
/usr/bin/time -lp xcodebuild "${build_args[@]}"
