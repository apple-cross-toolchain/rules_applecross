#!/bin/bash
# The clean-room codesign, given the signing identity configured with
# --repo_env=RULES_APPLECROSS_CODESIGN_KEY and _CERTIFICATES: rules_apple runs
# codesign with an environment of its own, which these never reach.
export RULES_APPLECROSS_CODESIGN_KEY="%{key}"
export RULES_APPLECROSS_CODESIGN_CERTIFICATES="%{certificates}"
exec "$(dirname "${BASH_SOURCE[0]}")/codesign.bin" "$@"
