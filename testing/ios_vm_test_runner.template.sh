#!/bin/bash

set -euo pipefail

premature_exit_file="${TEST_PREMATURE_EXIT_FILE:-}"
if [[ -n "$premature_exit_file" ]]; then
  touch "$premature_exit_file"
fi

mark_complete() {
  if [[ -n "$premature_exit_file" ]]; then
    rm -f "$premature_exit_file"
  fi
}

fail() {
  echo "error: $*" >&2
  mark_complete
  exit 1
}

if [[ "%(test_type)s" != "XCTEST" ]]; then
  fail "ios_vm_test_runner currently supports ios_unit_test only"
fi

if [[ -n "%(test_host_path)s" ]]; then
  fail "hosted iOS tests are not supported by ios_vm_test_runner"
fi

if [[ "${COVERAGE:-0}" == "1" ]]; then
  fail "coverage is not yet supported by ios_vm_test_runner"
fi

boot_args="$(cat <<'__IOS_VM_BOOT_ARGS__'
%(boot_args)s
__IOS_VM_BOOT_ARGS__
)"
test_filter="$(cat <<'__IOS_VM_TEST_FILTER__'
%(test_filter)s
__IOS_VM_TEST_FILTER__
)"
test_env_string="$(cat <<'__IOS_VM_TEST_ENV__'
%(test_env)s
__IOS_VM_TEST_ENV__
)"

runner_args=(
  --qemu "%(qemu_path)s"
  --bootkc "%(bootkc_path)s"
  --device-tree "%(device_tree_path)s"
  --trust-cache "%(trust_cache_path)s"
  --ramdisk "%(ramdisk_path)s"
  --slot-manifest "%(slot_manifest_path)s"
  --boot-args "$boot_args"
  --boot-attempts "%(boot_attempts)s"
  --boot-timeout "%(boot_timeout)s"
  --test-timeout "%(test_timeout)s"
  --test-bundle "%(test_bundle_path)s"
  --test-filter "$test_filter"
  --test-env-string "$test_env_string"
)

if [[ -n "%(sptm_path)s" ]]; then
  runner_args+=(--sptm "%(sptm_path)s" --txm "%(txm_path)s")
fi

test_env_inherit=%(test_env_inherit)s
for variable_name in "${test_env_inherit[@]:-}"; do
  if declare -p "$variable_name" >/dev/null 2>&1; then
    runner_args+=(--test-env "$variable_name=${!variable_name}")
  fi
done

if [[ -n "${XML_OUTPUT_FILE:-}" ]]; then
  runner_args+=(--xml-output "$XML_OUTPUT_FILE")
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --command_line_args=*)
      runner_args+=(--test-arg "${1#*=}")
      ;;
    *)
      fail "unsupported ios_vm_test_runner argument: $1"
      ;;
  esac
  shift
done

set +e
python3 "%(runner_path)s" "${runner_args[@]}"
test_exit_code=$?
set -e

mark_complete
exit "$test_exit_code"
