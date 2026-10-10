#!/usr/bin/env bash
# Regenerate committed Ember artwork. Requires Python 3 & Pillow; no app build.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 "$ROOT/packaging/render-icons.py"
