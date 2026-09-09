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

# Width × height in millipt. Print 56pt margins; slides 36pt; social/card 0.
PAGES = {
    "a4": {"width": 595000, "height": 842000, "margin": [56000, 56000, 56000, 56000]},
    "letter": {"width": 612000, "height": 792000, "margin": [56000, 56000, 56000, 56000]},
    "a4-landscape": {"width": 842000, "height": 595000, "margin": [56000, 56000, 56000, 56000]},
    "widescreen": {"width": 960000, "height": 540000, "margin": [36000, 36000, 36000, 36000]},
    "widescreen-43": {"width": 720000, "height": 540000, "margin": [36000, 36000, 36000, 36000]},
    "square": {"width": 600000, "height": 600000, "margin": [0, 0, 0, 0]},
    "portrait-45": {"width": 600000, "height": 750000, "margin": [0, 0, 0, 0]},
    "card": {"width": 252000, "height": 144000, "margin": [0, 0, 0, 0]},
}

SKILL_ROOT = Path(__file__).resolve().parents[1]
STARTER = SKILL_ROOT / "starter"
# Files that mean dest is already an author package — do not overwrite.
PACKAGE_FILE_MARKERS = frozenset({"manifest.json", "changelog.json"})
# Scaffold dirs: empty is OK (design-first mkdir); non-empty means a package.
PACKAGE_DIR_MARKERS = frozenset({"content", "styles", "assets"})


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


def replace_font(out: Path, font: Path) -> str | None:
    fonts_dir = out / "assets" / "fonts"
    for old in fonts_dir.glob("*.ttf"):
        old.unlink()
    for old in fonts_dir.glob("*.otf"):
        old.unlink()
    dest = fonts_dir / font.name
    err = copy_font_file(font, dest)
    if err:
        return err
    patch_theme_primary_font(out, font.stem)
    return None


def add_fonts(out: Path, fonts: list[Path]) -> str | None:
    fonts_dir = out / "assets" / "fonts"
    fonts_dir.mkdir(parents=True, exist_ok=True)
    stems: list[str] = []
    for font in fonts:
        dest = fonts_dir / font.name
        if dest.exists() and dest.resolve() != font.resolve():
            dest.unlink()
        if dest.resolve() != font.resolve():
            err = copy_font_file(font, dest)
            if err:
                return err
        stems.append(font.stem)
    patch_theme_add_font_aliases(out, stems)
    return None


def patch_theme_primary_font(out: Path, new_stem: str) -> None:
    """Roles still named Roboto-Regular after --font would FONT_MISSING."""
    path = out / "styles" / "theme.json"
    theme = json.loads(path.read_text(encoding="utf-8"))
    old_primary = "Roboto-Regular"
    theme["font_aliases"] = {new_stem: new_stem}
    for role in theme.get("roles", {}).values():
        if role.get("font_family") in (old_primary, "default"):
            role["font_family"] = new_stem
        for variant in (role.get("variants") or {}).values():
            overrides = variant.get("text_overrides") or {}
            if overrides.get("font_family") in (old_primary, "default"):
                overrides["font_family"] = new_stem
    path.write_text(json.dumps(theme, indent=2) + "\n", encoding="utf-8")


def patch_theme_add_font_aliases(out: Path, stems: list[str]) -> None:
    path = out / "styles" / "theme.json"
    theme = json.loads(path.read_text(encoding="utf-8"))
    aliases = theme.setdefault("font_aliases", {})
    for stem in stems:
        aliases[stem] = stem
    theme["font_aliases"] = aliases
    path.write_text(json.dumps(theme, indent=2) + "\n", encoding="utf-8")


def require_font_file(font: Path) -> str | None:
    if not font.is_file():
        return f"font not found: {font}"
    if font.suffix.lower() not in {".ttf", ".otf"}:
        return f"expected .ttf or .otf, got {font.suffix}"
    return None


