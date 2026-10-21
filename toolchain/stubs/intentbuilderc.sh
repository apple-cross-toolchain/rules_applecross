#!/bin/bash
if [[ -n "${PYTHON3:-}" ]]; then
  exec "$PYTHON3" "${BASH_SOURCE[0]}.py" "$@"
fi
exec python3 "${BASH_SOURCE[0]}.py" "$@"
