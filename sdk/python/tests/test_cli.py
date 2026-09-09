"""Integration tests for the pip-installed `k2f` CLI console script."""

from __future__ import annotations

import json
import os
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
INIT_PACKAGE = ROOT / "skills/k2f/scripts/init_package.py"


def resolve_k2f() -> str:
    """Prefer a checkout-built CLI over a stale global `~/.cargo/bin/k2f`.

    Workspace `target/` is checked before the pytest venv: an old
    `maturin develop` wrapper can be as stale as a cargo-installed binary.
    """
    env_cli = os.environ.get("K2F_CLI")
    if env_cli:
        path = Path(env_cli).expanduser().resolve()
        if path.is_file():
            return str(path)
    for rel in ("target/debug/k2f", "target/release/k2f"):
        path = ROOT / rel
        if path.is_file():
            return str(path)
    venv_cli = Path(sys.executable).parent / "k2f"
    if venv_cli.is_file():
        return str(venv_cli)
    which = shutil.which("k2f")
    assert which, "k2f not found — pip install / maturin develop, or cargo build -p k2f"
    return which


def pack_verify_env(*, with_k2f_cli: bool = True) -> dict[str, str]:
    exe = resolve_k2f()
    env = dict(os.environ)
    env["PATH"] = str(Path(exe).parent) + os.pathsep + env.get("PATH", "")
    if with_k2f_cli:
        env["K2F_CLI"] = exe
    else:
        env.pop("K2F_CLI", None)
    return env


def k2f_cmd(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [resolve_k2f(), *args],
        capture_output=True,
        text=True,
        check=False,
    )


def test_k2f_on_path_and_help_lists_core_commands() -> None:
    resolve_k2f()
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
    assert banner == "UNSIGNED", verify.stdout
    assert verify.returncode == 0, verify.stderr

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        png = tmp_path / "page0.png"
        pdf = tmp_path / "out.pdf"

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
            env=pack_verify_env(with_k2f_cli=False),
        )
        assert proc.returncode == 0, proc.stdout + proc.stderr
        assert out.is_file() and out.stat().st_size > 0
        assert "UNSIGNED" in proc.stdout or "UNSIGNED" in proc.stderr


def test_pack_verify_one_page_underfill_is_warning() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        doc = tmp_path / "doc"
        out = tmp_path / "doc.K2F"
        init_proc = _run_init(doc)
        assert init_proc.returncode == 0, init_proc.stderr
        (doc / "content" / "root.json").write_text(
            """{
  "id": "root",
  "role": "document",
  "content": {
    "type": "container",
    "value": {
      "children": [
        {
          "id": "root.title",
          "role": "h1",
          "content": { "type": "text", "value": "Short invoice title" }
        }
      ]
    }
  }
}
""",
            encoding="utf-8",
        )
        pack_proc = subprocess.run(
            [
                sys.executable,
                str(PACK_VERIFY),
                str(doc),
                "-o",
                str(out),
            ],
            capture_output=True,
            text=True,
            check=False,
            env=pack_verify_env(),
        )
        assert pack_proc.returncode == 0, pack_proc.stdout + pack_proc.stderr
        combined = pack_proc.stdout + pack_proc.stderr
        assert "PAGE_UNDERFILL" in combined
        assert out.is_file()


