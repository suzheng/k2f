#!/usr/bin/env bash
# DEPRECATED — do not run. See README.md in this directory.
#
# Legacy orchestrator: published crates.io, npm, and PyPI from a developer laptop.
# Superseded by .github/workflows/publish.yml (GitHub Actions).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
cd "$ROOT"

echo "=== Preflight ==="
bash scripts/publish-preflight.sh || true

echo "=== crates.io ==="
bash "$SCRIPT_DIR/publish-crates.sh" || echo "crates.io publish failed (see log)"

echo "=== npm ==="
bash "$SCRIPT_DIR/publish-npm.sh" || echo "npm publish failed (see log)"

echo "=== PyPI ==="
bash "$SCRIPT_DIR/publish-pypi-local.sh" || echo "PyPI publish failed (see log)"

echo "=== Done ==="
