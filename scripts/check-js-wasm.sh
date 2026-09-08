#!/usr/bin/env bash
# Fail if sdk/js shipped the viewer-only binary as the SDK WASM.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
sdk_js="$root/sdk/js/wasm/k2f_wasm.js"
sdk_wasm="$root/sdk/js/wasm/k2f_wasm_bg.wasm"
viewer_js="$root/sdk/js/wasm-viewer/k2f_wasm.js"
viewer_wasm="$root/sdk/js/wasm-viewer/k2f_wasm_bg.wasm"

for f in "$sdk_js" "$sdk_wasm" "$viewer_js" "$viewer_wasm"; do
  if [[ ! -f "$f" ]]; then
    echo "missing $f" >&2
    exit 1
  fi
done

if ! grep -q "markdown_to_k2f" "$sdk_js"; then
  echo "SDK wasm glue missing markdown_to_k2f ($sdk_js)" >&2
  exit 1
fi
if grep -q "markdown_to_k2f" "$viewer_js"; then
  echo "viewer-only wasm must not export markdown_to_k2f ($viewer_js)" >&2
  exit 1
fi

sdk_bytes=$(wc -c <"$sdk_wasm" | tr -d ' ')
viewer_bytes=$(wc -c <"$viewer_wasm" | tr -d ' ')
if [[ "$sdk_bytes" -le "$viewer_bytes" ]]; then
  echo "SDK wasm ($sdk_bytes) must be larger than viewer-only ($viewer_bytes)" >&2
  exit 1
fi

if cmp -s "$sdk_wasm" "$viewer_wasm"; then
  echo "SDK wasm is identical to viewer-only wasm; rebuild with isolated CARGO_TARGET_DIR" >&2
  exit 1
fi

echo "js wasm ok sdk=${sdk_bytes} viewer=${viewer_bytes}"
