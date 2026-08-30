import json
import sys
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


def _warning_node(node_id, text):
    return _text_node(node_id, "warning", text, break_inside="avoid")


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


def _build_invoice(data):
    headers = [cell["content"]["value"] for cell in data["rows"][0]]
    rows = [[cell["content"]["value"] for cell in row] for row in data["rows"][1:]]

    ed = k2f.Editor.open_template("invoice")
    _append_node(ed, "root", _heading_node("invoice.header", 1, "STATEMENT #2025-001"))
    _append_node(
        ed,
        "root",
        _text_node("invoice.details", "body", "Date: 15 Dec 2025\nBill To: TechInnovate Inc."),
    )
    _append_node(ed, "root", _table_node("invoice.table", headers, rows))
    _append_node(ed, "root", _warning_node("invoice.note", "Net 30."))
    ed.replace_text("invoice.note", "Net 14.")
    assert json.loads(ed.get_node("invoice.note"))["content"]["value"] == "Net 14."
    _append_node(ed, "root", _text_node("invoice.total", "body", "Grand Total: $7,047.00"))
    ed.set_running_footer("Page {{page_current}} of {{page_total}}")
    ed.validate_package()
    return ed


def test_invoice_from_real_json():
    data = json.loads((ROOT / "examples/invoice/assets/data/invoice_data.json").read_text())
    ed0 = _build_invoice(data)
    blob = bytes(ed0.save_bytes())
    assert blob[:2] == b"PK"
    node = json.loads(ed0.get_node("invoice.total"))
    assert "7,047.00" in node["content"]["value"]
    assert "layout" not in node
    ed0.replace_text("invoice.total", "Grand Total: $7,147.00")
    assert "7,147.00" in json.loads(ed0.get_node("invoice.total"))["content"]["value"]
    blob = bytes(ed0.save_bytes())
    pdf = bytes(ed0.export_pdf_bytes())
    assert pdf[:5] == b"%PDF-"
    try:
        k2f.Editor.open_bytes(pdf)
        raise AssertionError("PDF must not open as a K2F source")
    except RuntimeError as e:
        assert "PDF_IS_NOT_A_SOURCE" in str(e)

    ed = k2f.Editor.open_bytes(blob)
    ed.replace_text("invoice.total", "Grand Total: $7,247.00")
    relocked = bytes(ed.save_bytes())
    ed2 = k2f.Editor.open_bytes(relocked)
    assert "7,247.00" in json.loads(ed2.get_node("invoice.total"))["content"]["value"]
    pdf2 = bytes(ed2.export_pdf_bytes())
    assert pdf2[:5] == b"%PDF-"
    assert pdf != pdf2


def test_duplicate_id_code():
    ed = k2f.Editor.open_template("legal")
    _append_node(ed, "root", _text_node("n", "body", "a"))
    try:
        _append_node(ed, "root", _text_node("n", "body", "b"))
        raise AssertionError("expected duplicate id")
    except RuntimeError as e:
        assert str(e).startswith("DUPLICATE_ID:")


def test_system_prompt():
    text = k2f.system_prompt()
    assert "Do not write theme JSON" in text
    assert not hasattr(k2f, "invoice_system_prompt")


def test_format_schemas():
    import json

    schemas = k2f.format_schemas()
    expected = {
        "schema/manifest.schema.json",
        "schema/nodes.schema.json",
        "schema/styles.schema.json",
        "schema/visual_primitives.schema.json",
        "schema/signatures.schema.json",
    }
    assert set(schemas) == expected
    for key, text in schemas.items():
        parsed = json.loads(text)
        assert parsed["$schema"].startswith("https://json-schema.org/")


if __name__ == "__main__":
    test_invoice_from_real_json()
    test_duplicate_id_code()
    test_system_prompt()
    test_format_schemas()
    print("ok")
