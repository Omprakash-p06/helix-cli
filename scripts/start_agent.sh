#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "======================================================="
echo "  Starting Helix Agent Stack"
echo "======================================================="

if [[ -f "$PROJECT_DIR/venv/bin/activate" ]]; then
	# shellcheck disable=SC1091
	source "$PROJECT_DIR/venv/bin/activate"
fi

exec python3 "$PROJECT_DIR/start.py" "$@"
