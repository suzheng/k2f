import io
import json
import zipfile
from pathlib import Path

import k2f

ROOT = Path(__file__).resolve().parents[3]


def _text_node(node_id, role, text, **extra):
    node = {"id": node_id, "role": role, "content": {"type": "text", "value": text}}
    node.update(extra)
    return node


def _heading_node(node_id, level, text):
    role = f"h{level}" if level <= 4 else "h4"
    return _text_node(node_id, role, text, keep_with_next=True)


def _table_node(table_id, columns, rows):
    cols = len(columns)
    widths = [{"fr": 1}] * cols
    table_rows = [
        [_text_node(f"{table_id}.h.c{i}", "table_header_cell", label) for i, label in enumerate(columns)]
    ]
    for ri, row in enumerate(rows):
        alt = ri % 2 == 1
        body = []
        for ci, value in enumerate(row):
            cell = _text_node(f"{table_id}.r{ri}.c{ci}", "table_row_cell", value)
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


def _child_count(ed, parent_id="root"):
    parent = json.loads(ed.get_node(parent_id))
    return len(parent["content"]["value"]["children"])


def _append_node(ed, parent_id, node):
    ed.insert_node(parent_id, _child_count(ed, parent_id), json.dumps(node))


def test_save_is_unsigned_sign_uses_lock_and_utc():
    data = json.loads((ROOT / "examples/invoice/assets/data/invoice_data.json").read_text())
    headers = [cell["content"]["value"] for cell in data["rows"][0]]
    rows = [[cell["content"]["value"] for cell in row] for row in data["rows"][1:]]

    ed = k2f.Editor.open_template("invoice")
    ed.set_generated_by("agent.invoice-bot")
    _append_node(ed, "root", _heading_node("invoice.header", 1, "STATEMENT #2025-001"))
    _append_node(ed, "root", _table_node("invoice.table", headers, rows))
    _append_node(ed, "root", _text_node("invoice.total", "body", "Grand Total: $7,047.00"))
    blob = bytes(ed.save_bytes())
    names = zipfile.ZipFile(io.BytesIO(blob)).namelist()
    assert "signatures/v1.json" not in names
    assert "document.K2F.lock" in names

    secret, public, _fp = k2f.generate_signing_key()
    signed = bytes(k2f.sign(blob, secret, "Jane Doe", 1_704_067_200))
    rec = json.loads(zipfile.ZipFile(io.BytesIO(signed)).read("signatures/v1.json"))
    assert rec["version"] == 1
    assert rec["signatures"][0]["alg"] == "ed25519"
    assert rec["signatures"][0]["signed_by"] == "Jane Doe"
    assert rec["signatures"][0]["signed_at"] == 1_704_067_200
    assert rec["signatures"][0]["public_key"] == public

    now = bytes(k2f.sign(blob, secret))
    rec_now = json.loads(zipfile.ZipFile(io.BytesIO(now)).read("signatures/v1.json"))
    assert rec_now["signatures"][0]["signed_at"] > 1_700_000_000


if __name__ == "__main__":
    test_save_is_unsigned_sign_uses_lock_and_utc()
    print("ok")
