import json
from pathlib import Path

import k2f

ROOT = Path(__file__).resolve().parents[3]


def _open(name):
    return k2f.Editor.open_dir(str(ROOT / "templates" / name))


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


def test_editor_relock_invoice_total():
    data = json.loads((ROOT / "examples/invoice/assets/data/invoice_data.json").read_text())
    headers = [cell["content"]["value"] for cell in data["rows"][0]]
    rows = [[cell["content"]["value"] for cell in row] for row in data["rows"][1:]]

    ed = _open("invoice")
    _append_node(ed, "root", _heading_node("invoice.header", 1, "STATEMENT #2025-001"))
    _append_node(ed, "root", _table_node("invoice.table", headers, rows))
    _append_node(ed, "root", _text_node("invoice.total", "body", "Grand Total: $7,047.00"))
    blob = bytes(ed.save_bytes())

    ed = k2f.Editor.open_bytes(blob)
    node = json.loads(ed.get_node("invoice.total"))
    assert "7,047.00" in node["content"]["value"]
    clip = json.loads(ed.clipboard("invoice.total"))
    assert clip["id"] == "invoice.total"
    assert "7,047.00" in clip["text"]
    assert "x" not in clip
    ed.replace_text("invoice.total", "Grand Total: $110.00")
    assert json.loads(ed.get_node("invoice.total"))["content"]["value"] == "Grand Total: $110.00"
    out = bytes(ed.save_bytes())
    assert out[:2] == b"PK"
    ed2 = k2f.Editor.open_bytes(out)
    assert json.loads(ed2.get_node("invoice.total"))["content"]["value"] == "Grand Total: $110.00"
    assert "STATEMENT" in json.loads(ed2.get_node("invoice.header"))["content"]["value"]


def test_editor_open_dir_invoice_template():
    template_dir = ROOT / "templates" / "invoice"
    ed = k2f.Editor.open_dir(str(template_dir))
    outline = json.loads(ed.outline())
    assert outline[0]["id"] == "root"
    ed.validate_package()


def test_compensation_amount_relock():
    ed = _open("legal")
    _append_node(ed, "root", _heading_node("contract.title", 1, "Independent Contractor Agreement"))
    _append_node(ed, "root", _text_node("contract.compensation.amount", "body", "USD 100"))
    blob = bytes(ed.save_bytes())
    secret, _public, _fp = k2f.generate_signing_key()
    signed = bytes(k2f.sign(blob, secret, "Acme Legal", 1_704_067_200))
    ed = k2f.Editor.open_bytes(signed)
    assert json.loads(ed.get_node("contract.compensation.amount"))["content"]["value"] == "USD 100"
    ed.replace_text("contract.compensation.amount", "USD 110")
    out = bytes(ed.save_bytes())
    ed2 = k2f.Editor.open_bytes(out)
    assert json.loads(ed2.get_node("contract.compensation.amount"))["content"]["value"] == "USD 110"
    sel = json.loads(ed2.selection("contract.compensation.amount"))
    assert sel["id"] == "contract.compensation.amount"
    assert sel["text"] == "USD 110"


if __name__ == "__main__":
    test_editor_relock_invoice_total()
    test_editor_open_dir_invoice_template()
    test_compensation_amount_relock()
    print("ok")