def copy_font_file(src: Path, dest: Path) -> str | None:
    """Copy one face into the package. Fail closed — K2F cannot use OS fonts at render."""
    try:
        shutil.copy(src, dest)
    except OSError as e:
        return (
            f"cannot copy font {src}: {e}\n"
            "K2F embeds fonts in the package (no system-font fallback at render). "
            "Copy the .ttf/.otf to a path you can read, then pass --font/--add-font. "
            "Do not point at locked OS font dirs (e.g. /System/Library/Fonts)."
        )
    return None


def dir_names(path: Path) -> set[str]:
    return {p.name for p in path.iterdir()}


def drop_empty_scaffold_dirs(out: Path) -> None:
    """Remove empty content/styles/assets so copytree can recreate them.

    Design-first `mkdir` of those names is not a package. Non-empty dirs stay
    and `prepare_dest` will refuse. Dropping empties keeps failure cleanup
    accurate (preexisting snapshot would otherwise keep a dir we filled).
    """
    for name in PACKAGE_DIR_MARKERS:
        path = out / name
        if path.is_dir() and not any(path.iterdir()):
            path.rmdir()


def blocking_markers(out: Path, names: set[str]) -> list[str]:
    blocked: list[str] = []
    for name in sorted(names & PACKAGE_FILE_MARKERS):
        if (out / name).is_file():
            blocked.append(name)
    for name in sorted(names & PACKAGE_DIR_MARKERS):
        path = out / name
        if path.is_file() or (path.is_dir() and any(path.iterdir())):
            blocked.append(name)
    return blocked


def prepare_dest(out: Path) -> set[str] | None:
    """Return preexisting names when reusing a notes-only dir; None if `out` is created.

    Empty dirs and sidecar-only dirs (e.g. design.md inside the author dir) are OK.
    Refuse files and directories that already look like a K2F author package.
    """
    if not out.exists():
        return None
    if not out.is_dir():
        raise DestError(f"error: --dir is a file: {out}")
    drop_empty_scaffold_dirs(out)
    names = dir_names(out)
    markers = blocking_markers(out, names)
    if markers:
        listed = ", ".join(markers)
        raise DestError(
            f"error: exists as a package ({listed}): {out}\n"
            "Edit JSON in place, or choose another --dir / --workspace. "
            "Write design.md inside the author directory (e.g. ./out/doc/source/design.md). "
            "Notes-only dirs and empty content/styles/assets are OK. There is no --force."
        )
    return names


def cleanup_init(out: Path, preexisting: set[str] | None) -> None:
    """On failure: delete a directory we created; otherwise remove only starter files."""
    if preexisting is None:
        if out.exists():
            shutil.rmtree(out)
        return
    if not out.is_dir():
        return
    for child in list(out.iterdir()):
        if child.name in preexisting:
            continue
        if child.is_dir() and not child.is_symlink():
            shutil.rmtree(child)
        else:
            child.unlink()


class DestError(Exception):
    pass


