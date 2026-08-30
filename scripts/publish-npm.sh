#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="$ROOT/../k2f-private/.env"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "missing $ENV_FILE (expected NPM_TOKEN)" >&2
  exit 1
fi

set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a

: "${NPM_TOKEN:?NPM_TOKEN missing in k2f-private/.env}"

# Scoped packages require the npm org @openk2f (https://www.npmjs.com/org/openk2f).

cd "$ROOT"
bash scripts/build-sdk-js.sh
bash scripts/build-viewer-only-js.sh

cd "$ROOT/sdk/js"
export NODE_AUTH_TOKEN="$NPM_TOKEN"
npm publish --access public

echo "npm publish complete: $(node -p "require('./package.json').name")@$(node -p "require('./package.json').version")"
