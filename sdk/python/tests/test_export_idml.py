import json
import zipfile
from io import BytesIO
from pathlib import Path

import k2f

ROOT = Path(__file__).resolve().parents[3]
INVOICE = ROOT / "examples/published/invoice.K2F"


def _stories_xml(idml: bytes) -> str:
    """IDML body text lives in Stories/*.xml, not Spreads."""
    with zipfile.ZipFile(BytesIO(idml)) as archive:
        names = sorted(
            name.replace("\\", "/")
            for name in archive.namelist()
            if name.replace("\\", "/").startswith("Stories/")
            and name.replace("\\", "/").endswith(".xml")
        )
        return "\n".join(archive.read(name).decode() for name in names)


def test_editor_export_idml_bytes():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    idml = bytes(ed.export_idml_bytes())
    assert idml.startswith(b"PK")
    # 拒绝 idml 当源由 Rust SDK `export_idml` 测；不要 Editor.open_bytes(idml)
    # （OpenedDocument 走 K2F unpack，不会发 IDML_IS_NOT_A_SOURCE）
    assert not hasattr(k2f, "export_idml")


def test_editor_export_idml_bytes_matches_saved_package():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    from_editor = bytes(ed.export_idml_bytes())
    saved = bytes(ed.save_bytes())
    ed2 = k2f.Editor.open_bytes(saved)
    from_saved = bytes(ed2.export_idml_bytes())
    assert from_editor == from_saved
    assert from_editor.startswith(b"PK")


def test_export_idml_unlocked_template_maps_error_code():
    ed = k2f.Editor.open_dir(str(ROOT / "templates" / "invoice"))
    try:
        ed.export_idml_bytes()
        raise AssertionError("expected UNLOCKED on template without lock")
    except RuntimeError as err:
        assert "UNLOCKED" in str(err)


def test_export_idml_invoice_contains_semantic_text():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    header = json.loads(ed.get_node("invoice.header"))["content"]["value"]
    xml = _stories_xml(bytes(ed.export_idml_bytes()))
    assert header in xml, "Stories XML must include visible invoice header text"
