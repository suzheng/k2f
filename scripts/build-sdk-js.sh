#!/usr/bin/env bash
set -euo pipefail
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/sdk/js/wasm"
rustup target add wasm32-unknown-unknown >/dev/null
mkdir -p "$out"
if command -v wasm-pack >/dev/null 2>&1; then
  wasm-pack build "$root/engine/k2f_wasm" --target web --out-dir "$out" --release \
    -- --no-default-features --features viewer,sdk
else
  cargo build -p k2f_wasm --target wasm32-unknown-unknown --release \
    --no-default-features --features viewer,sdk
  wasm-bindgen --target web --out-dir "$out" \
    "$root/target/wasm32-unknown-unknown/release/k2f_wasm.wasm"
fi
rm -f "$out/package.json" "$out/README.md" "$out/.gitignore" "$out/.npmignore"
node "$root/scripts/emit-js-contracts.mjs"
echo "sdk wasm -> $out"
