#!/bin/bash
# Prefer the host's Info-ZIP when present; fall back to the bundled Python
# implementation for executor images that ship neither zip nor python3-free
# alternatives.
for real in /usr/bin/zip /usr/local/bin/zip /bin/zip; do
  [[ -x "$real" ]] && exec "$real" "$@"
done
exec python3 "${BASH_SOURCE[0]}.py" "$@"
