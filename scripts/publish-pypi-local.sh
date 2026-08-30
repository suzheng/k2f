#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="$ROOT/../k2f-private/.env"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "missing $ENV_FILE (expected PYPI_TOKEN)" >&2
  exit 1
fi

set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a

: "${PYPI_TOKEN:?PYPI_TOKEN missing in k2f-private/.env}"

export K2F_ENGINE_COMMIT_SHA="$(git -C "$ROOT" rev-parse HEAD)"

cd "$ROOT/sdk/python"
python3 -m pip install -q maturin twine
python3 -m maturin build --release
python3 -m twine upload ../../target/wheels/k2f-*.whl \
  --username __token__ \
  --password "$PYPI_TOKEN"

echo "PyPI publish complete"
