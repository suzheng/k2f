import json
import zipfile
from io import BytesIO
from pathlib import Path

import k2f

ROOT = Path(__file__).resolve().parents[3]
INVOICE = ROOT / "examples/published/invoice.K2F"


def _document_xml(docx: bytes) -> str:
    with zipfile.ZipFile(BytesIO(docx)) as archive:
        return archive.read("word/document.xml").decode()


def test_editor_export_docx_bytes():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    docx = bytes(ed.export_docx_bytes())
    assert docx.startswith(b"PK")
    # 拒绝 docx 当源由 Rust SDK `export_docx` 测；不要 Editor.open_bytes(docx)
    # （OpenedDocument 走 K2F unpack，不会发 DOCX_IS_NOT_A_SOURCE）
    assert not hasattr(k2f, "export_docx")


def test_editor_export_docx_bytes_matches_saved_package():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    from_editor = bytes(ed.export_docx_bytes())
    saved = bytes(ed.save_bytes())
    ed2 = k2f.Editor.open_bytes(saved)
    from_saved = bytes(ed2.export_docx_bytes())
    assert from_editor == from_saved
    assert from_editor.startswith(b"PK")


def test_export_docx_unlocked_template_maps_error_code():
    ed = k2f.Editor.open_template("invoice")
    try:
        ed.export_docx_bytes()
        raise AssertionError("expected UNLOCKED on template without lock")
    except RuntimeError as err:
        assert "UNLOCKED" in str(err)


def test_export_docx_invoice_contains_semantic_text():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    header = json.loads(ed.get_node("invoice.header"))["content"]["value"]
    xml = _document_xml(bytes(ed.export_docx_bytes()))
    assert header in xml, "document.xml must include visible invoice header text"


def test_editor_unsaved_edit_needs_save_for_full_text():
    """DOCX text runs follow lock glyphs; save/relock before expecting edited copy."""
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    before = json.loads(ed.get_node("invoice.header"))["content"]["value"]
    before_xml = _document_xml(bytes(ed.export_docx_bytes()))
    assert before in before_xml

    token = "UNIQUE_PY_DOCX_UNSAVED_TOKEN"
    ed.replace_text("invoice.header", token)
    unsaved_xml = _document_xml(bytes(ed.export_docx_bytes()))
    assert token not in unsaved_xml
    assert before not in unsaved_xml

    saved = bytes(ed.save_bytes())
    saved_header = json.loads(k2f.Editor.open_bytes(saved).get_node("invoice.header"))[
        "content"
    ]["value"]
    assert saved_header == token
