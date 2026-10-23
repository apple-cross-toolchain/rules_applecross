#!/bin/bash
# Reports RULES_APPLECROSS_BUILD_MACHINE_OS_BUILD (rules_apple's
# BuildMachineOSBuild) for -buildVersion; otherwise the ported sw_vers.
build="%{build_machine_os_build}"
case "${1:-}" in
  -buildVersion|--buildVersion) echo "$build"; exit 0 ;;
esac
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -z "${1:-}" ]; then
  "$here/sw_vers.ported" | sed "s/^\(BuildVersion:[[:space:]]*\).*/\1$build/"
else
  exec "$here/sw_vers.ported" "$@"
fi
