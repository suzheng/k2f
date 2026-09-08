#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="$ROOT/../k2f-private/.env"

if [[ -z "${NPM_TOKEN:-}" && -z "${NODE_AUTH_TOKEN:-}" ]]; then
  if [[ ! -f "$ENV_FILE" ]]; then
    echo "missing NPM_TOKEN/NODE_AUTH_TOKEN or $ENV_FILE" >&2
    exit 1
  fi
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

: "${NPM_TOKEN:=${NODE_AUTH_TOKEN:-}}"
: "${NPM_TOKEN:?NPM_TOKEN or NODE_AUTH_TOKEN required}"

# Scoped packages require the npm org @openk2f (https://www.npmjs.com/org/openk2f).

cd "$ROOT"
bash scripts/build-sdk-js.sh
bash scripts/build-viewer-only-js.sh
bash scripts/check-js-wasm.sh

cd "$ROOT/sdk/js"
NPMRC="$ROOT/sdk/js/.npmrc.publish"
printf '//registry.npmjs.org/:_authToken=%s\n' "$NPM_TOKEN" > "$NPMRC"
trap 'rm -f "$NPMRC"' EXIT
npm publish --access public --userconfig "$NPMRC"

echo "npm publish complete: $(node -p "require('./package.json').name")@$(node -p "require('./package.json').version")"
