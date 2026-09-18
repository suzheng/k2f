import json
import zipfile
from io import BytesIO
from pathlib import Path

import k2f

ROOT = Path(__file__).resolve().parents[3]
INVOICE = ROOT / "examples/published/invoice.K2F"


def _inner_idml(pkg: bytes) -> bytes:
    with zipfile.ZipFile(BytesIO(pkg)) as archive:
        names = [
            name.replace("\\", "/")
            for name in archive.namelist()
            if name.replace("\\", "/").endswith(".idml")
        ]
        assert names, "package zip must contain an .idml"
        return archive.read(names[0])


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
    pkg = bytes(ed.export_idml_bytes())
    assert pkg.startswith(b"PK")
    with zipfile.ZipFile(BytesIO(pkg)) as archive:
        names = [n.replace("\\", "/") for n in archive.namelist()]
    assert any("Document Fonts" in n and n.endswith("Roboto-Regular.ttf") for n in names)
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
    xml = _stories_xml(_inner_idml(bytes(ed.export_idml_bytes()))).replace("\u00a0", " ")
    assert header in xml, "Stories XML must include visible invoice header text"


def test_export_idml_inner_mimetype_stored_first():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    idml = _inner_idml(bytes(ed.export_idml_bytes()))
    with zipfile.ZipFile(BytesIO(idml)) as archive:
        first = archive.infolist()[0]
        assert first.filename == "mimetype"
        assert first.compress_type == zipfile.ZIP_STORED
        assert (
            archive.read("mimetype").decode().strip()
            == "application/vnd.adobe.indesign-idml-package"
        )


def _font_bytes(pkg: bytes, name: str) -> bytes:
    with zipfile.ZipFile(BytesIO(pkg)) as archive:
        hits = [
            n.replace("\\", "/")
            for n in archive.namelist()
            if n.replace("\\", "/").endswith(name)
        ]
        assert hits, f"package zip missing {name}"
        return archive.read(hits[0])


def test_editor_export_idml_inner_matches_cli_zip():
    import tempfile

    from test_cli import k2f_cmd

    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    from_editor = bytes(ed.export_idml_bytes())
    with tempfile.TemporaryDirectory() as tmp:
        zpath = Path(tmp) / "invoice.zip"
        proc = k2f_cmd("export-idml", str(INVOICE), "-o", str(zpath))
        assert proc.returncode == 0, proc.stderr
        from_cli = zpath.read_bytes()
    assert from_cli.startswith(b"PK")
    assert _inner_idml(from_editor) == _inner_idml(from_cli)
    assert _font_bytes(from_editor, "Roboto-Regular.ttf") == _font_bytes(
        from_cli, "Roboto-Regular.ttf"
    )


def test_export_idml_only_bytes_is_lone_idml():
    ed = k2f.Editor.open_bytes(INVOICE.read_bytes())
    only = bytes(ed.export_idml_only_bytes())
    with zipfile.ZipFile(BytesIO(only)) as archive:
        assert archive.infolist()[0].filename == "mimetype"
