#!/usr/bin/env bash
set -euo pipefail
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/sdk/js/wasm"
# Isolated from build-viewer-only-js.sh — a shared cargo target lets the
# second wasm-pack overwrite k2f_wasm.wasm and both out-dirs end up identical.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/wasm-sdk}"
rustup target add wasm32-unknown-unknown >/dev/null
mkdir -p "$out"
artifact="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/k2f_wasm.wasm"
if command -v wasm-pack >/dev/null 2>&1; then
  wasm-pack build "$root/engine/k2f_wasm" --target web --out-dir "$out" --release \
    -- --no-default-features --features viewer,sdk
else
  cargo build -p k2f_wasm --target wasm32-unknown-unknown --release \
    --no-default-features --features viewer,sdk
  wasm-bindgen --target web --out-dir "$out" "$artifact"
fi
rm -f "$out/package.json" "$out/README.md" "$out/.gitignore" "$out/.npmignore"
node "$root/scripts/emit-js-contracts.mjs"
echo "sdk wasm -> $out"
