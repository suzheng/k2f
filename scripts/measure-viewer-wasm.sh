#!/usr/bin/env bash
set -euo pipefail
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
root="$(cd "$(dirname "$0")/.." && pwd)"
target="wasm32-unknown-unknown"
rustup target add "$target" >/dev/null

echo "== cargo tree: viewer must not pull k2f_layout =="
if cargo tree -p k2f_wasm --no-default-features --features viewer 2>/dev/null | grep -q 'k2f_layout'; then
  echo "FAIL: k2f_wasm --features viewer depends on k2f_layout" >&2
  exit 1
fi
echo "ok: viewer feature has no k2f_layout"

measure() {
  local label="$1"
  local features="$2"
  local out_dir="$3"
  export CARGO_TARGET_DIR="$root/target/wasm-measure-$label"
  if command -v wasm-pack >/dev/null 2>&1; then
    wasm-pack build "$root/engine/k2f_wasm" --target web --out-dir "$out_dir" --release \
      -- --no-default-features --features "$features"
  else
    cargo build -p k2f_wasm --target "$target" --release --no-default-features --features "$features"
    mkdir -p "$out_dir"
    wasm-bindgen --target web --out-dir "$out_dir" \
      "$CARGO_TARGET_DIR/$target/release/k2f_wasm.wasm"
  fi
  local wasm="$out_dir/k2f_wasm_bg.wasm"
  if [[ ! -f "$wasm" ]]; then
    echo "missing $wasm" >&2
    exit 1
  fi
  local bytes
  bytes=$(wc -c <"$wasm" | tr -d ' ')
  echo "$label: $bytes bytes ($wasm)" >&2
  echo "$bytes"
}

vonly_bytes="$(measure "viewer-only" "viewer" "$root/sdk/js/wasm-viewer-measure")"
vsdk_bytes="$(measure "viewer+sdk" "viewer,sdk" "$root/sdk/js/wasm-measure")"

if [[ "${vonly_bytes}" -ge "${vsdk_bytes}" ]]; then
  echo "FAIL: viewer-only (${vonly_bytes}) should be smaller than viewer+sdk (${vsdk_bytes})" >&2
  exit 1
fi

echo "ok: viewer-only is smaller than viewer+sdk"
