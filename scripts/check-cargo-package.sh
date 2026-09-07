#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONFIG="$(mktemp)"
trap 'rm -f "$CONFIG"' EXIT

cat >"$CONFIG" <<EOF
[patch.crates-io]
k2f_core = { path = "$ROOT/engine/k2f_core" }
k2f_text = { path = "$ROOT/engine/k2f_text" }
k2f_math = { path = "$ROOT/engine/k2f_math" }
k2f_layout = { path = "$ROOT/engine/k2f_layout" }
k2f_package = { path = "$ROOT/engine/k2f_package" }
k2f_markdown = { path = "$ROOT/engine/k2f_markdown" }
k2f_paint = { path = "$ROOT/engine/k2f_paint" }
k2f_pptx = { path = "$ROOT/export/k2f_pptx" }
k2f_docx = { path = "$ROOT/export/k2f_docx" }
k2f_pdf = { path = "$ROOT/engine/k2f_pdf" }
k2f_sdk = { path = "$ROOT/engine/k2f_sdk" }
EOF

cd "$ROOT"
for c in k2f_core k2f_text k2f_math k2f_markdown k2f_package k2f_layout k2f_paint k2f_pptx k2f_docx k2f_pdf k2f_sdk k2f; do
  echo "cargo package -p $c"
  cargo package -p "$c" --allow-dirty --no-verify --config "$CONFIG"
done

echo "all publishable crates packaged OK"