def main() -> int:
    parser = argparse.ArgumentParser(description="Init an unpacked K2F package from skill starter/")
    parser.add_argument(
        "--dir",
        type=Path,
        default=None,
        help="Author package directory (manifest.json, content/, styles/, assets/). "
        "Prefer --workspace; use --dir for an existing unpacked tree.",
    )
    parser.add_argument(
        "--workspace",
        type=Path,
        default=None,
        help="Create PATH/source (author package) and PATH/tmp. "
        "Deliverables (.K2F, PDF, DOCX, PPTX) go in PATH. Mutually exclusive with --dir.",
    )
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
        "mm→millipt: round(mm * 72000 / 254). Prefer a --page preset when one fits "
        "(square, portrait-45, card).",
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
        help="Replace bundled Roboto with this covering TTF/OTF and retarget theme roles "
        "(readable .ttf/.otf you can copy — not a locked OS font dir; "
        "use a CJK face that includes Latin, e.g. Noto Sans JP)",
    )
    parser.add_argument(
        "--add-font",
        type=Path,
        action="append",
        default=[],
        metavar="TTF",
        help="Keep Roboto and copy this TTF/OTF for glyph fallback (repeatable). "
        "Japanese/Korean/math: --add-font covering.otf (NotoSansSC is Simplified Chinese, not kana)",
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

    if (args.dir is None) == (args.workspace is None):
        print("error: pass exactly one of --dir or --workspace", file=sys.stderr)
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

    workspace: Path | None = None
    if args.workspace is not None:
        workspace = args.workspace.expanduser().resolve()
        if workspace.exists() and not workspace.is_dir():
            print(f"error: --workspace is a file: {workspace}", file=sys.stderr)
            return 1
        workspace.mkdir(parents=True, exist_ok=True)
        (workspace / "tmp").mkdir(exist_ok=True)
        out = workspace / "source"
    else:
        assert args.dir is not None
        out = args.dir.expanduser().resolve()
    try:
        preexisting = prepare_dest(out)
    except DestError as e:
        print(str(e), file=sys.stderr)
        return 1

    shutil.copytree(STARTER, out, dirs_exist_ok=True)

    def fail(msg: str) -> int:
        print(msg, file=sys.stderr)
        cleanup_init(out, preexisting)
        return 1

    if args.author:
        manifest_path = out / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["author"] = args.author
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    patch_manifest(out, args.title, args.page, margin, args.width, args.height)

    extra_fonts: list[Path] = []
    for raw in args.add_font:
        extra = raw.expanduser().resolve()
        err = require_font_file(extra)
        if err:
            return fail(f"error: --add-font: {err}")
        extra_fonts.append(extra)

    if args.font is not None:
        font = args.font.expanduser().resolve()
        err = require_font_file(font)
        if err:
            return fail(
                f"error: {err}\nPass --font to a TTF/OTF, or reinstall the skill starter bundle."
            )
        err = replace_font(out, font)
        if err:
            return fail(f"error: --font: {err}")
    else:
        font = default_font.expanduser().resolve()
        if not font.is_file():
            return fail(
                f"error: bundled Roboto missing: {font}\nReinstall the skill starter bundle."
            )

    if extra_fonts:
        err = add_fonts(out, extra_fonts)
        if err:
            return fail(f"error: --add-font: {err}")

    page = PAGES[args.page]
    width = args.width if args.width is not None else page["width"]
    height = args.height if args.height is not None else page["height"]
    applied_margin = margin if margin is not None else list(page["margin"])

    print(f"initialized {out} from starter/")
    size_note = f"page={args.page}"
    if args.width is not None:
        size_note += " (custom size)"
    print(f"  title={args.title!r}  {size_note}  {width}x{height}  margin={applied_margin}")
    if extra_fonts:
        print("  extra fonts (glyph fallback): " + ", ".join(p.name for p in extra_fonts))
    if args.page in ("widescreen", "widescreen-43") and applied_margin != [0, 0, 0, 0]:
        print(
            "  tip: for full-bleed slide decks use --margin 0 + catalog/content/ex_poster_shell.json "
            "(role page_shell is the safe inset; set layout.height to 540000 — see writing/package.md)"
        )
    if args.page in ("square", "portrait-45", "card"):
        print(
            "  tip: copy catalog/content/ex_poster_shell.json "
            f"(role page_shell; set layout.height to {height} — see writing/package.md)"
        )
    if args.page == "card":
        print(
            "  tip: duplex = two page-height shells; back also break_before=page; "
            "--expect-pages 2 (see writing/package.md)"
        )
    if args.width is not None and applied_margin != [0, 0, 0, 0]:
        print(
            "  tip: full-bleed posters often use --margin 0 + catalog/content/ex_poster_shell.json "
            "(role page_shell; set layout.height to page height — see writing/package.md)"
        )
    print(
        "Next: edit content/root.json (+ optional content/*.json includes) and styles/theme.json; "
        "copy shapes from catalog/content/ex_*.json; then pack_verify.py"
    )
    if workspace is not None:
        stem = workspace.name
        packed = workspace / f"{stem}.K2F"
        print(
            f"  python scripts/pack_verify.py {out} -o {packed} --render preview.png"
        )
        print(f"  preview → {workspace / 'tmp' / 'preview.png'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
