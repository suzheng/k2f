#!/usr/bin/env python3
"""Rebuild assets/fonts/NotoEmoji-Regular.ttf from pinned Noto Emoji (monochrome).

Subsets to dingbats/emoji used by in-repo Markdown and the pepkio plan-doc tree.
Static Regular only — no variable fvar in the committed WASM face.
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
DST = ROOT / "assets/fonts/NotoEmoji-Regular.ttf"

# Pinned gstatic Regular (OFL). Update when rebasing Noto Emoji.
PINNED_URL = (
    "https://fonts.gstatic.com/s/notoemoji/v39/"
    "bMrnmSyK7YY-MEu6aWjPDs-ar6uWaGWuob-r0jwvS-FGJCMY.ttf"
)
CACHE = ROOT / "target/font-src/NotoEmoji-Regular-full.ttf"

EXTRA = [
    0x2705,  # ✅
    0x274C,  # ❌
    0x1F4AC,  # 💬
    0x1F534,  # 🔴
    0x1F7E1,  # 🟡
    0x1F7E2,  # 🟢
    0x26A0,  # ⚠ (SC may cover; harmless duplicate)
]

DOC_ROOTS = [
    ROOT / "tests/fixtures/markdown",
    ROOT / "docs/spec",
    ROOT / "docs/guide",
]


def cmap_of(path: Path) -> set[int]:
    font = TTFont(path)
    return set((font.getBestCmap() or {}).keys())


def chars_from(path: Path) -> set[int]:
    text = path.read_text(encoding="utf-8")
    return {ord(ch) for ch in text if ch not in "\n\r\t"}


def md_files(root: Path) -> list[Path]:
    if root.is_file() and root.suffix.lower() == ".md":
        return [root]
    if not root.is_dir():
        return []
    return sorted(p for p in root.rglob("*.md") if p.is_file())


def emoji_candidates(text_cps: set[int], sc_cmap: set[int]) -> set[int]:
    """Codepoints SC lacks that look like emoji/dingbats."""
    out: set[int] = set()
    for cp in text_cps:
        if cp in sc_cmap or cp == 0:
            continue
        if cp >= 0x1F000 or (0x2600 <= cp <= 0x27BF) or cp in EXTRA:
            out.add(cp)
    out.update(EXTRA)
    return out


def download_full(dest: Path) -> Path:
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists():
        return dest
    print(f"downloading {PINNED_URL}", file=sys.stderr)
    urllib.request.urlretrieve(PINNED_URL, dest)
    return dest


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--src", type=Path, help="full NotoEmoji-Regular.ttf")
    parser.add_argument("roots", nargs="*", type=Path, help="extra Markdown roots")
    args = parser.parse_args()

    sc_path = ROOT / "assets/fonts/NotoSansSC-Regular.otf"
    sc_cmap = cmap_of(sc_path) if sc_path.exists() else set()

    needed: set[int] = set(EXTRA)
    for root in args.roots or DOC_ROOTS:
        for path in md_files(root):
            needed.update(emoji_candidates(chars_from(path), sc_cmap))

    src = args.src or download_full(CACHE)
    if not src.exists():
        raise SystemExit(f"missing source font {src}")

    src_cmap = cmap_of(src)
    missing = sorted(cp for cp in needed if cp not in src_cmap)
    if missing:
        preview = ", ".join(f"U+{cp:04X}" for cp in missing[:16])
        raise SystemExit(f"source lacks {len(missing)} codepoints: {preview}")

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

    size = DST.stat().st_size
    n = len(cmap_of(DST))
    print(f"wrote {DST} ({size} bytes, {n} cmap glyphs)")
    if size > 500_000:
        raise SystemExit(f"subset {size} bytes exceeds 500KB target")


if __name__ == "__main__":
    main()
