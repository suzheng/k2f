"""Integration tests for the pip-installed `k2f` CLI console script."""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
INVOICE = ROOT / "examples/published/invoice.K2F"
CATALOG = ROOT / "skills/k2f/catalog"
PACK_VERIFY = ROOT / "skills/k2f/scripts/pack_verify.py"


def k2f_cmd(*args: str) -> subprocess.CompletedProcess[str]:
    exe = shutil.which("k2f")
    assert exe, "k2f not on PATH — pip install / maturin develop required"
    return subprocess.run(
        [exe, *args],
        capture_output=True,
        text=True,
        check=False,
    )


def test_k2f_on_path_and_help_lists_core_commands() -> None:
    assert shutil.which("k2f"), "k2f console script missing from PATH"
    proc = k2f_cmd("--help")
    assert proc.returncode == 0, proc.stderr
    help_text = proc.stdout.lower()
    for cmd in (
        "pack",
        "compile",
        "verify",
        "render",
        "export-pdf",
        "export-pptx",
        "export-docx",
        "markdown",
    ):
        assert cmd in help_text, f"missing {cmd} in k2f --help"


@pytest.mark.skipif(not INVOICE.is_file(), reason="invoice.K2F fixture missing")
def test_verify_render_export_pdf_on_invoice() -> None:
    verify = k2f_cmd("verify", str(INVOICE))
    banner = verify.stdout.strip()
    assert banner in ("UNSIGNED", "VALID", "ENGINE_MISMATCH"), verify.stdout
    if banner != "ENGINE_MISMATCH":
        assert verify.returncode == 0, verify.stderr

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        png = tmp_path / "page0.png"
        pdf = tmp_path / "out.pdf"

        if banner != "ENGINE_MISMATCH":
            render = k2f_cmd(
                "render",
                str(INVOICE),
                "--page",
                "0",
                "-o",
                str(png),
            )
            assert render.returncode == 0, render.stderr
            png_bytes = png.read_bytes()
            assert png_bytes[:8] == b"\x89PNG\r\n\x1a\n", "render did not write PNG"

        export = k2f_cmd(
            "export-pdf",
            str(INVOICE),
            "-o",
            str(pdf),
        )
        assert export.returncode == 0, export.stderr
        assert pdf.read_bytes().startswith(b"%PDF-"), "export-pdf did not write PDF"


@pytest.mark.skipif(not INVOICE.is_file(), reason="invoice.K2F fixture missing")
def test_export_docx_on_invoice() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        docx = Path(tmp) / "out.docx"
        export_docx = k2f_cmd(
            "export-docx",
            str(INVOICE),
            "-o",
            str(docx),
        )
        assert export_docx.returncode == 0, export_docx.stderr
        assert docx.read_bytes().startswith(b"PK"), "export-docx did not write ZIP"


@pytest.mark.skipif(not CATALOG.is_dir(), reason="skill catalog missing")
def test_pack_verify_catalog_without_k2f_cli_env() -> None:
    env = {k: v for k, v in __import__("os").environ.items() if k != "K2F_CLI"}
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "catalog.K2F"
        proc = subprocess.run(
            [
                sys.executable,
                str(PACK_VERIFY),
                str(CATALOG),
                "-o",
                str(out),
            ],
            capture_output=True,
            text=True,
            check=False,
            env=env,
        )
        assert proc.returncode == 0, proc.stdout + proc.stderr
        assert out.is_file() and out.stat().st_size > 0
        assert "UNSIGNED" in proc.stdout or "UNSIGNED" in proc.stderr


