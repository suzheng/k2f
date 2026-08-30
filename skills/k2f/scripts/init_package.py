#!/usr/bin/env python3
"""Create a new unpacked K2F package from the skill starter template.

Copies starter/ (empty tree + core theme + bundled Roboto). Does not copy
format schemas — `k2f pack` injects them. Patch title/page/margin after copy.
"""

from __future__ import annotations

import argparse
import json
import shutil
import sys
from pathlib import Path

# Width × height in millipt. Print pages default to 56pt margins; slide sizes 36pt.
PAGES = {
    "a4": {"width": 595000, "height": 842000, "margin": [56000, 56000, 56000, 56000]},
    "letter": {"width": 612000, "height": 792000, "margin": [56000, 56000, 56000, 56000]},
    "a4-landscape": {"width": 842000, "height": 595000, "margin": [56000, 56000, 56000, 56000]},
    "widescreen": {"width": 960000, "height": 540000, "margin": [36000, 36000, 36000, 36000]},
    "widescreen-43": {"width": 720000, "height": 540000, "margin": [36000, 36000, 36000, 36000]},
}

SKILL_ROOT = Path(__file__).resolve().parents[1]
STARTER = SKILL_ROOT / "starter"


def parse_margin(raw: str) -> list[int]:
    """Parse millipt margin: one value (all sides) or four comma-separated (top,right,bottom,left)."""
    parts = [p.strip() for p in raw.split(",") if p.strip()]
    if len(parts) == 1:
        v = int(parts[0])
        if v < 0:
            raise ValueError("margin must be >= 0")
        return [v, v, v, v]
    if len(parts) == 4:
        vals = [int(p) for p in parts]
        if any(v < 0 for v in vals):
            raise ValueError("margin must be >= 0")
        return vals
    raise ValueError("expected 1 value or 4 comma-separated millipt values (top,right,bottom,left)")


def patch_manifest(
    out: Path,
    title: str,
    page_key: str,
    margin: list[int] | None,
    width: int | None,
    height: int | None,
) -> None:
    manifest_path = out / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["title"] = title
    page = PAGES[page_key]
    manifest.setdefault("page_config", {})
    manifest["page_config"]["width"] = width if width is not None else page["width"]
    manifest["page_config"]["height"] = height if height is not None else page["height"]
    manifest["page_config"]["margin"] = margin if margin is not None else list(page["margin"])
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def replace_font(out: Path, font: Path) -> None:
    fonts_dir = out / "assets" / "fonts"
    for old in fonts_dir.glob("*.ttf"):
        old.unlink()
    for old in fonts_dir.glob("*.otf"):
        old.unlink()
    dest = fonts_dir / font.name
    shutil.copy2(font, dest)


def main() -> int:
    parser = argparse.ArgumentParser(description="Init an unpacked K2F package from skill starter/")
    parser.add_argument("--dir", required=True, type=Path, help="Output package directory")
    parser.add_argument("--title", required=True, help="Document title")
    parser.add_argument(
        "--page",
        required=True,
        choices=sorted(PAGES.keys()),
        help="Page size preset (canvas_mode stays paged). Override with --width/--height.",
    )
    parser.add_argument(
        "--width",
        type=int,
        default=None,
        metavar="MILLIPT",
        help="Override page_config.width in millipt (must pair with --height). "
        "mm→millipt: round(mm * 72000 / 254)",
    )
    parser.add_argument(
        "--height",
        type=int,
        default=None,
        metavar="MILLIPT",
        help="Override page_config.height in millipt (must pair with --width)",
    )
    parser.add_argument(
        "--margin",
        default=None,
        help="Override page_config.margin in millipt: 36000 or 36000,48000,36000,48000 (top,right,bottom,left)",
    )
    default_font = STARTER / "assets" / "fonts" / "Roboto-Regular.ttf"
    parser.add_argument(
        "--font",
        type=Path,
        default=None,
        help="Replace bundled Roboto with this TTF/OTF (default: starter/assets/fonts/Roboto-Regular.ttf)",
    )
    parser.add_argument(
        "--author",
        default="",
        help="Optional author string for manifest",
    )
    args = parser.parse_args()

    if not STARTER.is_dir():
        print(f"error: starter template missing: {STARTER}", file=sys.stderr)
        return 1

    if (args.width is None) ^ (args.height is None):
        print("error: --width and --height must be set together", file=sys.stderr)
        return 1
    if args.width is not None and (args.width < 1 or args.height < 1):
        print("error: --width/--height must be >= 1 millipt", file=sys.stderr)
        return 1

    margin = None
    if args.margin is not None:
        try:
            margin = parse_margin(args.margin)
        except ValueError as e:
            print(f"error: --margin: {e}", file=sys.stderr)
            return 1

    out = args.dir.expanduser().resolve()
    if out.exists():
        print(f"error: exists (remove or choose another path): {out}", file=sys.stderr)
        return 1

    shutil.copytree(STARTER, out)

    if args.author:
        manifest_path = out / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["author"] = args.author
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    patch_manifest(out, args.title, args.page, margin, args.width, args.height)

    font = (args.font or default_font).expanduser().resolve()
    if not font.is_file():
        print(
            f"error: font not found: {font}\n"
            "Pass --font to a TTF/OTF, or reinstall the skill starter bundle.",
            file=sys.stderr,
        )
        shutil.rmtree(out)
        return 1
    if font.suffix.lower() not in {".ttf", ".otf"}:
        print(f"error: expected .ttf or .otf, got {font.suffix}", file=sys.stderr)
        shutil.rmtree(out)
        return 1
    if font != default_font:
        replace_font(out, font)

    page = PAGES[args.page]
    width = args.width if args.width is not None else page["width"]
    height = args.height if args.height is not None else page["height"]
    applied_margin = margin if margin is not None else list(page["margin"])

    print(f"initialized {out} from starter/")
    size_note = f"page={args.page}"
    if args.width is not None:
        size_note += " (custom size)"
    print(f"  title={args.title!r}  {size_note}  {width}x{height}  margin={applied_margin}")
    if args.page in ("widescreen", "widescreen-43") and applied_margin != [0, 0, 0, 0]:
        print(
            "  tip: for full-bleed slide decks use --margin 0, then inner role padding_pt "
            "+ per-slide layout height + break_inside avoid (see writing/package.md)"
        )
    if args.width is not None and applied_margin != [0, 0, 0, 0]:
        print(
            "  tip: full-bleed posters often use --margin 0 + one fixed-height child "
            "(see single-page poster recipe in writing/package.md)"
        )
    print(
        "Next: edit content/root.json (+ optional content/*.json includes) and styles/theme.json; "
        "copy shapes from catalog/content/ex_*.json; then pack_verify.py"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
