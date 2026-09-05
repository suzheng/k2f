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