def test_init_package_pack_verify() -> None:
    init = ROOT / "skills/k2f/scripts/init_package.py"
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        doc = tmp_path / "doc"
        out = tmp_path / "doc.K2F"
        init_proc = subprocess.run(
            [
                sys.executable,
                str(init),
                "--dir",
                str(doc),
                "--title",
                "Pip CLI smoke",
                "--page",
                "a4",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert init_proc.returncode == 0, init_proc.stderr
        pack_proc = subprocess.run(
            [
                sys.executable,
                str(PACK_VERIFY),
                str(doc),
                "-o",
                str(out),
                "--expect-pages",
                "1",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert pack_proc.returncode == 0, pack_proc.stdout + pack_proc.stderr
        assert out.is_file() and out.stat().st_size > 0


def _run_init(dest: Path, extra: list[str] | None = None) -> subprocess.CompletedProcess[str]:
    init = ROOT / "skills/k2f/scripts/init_package.py"
    cmd = [
        sys.executable,
        str(init),
        "--dir",
        str(dest),
        "--title",
        "Pip CLI smoke",
        "--page",
        "a4",
    ]
    if extra:
        cmd.extend(extra)
    return subprocess.run(cmd, capture_output=True, text=True, check=False)


def test_init_package_into_empty_and_sidecar_dir() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        empty = tmp_path / "empty"
        empty.mkdir()
        proc = _run_init(empty)
        assert proc.returncode == 0, proc.stderr
        assert (empty / "manifest.json").is_file()

        notes = tmp_path / "notes"
        notes.mkdir()
        design = notes / "design.md"
        design.write_text("# spec\n", encoding="utf-8")
        proc = _run_init(notes)
        assert proc.returncode == 0, proc.stderr
        assert design.read_text(encoding="utf-8") == "# spec\n"
        assert (notes / "manifest.json").is_file()

        taken = tmp_path / "taken"
        taken.mkdir()
        (taken / "manifest.json").write_text("{}\n", encoding="utf-8")
        proc = _run_init(taken)
        assert proc.returncode != 0
        assert "exists" in proc.stderr.lower() or "package" in proc.stderr.lower()
        assert (taken / "manifest.json").read_text(encoding="utf-8") == "{}\n"


def test_init_package_font_fail_keeps_sidecar() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        dest = Path(tmp) / "doc"
        dest.mkdir()
        design = dest / "design.md"
        design.write_text("keep\n", encoding="utf-8")
        proc = _run_init(dest, ["--font", str(Path(tmp) / "missing.ttf")])
        assert proc.returncode != 0
        assert design.is_file()
        assert design.read_text(encoding="utf-8") == "keep\n"
        assert not (dest / "manifest.json").exists()


def test_pack_verify_bare_render_writes_to_author_dir() -> None:
    import importlib.util

    spec = importlib.util.spec_from_file_location("pack_verify", PACK_VERIFY)
    assert spec is not None and spec.loader is not None
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    source = Path("/tmp/k2f-author-dir")
    assert mod.resolve_render_path(Path("preview.png"), source) == (
        source / "preview.png"
    ).resolve()
    nested = mod.resolve_render_path(Path("out/preview.png"), source)
    assert nested != (source / "preview.png").resolve()

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        doc = tmp_path / "doc"
        out = tmp_path / "elsewhere" / "doc.K2F"
        init_proc = _run_init(doc)
        assert init_proc.returncode == 0, init_proc.stderr
        # Render still parses every fonts/ file on some CLI builds; licenses are
        # sidecars (Issue 1), not this path-resolution test.
        for extra in (doc / "assets" / "fonts").rglob("*"):
            if extra.is_file() and extra.suffix.lower() not in {".ttf", ".otf"}:
                extra.unlink()
        pack_proc = subprocess.run(
            [
                sys.executable,
                str(PACK_VERIFY),
                str(doc),
                "-o",
                str(out),
                "--expect-pages",
                "1",
                "--render",
                "preview.png",
            ],
            capture_output=True,
            text=True,
            check=False,
            cwd=tmp_path,
        )
        assert pack_proc.returncode == 0, pack_proc.stdout + pack_proc.stderr
        preview = doc / "preview.png"
        assert preview.is_file() and preview.read_bytes()[:8] == b"\x89PNG\r\n\x1a\n"
        assert not (tmp_path / "preview.png").exists()
