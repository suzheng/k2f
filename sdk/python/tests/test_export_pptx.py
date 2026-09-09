import json
import zipfile
from io import BytesIO
from pathlib import Path

import k2f

ROOT = Path(__file__).resolve().parents[3]
INVOICE = ROOT / "examples/published/invoice.K2F"


def _slides_xml(pptx: bytes) -> str:
    with zipfile.ZipFile(BytesIO(pptx)) as archive:
        names = sorted(
            name
            for name in archive.namelist()
            if name.startswith("ppt/slides/slide") and name.endswith(".xml")
        )
        return "\n".join(archive.read(name).decode() for name in names)


def test_editor_export_pptx_bytes():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    pptx = bytes(ed.export_pptx_bytes())
    assert pptx.startswith(b"PK")
    # 拒绝 pptx 当源由 Rust SDK `export_pptx` 测；不要 Editor.open_bytes(pptx)
    # （OpenedDocument 走 K2F unpack，不会发 PPTX_IS_NOT_A_SOURCE）
    assert not hasattr(k2f, "export_pptx")


def test_editor_export_pptx_bytes_matches_saved_package():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    from_editor = bytes(ed.export_pptx_bytes())
    saved = bytes(ed.save_bytes())
    ed2 = k2f.Editor.open_bytes(saved)
    from_saved = bytes(ed2.export_pptx_bytes())
    assert from_editor == from_saved
    assert from_editor.startswith(b"PK")


def test_editor_unsaved_edit_uses_in_memory_manifest_text():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    token = "UNIQUE_PY_PPTX_UNSAVED_TOKEN"
    ed.replace_text("invoice.header", token)
    # PPTX maps lock glyph ranges onto semantic text; dirty tree/lock pairs
    # are not a stable export. Relock (save) is the contract.
    saved = bytes(ed.save_bytes())
    saved_xml = _slides_xml(bytes(k2f.Editor.open_bytes(saved).export_pptx_bytes()))
    assert token in saved_xml, "relocked export must keep edited header text"


def test_export_pptx_unlocked_template_maps_error_code():
    ed = k2f.Editor.open_dir(str(ROOT / "templates" / "invoice"))
    try:
        ed.export_pptx_bytes()
        raise AssertionError("expected UNLOCKED on template without lock")
    except RuntimeError as err:
        assert "UNLOCKED" in str(err)


def test_export_pptx_invoice_contains_semantic_text():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    header = json.loads(ed.get_node("invoice.header"))["content"]["value"]
    xml = _slides_xml(bytes(ed.export_pptx_bytes()))
    assert header in xml, "pptx slide XML must include visible invoice header text"
