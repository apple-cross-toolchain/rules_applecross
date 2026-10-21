#!/bin/bash
# Runs a command in the Linux build container, with this checkout at its macOS
# path, the VM's read-only Xcode at /Applications/Xcode.app and a cache volume.
# RULES_APPLECROSS_VOLUMES adds "volume:path" mounts.
#
#   tools/linux_vm/run.sh bazel build //examples/ios/HelloWorldSwiftUI:HelloWorld
set -euo pipefail

IMAGE="${RULES_APPLECROSS_IMAGE:-localhost/rules-applecross-linux}"
VM_XCODE="${RULES_APPLECROSS_VM_XCODE:-/var/xcode/Xcode.app}"
CACHE_VOLUME="${RULES_APPLECROSS_CACHE_VOLUME:-rules-applecross-cache}"
WORKDIR="${RULES_APPLECROSS_WORKDIR:-$PWD}"

volumes=()
for v in ${RULES_APPLECROSS_VOLUMES:-}; do volumes+=(-v "$v"); done

# Record this Mac's macOS build as BuildMachineOSBuild.
build_machine=()
if command -v sw_vers >/dev/null; then
  build_machine=(-e "RULES_APPLECROSS_BUILD_MACHINE_OS_BUILD=$(sw_vers -buildVersion)")
fi

tty=()
if [ -t 0 ] && [ -t 1 ]; then tty=(-it); fi

exec podman run --rm ${tty[@]+"${tty[@]}"} \
  --security-opt label=disable \
  -v "$VM_XCODE:/Applications/Xcode.app:ro" \
  -v "/Users:/Users" \
  -v "$CACHE_VOLUME:/root/.cache" \
  ${volumes[@]+"${volumes[@]}"} \
  ${build_machine[@]+"${build_machine[@]}"} \
  -w "$WORKDIR" \
  "$IMAGE" "$@"
