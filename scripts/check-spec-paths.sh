#!/usr/bin/env bash
set -euo pipefail
spec=docs/spec/k2f-v0.1.md
for path in manifest.json content/root.json styles/theme.json document.K2F.lock signatures/v1.json; do
  grep -q "$path" "$spec" || { echo "missing path in spec: $path"; exit 1; }
done
for schema in manifest.schema.json nodes.schema.json styles.schema.json visual_primitives.schema.json signatures.schema.json; do
  grep -q "$schema" "$spec" || { echo "missing schema in spec: $schema"; exit 1; }
done
if grep -q 'agent_invoice.schema.json' "$spec"; then
  echo "spec must not list agent_invoice as a format schema"
  exit 1
fi
if grep -q 'OfficialTheme' "$spec"; then
  echo "spec must not require OfficialTheme (themes are SDK catalog only)"
  exit 1
fi
if grep -q 'Agents must use SDK' "$spec"; then
  echo "spec must not mandate SDK theme enum"
  exit 1
fi
echo "spec paths ok"
