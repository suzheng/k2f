#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="$ROOT/../k2f-private/.env"

if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
  if [[ ! -f "$ENV_FILE" ]]; then
    echo "missing CARGO_REGISTRY_TOKEN or $ENV_FILE (expected CREATE_API_KEY)" >&2
    exit 1
  fi
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
  export CARGO_REGISTRY_TOKEN="${CREATE_API_KEY:?CREATE_API_KEY missing in k2f-private/.env}"
fi

# Runtime dependency order. Dev-dependencies are stripped at publish time so
# path-only test deps do not block the index propagation chain.
CRATES=(
  k2f_core
  k2f_text
  k2f_math
  k2f_layout
  k2f_package
  k2f_markdown
  k2f_paint
  k2f_pptx
  k2f_docx
  k2f_pdf
  k2f_sdk
  k2f
)

manifest_for() {
  case "$1" in
    k2f_core) echo "$ROOT/engine/k2f_core/Cargo.toml" ;;
    k2f_text) echo "$ROOT/engine/k2f_text/Cargo.toml" ;;
    k2f_math) echo "$ROOT/engine/k2f_math/Cargo.toml" ;;
    k2f_layout) echo "$ROOT/engine/k2f_layout/Cargo.toml" ;;
    k2f_package) echo "$ROOT/engine/k2f_package/Cargo.toml" ;;
    k2f_markdown) echo "$ROOT/engine/k2f_markdown/Cargo.toml" ;;
    k2f_paint) echo "$ROOT/engine/k2f_paint/Cargo.toml" ;;
    k2f_pptx) echo "$ROOT/export/k2f_pptx/Cargo.toml" ;;
    k2f_docx) echo "$ROOT/export/k2f_docx/Cargo.toml" ;;
    k2f_pdf) echo "$ROOT/engine/k2f_pdf/Cargo.toml" ;;
    k2f_sdk) echo "$ROOT/engine/k2f_sdk/Cargo.toml" ;;
    k2f) echo "$ROOT/cli/k2f_cli/Cargo.toml" ;;
    *) echo "unknown crate: $1" >&2; exit 1 ;;
  esac
}

strip_dev_deps() {
  local manifest="$1"
  if grep -q '^\[dev-dependencies\]' "$manifest"; then
    cp "$manifest" "${manifest}.publishbak"
    awk 'BEGIN{keep=1} /^\[dev-dependencies\]/{keep=0} keep{print}' "$manifest" > "${manifest}.tmp"
    mv "${manifest}.tmp" "$manifest"
  fi
}

restore_dev_deps() {
  local manifest="$1"
  if [[ -f "${manifest}.publishbak" ]]; then
    mv "${manifest}.publishbak" "$manifest"
  fi
}

crate_version() {
  grep -m1 '^version =' "$(manifest_for "$1")" | sed 's/.*"\(.*\)"/\1/'
}

already_on_crates_io() {
  local ver
  ver="$(crate_version "$1")"
  curl -fsS -A "k2f-publish-script" "https://crates.io/api/v1/crates/$1/$ver" >/dev/null 2>&1
}

cd "$ROOT"
for c in "${CRATES[@]}"; do
  if already_on_crates_io "$c"; then
    echo "=== skip $c (already on crates.io) ==="
    continue
  fi

  manifest="$(manifest_for "$c")"
  strip_dev_deps "$manifest"
  trap 'restore_dev_deps "$manifest"' EXIT

  echo "=== cargo publish -p $c ==="
  cargo publish -p "$c" --allow-dirty --no-verify --token "$CARGO_REGISTRY_TOKEN"

  restore_dev_deps "$manifest"
  trap - EXIT

  if [[ "$c" != "k2f" ]]; then
    echo "waiting 90s for index propagation..."
    sleep 90
  fi
done

echo "crates.io publish complete"
