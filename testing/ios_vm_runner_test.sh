#!/bin/bash
set -euo pipefail

exec python3 "$(dirname -- "$0")/ios_vm_runner_test.py"
