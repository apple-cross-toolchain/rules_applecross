#!/bin/bash
# Trampoline so the Python implementation can run on executor images without
# a system python3: rules_apple tools export their hermetic interpreter as
# PYTHON3 before invoking SDK tools.
if [[ -n "${PYTHON3:-}" ]]; then
  exec "$PYTHON3" "${BASH_SOURCE[0]}.py" "$@"
fi
exec python3 "${BASH_SOURCE[0]}.py" "$@"
