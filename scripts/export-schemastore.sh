#!/usr/bin/env bash
# Generate SchemaStore vendored copies from repo-root schema/*.schema.json.
# Does not modify source schemas (engine uses k2f:// $id and $ref).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/schema"
OUT="$ROOT/schema/schemastore"

export ROOT OUT
python3 <<'PY'
import json
import os
import re
from pathlib import Path

root = Path(os.environ["ROOT"])
src = root / "schema"
out = Path(os.environ["OUT"])
out.mkdir(parents=True, exist_ok=True)

MAP = {
    "manifest.schema.json": "k2f-manifest.json",
    "nodes.schema.json": "k2f-nodes.json",
    "styles.schema.json": "k2f-styles.json",
    "visual_primitives.schema.json": "partial-k2f-visual-primitives.json",
    "signatures.schema.json": "partial-k2f-signatures.json",
}

BASE = "https://www.schemastore.org"

# k2f://schema/foo.schema.json -> schemastore filename
def k2f_uri_to_http(uri: str) -> str:
    if uri.startswith("k2f://schema/"):
        src_name = uri[len("k2f://schema/") :]
        if src_name not in MAP:
            raise SystemExit(f"unknown k2f schema in ref: {uri}")
        return f"{BASE}/{MAP[src_name]}"
    return uri


def rewrite_refs(obj):
    if isinstance(obj, dict):
        if "$ref" in obj and isinstance(obj["$ref"], str):
            ref = obj["$ref"]
            if ref.startswith("k2f://schema/"):
                obj = {**obj, "$ref": k2f_uri_to_http(ref.split("#")[0]) + (
                    "#" + ref.split("#", 1)[1] if "#" in ref else ""
                )}
            elif ref == "k2f://schema/nodes.schema.json":
                obj = {**obj, "$ref": f"{BASE}/k2f-nodes.json"}
        return {k: rewrite_refs(v) for k, v in obj.items()}
    if isinstance(obj, list):
        return [rewrite_refs(v) for v in obj]
    return obj


for src_name, dest_name in MAP.items():
    path = src / src_name
    data = json.loads(path.read_text(encoding="utf-8"))
    data["$id"] = f"{BASE}/{dest_name}"
    data = rewrite_refs(data)
    dest = out / dest_name
    dest.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"wrote {dest.relative_to(root)}")

PY

echo "SchemaStore export OK: $OUT"
