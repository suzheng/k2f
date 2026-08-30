#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root/sdk/python"
source .venv/bin/activate
K2F_ROOT="$root" python3 - <<'PY'
import json
import os
from pathlib import Path

import k2f

root = Path(os.environ["K2F_ROOT"])
data_path = root / "examples/invoice/assets/data/invoice_data.json"
data = json.loads(data_path.read_text())
headers = [c["content"]["value"] for c in data["rows"][0]]
rows = [[c["content"]["value"] for c in r] for r in data["rows"][1:]]


def text_node(node_id, role, text, **extra):
    node = {"id": node_id, "role": role, "content": {"type": "text", "value": text}}
    node.update(extra)
    return node


def heading_node(node_id, level, text):
    role = f"h{level}" if level <= 4 else "h4"
    return text_node(node_id, role, text, keep_with_next=True)


def table_node(table_id, columns, rows):
    cols = len(columns)
    widths = [{"fr": 1}] * cols
    table_rows = [
        [text_node(f"{table_id}.h.c{i}", "table_header_cell", label) for i, label in enumerate(columns)]
    ]
    for ri, row in enumerate(rows):
        alt = ri % 2 == 1
        body = []
        for ci, value in enumerate(row):
            cell = text_node(f"{table_id}.r{ri}.c{ci}", "table_row_cell", value)
            if alt:
                cell["variant"] = "alt"
            body.append(cell)
        table_rows.append(body)
    return {
        "id": table_id,
        "role": "table",
        "content": {
            "type": "table",
            "value": {
                "column_widths": widths,
                "header_rows": 1,
                "gap": 4000,
                "data": {"type": "inline", "rows": table_rows},
            },
        },
    }


def child_count(ed, parent_id="root"):
    parent = json.loads(ed.get_node(parent_id))
    return len(parent["content"]["value"]["children"])


def append_node(ed, parent_id, node):
    ed.insert_node(parent_id, child_count(ed, parent_id), json.dumps(node))


ed = k2f.Editor.open_template("invoice")
append_node(ed, "root", heading_node("invoice.title", 1, "STATEMENT"))
append_node(ed, "root", table_node("invoice.lines", headers, rows))
append_node(ed, "root", text_node("invoice.total", "body", "Grand Total: $0.00"))
ed.validate_package()

out = Path("/tmp/k2f-getting-started.K2F")
out.write_bytes(bytes(ed.save_bytes()))
assert out.read_bytes()[:2] == b"PK"

pdf = Path("/tmp/k2f-getting-started.pdf")
pdf.write_bytes(bytes(ed.export_pdf_bytes()))
assert pdf.read_bytes()[:5] == b"%PDF-"

try:
    k2f.Editor.open_bytes(pdf.read_bytes())
    raise SystemExit("PDF must not open as K2F")
except RuntimeError as e:
    assert "PDF_IS_NOT_A_SOURCE" in str(e)

print("getting-started ok")
PY

cargo build -p k2f --release -q

if [[ -x "$root/target/release/k2f" ]]; then
  "$root/target/release/k2f" verify /tmp/k2f-getting-started.K2F
elif [[ -x "$root/target/debug/k2f" ]]; then
  K2F_ENGINE_COMMIT_SHA="$(git -C "$root" rev-parse HEAD)" \
    "$root/target/debug/k2f" verify /tmp/k2f-getting-started.K2F
elif command -v k2f >/dev/null 2>&1; then
  k2f verify /tmp/k2f-getting-started.K2F
else
  cargo run -p k2f --release --quiet -- verify /tmp/k2f-getting-started.K2F
fi
