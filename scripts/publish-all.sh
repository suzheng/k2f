#!/usr/bin/env bash
# Publish all registry targets. Requires k2f-private/.env with tokens.
# Known blockers (2026-08-28):
#   - crates.io: account must have verified email (https://crates.io/settings/profile)
#   - npm: published as @openk2f/k2f (unscoped "k2f" blocked by npm typosquat policy)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== Preflight ==="
bash scripts/publish-preflight.sh || true

echo "=== crates.io ==="
bash scripts/publish-crates.sh || echo "crates.io publish failed (see log)"

echo "=== npm ==="
bash scripts/publish-npm.sh || echo "npm publish failed (see log)"

echo "=== PyPI ==="
bash scripts/publish-pypi-local.sh || echo "PyPI publish failed (see log)"

echo "=== Done ==="
