#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/schema"
DST="$ROOT/engine/k2f_package/schema"
SKILL="$ROOT/skills/k2f/schema"
AGENT_SRC="$ROOT/docs/instructions/agent_v0.md"
AGENT_DST="$ROOT/engine/k2f_sdk/instructions/agent_v0.md"

fail=0
for f in manifest.schema.json nodes.schema.json styles.schema.json visual_primitives.schema.json signatures.schema.json; do
  if ! diff -q "$SRC/$f" "$DST/$f" >/dev/null 2>&1; then
    echo "vendored schema drift: $f (expected $DST/$f to match $SRC/$f)" >&2
    fail=1
  fi
  if ! diff -q "$SRC/$f" "$SKILL/$f" >/dev/null 2>&1; then
    echo "skill schema drift: $f (expected $SKILL/$f to match $SRC/$f)" >&2
    fail=1
  fi
done

if ! diff -q "$AGENT_SRC" "$AGENT_DST" >/dev/null 2>&1; then
  echo "vendored agent prompt drift: agent_v0.md" >&2
  fail=1
fi

if [[ "$fail" -ne 0 ]]; then
  echo "Run: cp schema/*.schema.json engine/k2f_package/schema/ && cp schema/*.schema.json skills/k2f/schema/ && cp docs/instructions/agent_v0.md engine/k2f_sdk/instructions/" >&2
  exit 1
fi

echo "vendored schema, skill schema, and agent_v0.md OK"
