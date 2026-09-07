#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="$ROOT/../k2f-private/.env"

if [[ -z "${PYPI_TOKEN:-}" && -z "${TWINE_PASSWORD:-}" ]]; then
  if [[ ! -f "$ENV_FILE" ]]; then
    echo "missing PYPI_TOKEN/TWINE_PASSWORD or $ENV_FILE" >&2
    exit 1
  fi
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

: "${PYPI_TOKEN:=${TWINE_PASSWORD:-}}"
: "${PYPI_TOKEN:?PYPI_TOKEN or TWINE_PASSWORD required}"

export K2F_ENGINE_COMMIT_SHA="$(git -C "$ROOT" rev-parse HEAD)"

cd "$ROOT/sdk/python"
python3 -m pip install -q maturin twine
python3 -m maturin build --release
python3 -m twine upload ../../target/wheels/k2f-*.whl \
  --username __token__ \
  --password "$PYPI_TOKEN"

echo "PyPI publish complete"
