#!/bin/bash
PYTHONHOME="$(cd "${BASH_SOURCE[0]%/*}/../libexec/python3" && pwd)"
export PYTHONHOME
exec "$PYTHONHOME/bin/python3" "${BASH_SOURCE[0]}.py" "$@"
