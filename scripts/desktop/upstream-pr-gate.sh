#!/usr/bin/env bash
set -euo pipefail
exec node scripts/desktop/upstream-pr-gate.mjs "$@"
