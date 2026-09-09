#!/usr/bin/env bash
# DEPRECATED — do not run locally. PyPI publish is handled by publish.yml on git tag.
# See README.md in this directory.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"

: "${PYPI_TOKEN:=${TWINE_PASSWORD:-}}"
: "${PYPI_TOKEN:?PYPI_TOKEN or TWINE_PASSWORD required (source .env first)}"

export K2F_ENGINE_COMMIT_SHA="$(git -C "$ROOT" rev-parse HEAD)"
export CARGO_TARGET_DIR="$ROOT/target"

VERSION="$(grep -m1 '^version = ' "$ROOT/sdk/python/pyproject.toml" | sed 's/.*"\(.*\)"/\1/')"
WHEEL_DIR="$ROOT/target/wheels"
mkdir -p "$WHEEL_DIR"

cd "$ROOT/sdk/python"
python3 -m pip install -q maturin twine
python3 -m maturin build --release --out "$WHEEL_DIR"
python3 -m twine upload "$WHEEL_DIR/k2f-${VERSION}-"*.whl \
  --username __token__ \
  --password "$PYPI_TOKEN"

echo "PyPI publish complete"
