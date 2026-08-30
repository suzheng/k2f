#!/usr/bin/env python3
"""Batch-convert Markdown trees to .K2F using the SDK font stack or a catalog face.

For each .md file: if the committed Noto Sans SC + Noto Emoji companion covers every
character, call `k2f markdown` (SDK embeds both). Otherwise subset the first catalog
font that covers 100% and pass `--font` for the primary face. Uncovered codepoints are
printed and counted as failures. No system-font fallback.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
import urllib.request
import zipfile
from pathlib import Path

from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[1]
OFFICIAL = ROOT / "assets/fonts/NotoSansSC-Regular.otf"
EMOJI = ROOT / "assets/fonts/NotoEmoji-Regular.ttf"
CATALOG_PATH = ROOT / "scripts/font-catalog.json"
DEFAULT_OUT = ROOT / "target/md-k2f"


def cmap_of(path: Path) -> set[int]:
    font = TTFont(path)
    cmap = font.getBestCmap() or {}
    return set(cmap.keys())


def chars_from(text: str) -> set[int]:
    return {ord(ch) for ch in text if ch not in "\n\r\t"}


def md_files(root: Path) -> list[Path]:
    if root.is_file() and root.suffix.lower() == ".md":
        return [root]
    if not root.is_dir():
        return []
    return sorted(p for p in root.rglob("*.md") if p.is_file())


def fmt_cp(cp: int) -> str:
    return f"U+{cp:04X} {chr(cp)!r}"


def load_catalog() -> list[dict]:
    data = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
    return list(data["fonts"])


def ensure_font(entry: dict) -> Path:
    dest = ROOT / entry["cache"]
    if dest.exists():
        return dest
    dest.parent.mkdir(parents=True, exist_ok=True)
    url = entry["url"]
    print(f"downloading {url}", file=sys.stderr)
    if url.lower().endswith(".zip"):
        zip_path = dest.with_suffix(".zip")
        urllib.request.urlretrieve(url, zip_path)
        suffixes = tuple(entry.get("zip_member_suffix") or [dest.name])
        with zipfile.ZipFile(zip_path) as zf:
            names = [
                n
                for n in zf.namelist()
                if any(n.endswith(suf) for suf in suffixes) and not n.endswith("/")
            ]
            if not names:
                raise SystemExit(f"{entry['id']} zip has no matching member: {zf.namelist()[:20]}")
            names.sort(key=len)
            dest.write_bytes(zf.read(names[0]))
        zip_path.unlink(missing_ok=True)
    else:
        urllib.request.urlretrieve(url, dest)
    return dest


def subset(src: Path, needed: set[int], dest: Path) -> None:
    uni = dest.with_suffix(".unicodes.txt")
    uni.write_text("".join(f"U+{cp:04X}\n" for cp in sorted(needed)), encoding="utf-8")
    cmd = [
        sys.executable,
        "-m",
        "fontTools.subset",
        str(src),
        f"--unicodes-file={uni}",
        f"--output-file={dest}",
        "--glyph-names",
        "--notdef-glyph",
        "--notdef-outline",
        "--recommended-glyphs",
        "--layout-features=*",
        "--name-IDs=*",
        "--name-legacy",
        "--name-languages=*",
    ]
    subprocess.check_call(cmd)
    uni.unlink(missing_ok=True)


def find_k2f() -> Path:
    subprocess.check_call(["cargo", "build", "-p", "k2f", "-q"], cwd=ROOT)
    debug = ROOT / "target/debug/k2f"
    if not debug.is_file():
        raise SystemExit("k2f binary missing after cargo build -p k2f")
    return debug


def convert(
    k2f: Path,
    md_path: Path,
    dest: Path,
    theme: str,
    font: Path | None,
) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    cmd = [str(k2f), "markdown", str(md_path), "-o", str(dest), "--theme", theme]
    if font is not None:
        cmd.extend(["--font", str(font)])
    subprocess.check_call(cmd)


def dest_for(root: Path, md_path: Path, out: Path) -> Path:
    if root.is_file():
        return out / (root.stem + ".K2F")
    rel = md_path.relative_to(root).with_suffix(".K2F")
    return out / root.name / rel


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("roots", nargs="+", type=Path)
    parser.add_argument("-o", "--output", type=Path, default=DEFAULT_OUT)
    parser.add_argument("--theme", default="report")
    args = parser.parse_args()

    official = cmap_of(OFFICIAL)
    if EMOJI.exists():
        official |= cmap_of(EMOJI)
    catalog = load_catalog()
    k2f = find_k2f()
    failed = 0
    ok = 0

    with tempfile.TemporaryDirectory(prefix="k2f-md-font-") as tmp:
        tmp_dir = Path(tmp)
        for root in args.roots:
            root = root.resolve()
            files = md_files(root)
            if not files:
                print(f"skip missing root {root}", file=sys.stderr)
                continue
            for md_path in files:
                dest = dest_for(root, md_path, args.output)
                text = md_path.read_text(encoding="utf-8")
                needed = chars_from(text)
                needed.discard(0)
                missing_official = sorted(cp for cp in needed if cp not in official)
                try:
                    if not missing_official:
                        convert(k2f, md_path, dest, args.theme, None)
                        print(f"ok  {md_path} -> {dest}")
                        ok += 1
                        continue
                    font_path = None
                    for entry in catalog:
                        src = ensure_font(entry)
                        cmap = cmap_of(src)
                        if all(cp in cmap for cp in needed):
                            cut = tmp_dir / f"{md_path.stem}-{entry['id']}.otf"
                            subset(src, needed, cut)
                            font_path = cut
                            break
                    if font_path is None:
                        preview = ", ".join(fmt_cp(cp) for cp in missing_official[:24])
                        extra = "" if len(missing_official) <= 24 else f" … +{len(missing_official) - 24}"
                        print(f"fail {md_path} uncovered n={len(missing_official)} {preview}{extra}")
                        failed += 1
                        continue
                    convert(k2f, md_path, dest, args.theme, font_path)
                    print(f"ok  {md_path} -> {dest} font={font_path.name}")
                    ok += 1
                except subprocess.CalledProcessError as e:
                    print(f"fail {md_path} convert: {e}")
                    failed += 1

    print(f"converted={ok} failed={failed} out={args.output}", file=sys.stderr)
    raise SystemExit(1 if failed else 0)


if __name__ == "__main__":
    main()
