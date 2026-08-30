#!/usr/bin/env python3
"""Rebuild assets/fonts/NotoSansSC-Regular.otf from a full Noto Sans SC face.

Keeps the current cmap, adds Basic Latin, punctuation/arrows used by Markdown,
and every character in the Markdown trees that the full SC face can provide.
Codepoints the source font lacks (emoji, etc.) are skipped — those go through
the compile-time catalog, not this committed WASM subset.

Requires: fonttools (pyftsubset), a full NotoSansSC-Regular.otf
          (downloaded automatically unless --src is set).
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path

from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[1]
DST = ROOT / "assets/fonts/NotoSansSC-Regular.otf"
CONTRACT_DST = ROOT / "examples/contract/assets/fonts/NotoSansSC-Regular.otf"

# Pinned region OTF (Sans 2.004). We subset further for the committed face.
PINNED_ZIP = (
    "https://github.com/notofonts/noto-cjk/releases/download/"
    "Sans2.004/18_NotoSansSC.zip"
)

EXTRA = (
    list(range(0x20, 0x7F))
    + [
        0x00A0,  # NBSP
        0x00E9,  # é
        0x2011,  # ‑
        0x2013,  # –
        0x2014,  # —
        0x2022,  # •
        0x2026,  # …
        0x2190,  # ←
        0x2191,  # ↑
        0x2192,  # →
        0x2193,  # ↓
        0x2194,  # ↔
        0x221E,  # ∞
        0x2260,  # ≠
        0x2265,  # ≥
        0x25BC,  # ▼
        0x26A0,  # ⚠
        0x2713,  # ✓
        0x3000,  # ideographic space
        0x3001,  # 、
        0x3002,  # 。
        0x300C,  # 「
        0x300D,  # 」
        0x300E,  # 『
        0x300F,  # 』
        0x3010,  # 【
        0x3011,  # 】
    ]
    + list(range(0x2500, 0x257F + 1))  # box drawing
)

DOC_ROOTS = [
    ROOT / "tests/fixtures/markdown",
    ROOT / "docs/spec",
    ROOT / "docs/guide",
]


def cmap_of(path: Path) -> set[int]:
    font = TTFont(path)
    cmap = font.getBestCmap() or {}
    return set(cmap.keys())


def chars_from(path: Path) -> set[int]:
    text = path.read_text(encoding="utf-8")
    return {ord(ch) for ch in text if ch not in "\n\r\t"}


def md_files(root: Path) -> list[Path]:
    if root.is_file() and root.suffix.lower() == ".md":
        return [root]
    if not root.is_dir():
        return []
    return sorted(p for p in root.rglob("*.md") if p.is_file())


def download_full(dest: Path) -> Path:
    dest.parent.mkdir(parents=True, exist_ok=True)
    print(f"downloading {PINNED_ZIP}", file=sys.stderr)
    zip_path = dest.with_suffix(".zip")
    urllib.request.urlretrieve(PINNED_ZIP, zip_path)
    import zipfile

    with zipfile.ZipFile(zip_path) as zf:
        names = [
            n
            for n in zf.namelist()
            if n.endswith("NotoSansSC-Regular.otf") or n.endswith("NotoSansCJKsc-Regular.otf")
        ]
        if not names:
            raise SystemExit(f"zip has no SC Regular otf: {zf.namelist()[:20]}")
        names.sort(key=len)
        member = names[0]
        print(f"extracting {member}", file=sys.stderr)
        dest.write_bytes(zf.read(member))
    zip_path.unlink(missing_ok=True)
    return dest


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--src", type=Path, help="full NotoSansSC-Regular.otf")
    parser.add_argument("roots", nargs="*", type=Path, help="Markdown roots to union")
    args = parser.parse_args()

    existing = cmap_of(DST) if DST.exists() else set()
    needed: set[int] = set(existing)
    needed.update(EXTRA)
    roots = args.roots or DOC_ROOTS
    for root in roots:
        files = md_files(root)
        if not files:
            print(f"skip missing root {root}", file=sys.stderr)
            continue
        for path in files:
            needed.update(chars_from(path))
    needed.discard(0)

    src = args.src
    if src is None:
        src = ROOT / "target/font-src/NotoSansSC-Regular-full.otf"
        if not src.exists():
            download_full(src)
    if not src.exists():
        raise SystemExit(f"missing source font {src}")

    src_cmap = cmap_of(src)
    missing_in_src = sorted(cp for cp in needed if cp not in src_cmap)
    if missing_in_src:
        preview = ", ".join(f"U+{cp:04X} {chr(cp)!r}" for cp in missing_in_src[:24])
        print(
            f"skip {len(missing_in_src)} codepoints not in SC source (catalog): {preview}",
            file=sys.stderr,
        )
        needed -= set(missing_in_src)

    with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as fh:
        uni_path = Path(fh.name)
        for cp in sorted(needed):
            fh.write(f"U+{cp:04X}\n")

    cmd = [
        sys.executable,
        "-m",
        "fontTools.subset",
        str(src),
        f"--unicodes-file={uni_path}",
        f"--output-file={DST}",
        "--glyph-names",
        "--notdef-glyph",
        "--notdef-outline",
        "--recommended-glyphs",
        "--layout-features=*",
        "--name-IDs=*",
        "--name-legacy",
        "--name-languages=*",
    ]
    print(" ".join(cmd), file=sys.stderr)
    subprocess.check_call(cmd)
    uni_path.unlink(missing_ok=True)

    CONTRACT_DST.parent.mkdir(parents=True, exist_ok=True)
    CONTRACT_DST.write_bytes(DST.read_bytes())
    size = DST.stat().st_size
    n = len(cmap_of(DST))
    print(f"wrote {DST} ({size} bytes, {n} cmap glyphs)")
    print(f"copied {CONTRACT_DST}")
    if size > 1_500_000:
        raise SystemExit(f"subset {size} bytes exceeds 1.5MB target")


if __name__ == "__main__":
    main()