def test_init_package_pack_verify() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        doc = tmp_path / "doc"
        out = tmp_path / "doc.K2F"
        init_proc = subprocess.run(
            [
                sys.executable,
                str(INIT_PACKAGE),
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
            env=pack_verify_env(),
        )
        assert pack_proc.returncode == 0, pack_proc.stdout + pack_proc.stderr
        assert out.is_file() and out.stat().st_size > 0


def _run_init(dest: Path, extra: list[str] | None = None) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        str(INIT_PACKAGE),
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


def test_init_square_keeps_license_and_renders() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        dest = Path(tmp) / "post"
        proc = subprocess.run(
            [
                sys.executable,
                str(INIT_PACKAGE),
                "--dir",
                str(dest),
                "--title",
                "Square post",
                "--page",
                "square",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert proc.returncode == 0, proc.stderr
        assert (dest / "assets" / "fonts" / "licenses" / "Roboto-Apache.txt").is_file()
        page = json.loads((dest / "manifest.json").read_text(encoding="utf-8"))["page_config"]
        assert page["width"] == 600000
        assert page["height"] == 600000
        assert page["margin"] == [0, 0, 0, 0]
        assert "ex_poster_shell.json" in proc.stdout
        out = Path(tmp) / "post.K2F"
        pack_proc = subprocess.run(
            [
                sys.executable,
                str(PACK_VERIFY),
                str(dest),
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
            env=pack_verify_env(),
        )
        combined = pack_proc.stdout + pack_proc.stderr
        assert pack_proc.returncode == 0, combined
        assert "UnknownMagic" not in combined
        preview = out.parent / "tmp" / "preview.png"
        assert preview.is_file() and preview.read_bytes()[:8] == b"\x89PNG\r\n\x1a\n"


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


def test_pack_verify_bare_render_writes_to_workspace_tmp() -> None:
    import importlib.util

    spec = importlib.util.spec_from_file_location("pack_verify", PACK_VERIFY)
    assert spec is not None and spec.loader is not None
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    output = Path("/tmp/elsewhere/doc.K2F")
    assert mod.resolve_render_path(Path("preview.png"), output) == (
        output.parent / "tmp" / "preview.png"
    ).resolve()
    nested = mod.resolve_render_path(Path("out/preview.png"), output)
    assert nested != (output.parent / "tmp" / "preview.png").resolve()
    assert mod.extra_render_path(Path("/tmp/elsewhere/tmp/preview.png"), 1) == Path(
        "/tmp/elsewhere/tmp/preview-1.png"
    )

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        doc = tmp_path / "doc"
        out = tmp_path / "elsewhere" / "doc.K2F"
        init_proc = _run_init(doc)
        assert init_proc.returncode == 0, init_proc.stderr
        (doc / "stray.png").write_bytes(b"not-a-png")
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
            env=pack_verify_env(),
        )
        assert pack_proc.returncode == 0, pack_proc.stdout + pack_proc.stderr
        preview = out.parent / "tmp" / "preview.png"
        assert preview.is_file() and preview.read_bytes()[:8] == b"\x89PNG\r\n\x1a\n"
        assert not (doc / "preview.png").exists()
        assert not (tmp_path / "preview.png").exists()
        assert "stray.png" in pack_proc.stderr


def test_init_package_workspace_preserves_design_md() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        workspace = tmp_path / "report"
        source = workspace / "source"
        source.mkdir(parents=True)
        design = source / "design.md"
        design.write_text("# spec\n", encoding="utf-8")
        proc = subprocess.run(
            [
                sys.executable,
                str(INIT_PACKAGE),
                "--workspace",
                str(workspace),
                "--title",
                "Q3 Report",
                "--page",
                "a4",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert proc.returncode == 0, proc.stderr
        assert design.read_text(encoding="utf-8") == "# spec\n"
        assert (source / "manifest.json").is_file()
        assert (workspace / "tmp").is_dir()
        assert "pack_verify.py" in proc.stdout
        assert str(workspace / "report.K2F") in proc.stdout

        both = subprocess.run(
            [
                sys.executable,
                str(INIT_PACKAGE),
                "--dir",
                str(source),
                "--workspace",
                str(workspace),
                "--title",
                "Nope",
                "--page",
                "a4",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert both.returncode != 0
        assert "exactly one" in both.stderr


def test_workspace_pack_export_layout() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        workspace = tmp_path / "report"
        init_proc = subprocess.run(
            [
                sys.executable,
                str(INIT_PACKAGE),
                "--workspace",
                str(workspace),
                "--title",
                "Layout smoke",
                "--page",
                "a4",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert init_proc.returncode == 0, init_proc.stderr
        source = workspace / "source"
        packed = workspace / "report.K2F"
        pack_proc = subprocess.run(
            [
                sys.executable,
                str(PACK_VERIFY),
                str(source),
                "-o",
                str(packed),
                "--expect-pages",
                "1",
                "--render",
                "preview.png",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=pack_verify_env(),
        )
        assert pack_proc.returncode == 0, pack_proc.stdout + pack_proc.stderr
        assert packed.is_file()
        preview = workspace / "tmp" / "preview.png"
        assert preview.is_file() and preview.read_bytes()[:8] == b"\x89PNG\r\n\x1a\n"
        assert not (source / "preview.png").exists()

        pdf = workspace / "report.pdf"
        docx = workspace / "report.docx"
        pptx = workspace / "report.pptx"
        for cmd, dest in (
            (["export-pdf", str(packed), "-o", str(pdf)], pdf),
            (["export-docx", str(packed), "-o", str(docx)], docx),
            (["export-pptx", str(packed), "-o", str(pptx)], pptx),
        ):
            export = k2f_cmd(*cmd)
            assert export.returncode == 0, export.stderr
            assert dest.is_file() and dest.stat().st_size > 0
        assert pdf.read_bytes().startswith(b"%PDF-")
        assert docx.read_bytes().startswith(b"PK")
        assert pptx.read_bytes().startswith(b"PK")
        assert not (source / "report.K2F").exists()
        assert not (source / "report.pdf").exists()
