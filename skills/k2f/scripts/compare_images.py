#!/usr/bin/env python3
# /// script
# requires-python = ">=3.9"
# dependencies = [
#     "pillow>=10.0.0",
#     "numpy>=1.22.0",
# ]
# ///
"""
compare_images.py — stack a reference image on a K2F render to spot layout drift.

Blends the two PNGs into a semi-transparent overlay and writes a diff heatmap.
Use after pack_verify.py --render when the user supplied a reference image.

Agent defaults (override with CLI flags):
  --align fit              preserve aspect ratio; letterbox to reference size
  -o ./out/doc/tmp/compare  output prefix → compare_diff.png & compare_overlay.png

Run from the skill directory (needs Pillow + NumPy; PEP 723 deps — use uv run):
  uv run scripts/compare_images.py reference.png ./out/doc/tmp/preview.png
  uv run scripts/compare_images.py ref.png render.png -o ./out/doc/tmp/compare --align fit
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Tuple

# Defaults agents see when skimming the file head (also argparse defaults).
DEFAULT_ALIGN = "fit"
DEFAULT_OUTPUT = "tmp/compare"
DEFAULT_ALPHA = 0.5
DEFAULT_THRESHOLD = 15

try:
    import numpy as np
    from PIL import Image
except ImportError:
    print(
        "error: Pillow and NumPy are required.\n"
        f"Run: uv run {Path(__file__).resolve()} <reference.png> <render.png>",
        file=sys.stderr,
    )
    raise SystemExit(1)


def load_and_normalize_images(
    template_path: str | Path,
    generated_path: str | Path,
    align_mode: str = DEFAULT_ALIGN,
) -> Tuple[Image.Image, Image.Image, dict]:
    """Load both images and normalize them to the same dimensions."""
    template_path = Path(template_path)
    generated_path = Path(generated_path)

    if not template_path.exists():
        raise FileNotFoundError(f"Template image not found: {template_path}")
    if not generated_path.exists():
        raise FileNotFoundError(f"Generated image not found: {generated_path}")

    img_tpl_raw = Image.open(template_path).convert("RGBA")
    img_gen_raw = Image.open(generated_path).convert("RGBA")

    info = {
        "template_path": str(template_path),
        "generated_path": str(generated_path),
        "template_size": img_tpl_raw.size,
        "generated_size": img_gen_raw.size,
        "align_mode": align_mode,
    }

    tw, th = img_tpl_raw.size
    gw, gh = img_gen_raw.size

    if img_tpl_raw.size == img_gen_raw.size:
        img_tpl = img_tpl_raw
        img_gen = img_gen_raw
        info["final_size"] = img_tpl.size
        return img_tpl, img_gen, info

    if align_mode == "resize":
        img_tpl = img_tpl_raw
        img_gen = img_gen_raw.resize((tw, th), Image.Resampling.LANCZOS)
        info["final_size"] = (tw, th)
    elif align_mode == "fit":
        scale = min(tw / gw, th / gh)
        nw, nh = int(round(gw * scale)), int(round(gh * scale))
        resized_gen = img_gen_raw.resize((nw, nh), Image.Resampling.LANCZOS)

        img_gen = Image.new("RGBA", (tw, th), (255, 255, 255, 0))
        offset = ((tw - nw) // 2, (th - nh) // 2)
        img_gen.paste(resized_gen, offset)
        img_tpl = img_tpl_raw
        info["final_size"] = (tw, th)
    elif align_mode == "pad":
        max_w = max(tw, gw)
        max_h = max(th, gh)

        canvas_tpl = Image.new("RGBA", (max_w, max_h), (255, 255, 255, 0))
        canvas_tpl.paste(img_tpl_raw, ((max_w - tw) // 2, (max_h - th) // 2))

        canvas_gen = Image.new("RGBA", (max_w, max_h), (255, 255, 255, 0))
        canvas_gen.paste(img_gen_raw, ((max_w - gw) // 2, (max_h - gh) // 2))

        img_tpl = canvas_tpl
        img_gen = canvas_gen
        info["final_size"] = (max_w, max_h)
    else:
        min_w = min(tw, gw)
        min_h = min(th, gh)
        img_tpl = img_tpl_raw.crop((0, 0, min_w, min_h))
        img_gen = img_gen_raw.crop((0, 0, min_w, min_h))
        info["final_size"] = (min_w, min_h)

    return img_tpl, img_gen, info


def create_overlay_image(
    img_tpl: Image.Image,
    img_gen: Image.Image,
    alpha: float = DEFAULT_ALPHA,
    bg_color: tuple[int, int, int] = (255, 255, 255),
) -> Image.Image:
    """Create a semi-transparent overlay blending reference and render."""
    tpl_flat = Image.new("RGB", img_tpl.size, bg_color)
    tpl_flat.paste(img_tpl, mask=img_tpl.split()[3])

    gen_flat = Image.new("RGB", img_gen.size, bg_color)
    gen_flat.paste(img_gen, mask=img_gen.split()[3])

    return Image.blend(tpl_flat, gen_flat, alpha)


def create_diff_image(
    img_tpl: Image.Image,
    img_gen: Image.Image,
    threshold: int = DEFAULT_THRESHOLD,
    highlight_color: tuple[int, int, int] = (255, 0, 110),
) -> Tuple[Image.Image, float]:
    """Diff heatmap; identical pixels stay faint gray, mismatches highlight."""
    tpl_rgb = np.array(img_tpl.convert("RGB"), dtype=np.float32)
    gen_rgb = np.array(img_gen.convert("RGB"), dtype=np.float32)

    diff = np.abs(tpl_rgb - gen_rgb)
    diff_magnitude = np.max(diff, axis=-1)

    mask_diff = diff_magnitude > threshold
    diff_pixel_ratio = float(np.mean(mask_diff)) * 100.0

    gray_tpl = np.array(img_tpl.convert("L"), dtype=np.float32)
    base = np.stack([gray_tpl * 0.7 + 76] * 3, axis=-1).astype(np.uint8)

    result = base.copy()
    result[mask_diff] = highlight_color

    return Image.fromarray(result, mode="RGB"), diff_pixel_ratio


def main() -> None:
    parser = argparse.ArgumentParser(
        description=(
            "Stack a reference PNG on a K2F render (overlay + diff). "
            "First arg = reference/mockup; second = pack_verify preview."
        )
    )
    parser.add_argument("template", help="Reference / mockup image (first layer)")
    parser.add_argument("generated", help="K2F render to compare (second layer)")
    parser.add_argument(
        "-o",
        "--output",
        default=DEFAULT_OUTPUT,
        help=f"Output prefix (default: {DEFAULT_OUTPUT} → *_diff.png & *_overlay.png)",
    )
    parser.add_argument(
        "-a",
        "--alpha",
        type=float,
        default=DEFAULT_ALPHA,
        help=f"Render opacity in overlay blend 0–1 (default: {DEFAULT_ALPHA})",
    )
    parser.add_argument(
        "--threshold",
        type=int,
        default=DEFAULT_THRESHOLD,
        help=f"Color delta for diff highlight 0–255 (default: {DEFAULT_THRESHOLD})",
    )
    parser.add_argument(
        "--align",
        choices=["resize", "fit", "pad", "none"],
        default=DEFAULT_ALIGN,
        help=f"Size mismatch handling (default: {DEFAULT_ALIGN})",
    )

    args = parser.parse_args()

    out_path = Path(args.output)
    if out_path.is_dir() or str(args.output).endswith(("/", "\\")):
        out_dir = out_path
        diff_path = out_dir / "diff.png"
        overlay_path = out_dir / "overlay.png"
    elif out_path.suffix.lower() == ".png":
        out_dir = out_path.parent
        stem = out_path.stem
        diff_path = out_dir / f"{stem}_diff.png"
        overlay_path = out_dir / f"{stem}_overlay.png"
    else:
        out_dir = out_path.parent
        stem = out_path.name
        diff_path = out_dir / f"{stem}_diff.png"
        overlay_path = out_dir / f"{stem}_overlay.png"

    out_dir.mkdir(parents=True, exist_ok=True)

    img_tpl, img_gen, info = load_and_normalize_images(
        args.template, args.generated, align_mode=args.align
    )

    print("Image comparison:")
    print(
        f"  reference: {info['template_path']} "
        f"({info['template_size'][0]}x{info['template_size'][1]})"
    )
    print(
        f"  render:    {info['generated_path']} "
        f"({info['generated_size'][0]}x{info['generated_size'][1]})"
    )
    print(
        f"  canvas:    {info['final_size'][0]}x{info['final_size'][1]} "
        f"(align: {info['align_mode']})"
    )

    img_overlay = create_overlay_image(img_tpl, img_gen, alpha=args.alpha)
    img_overlay.save(overlay_path)

    img_diff, diff_percent = create_diff_image(
        img_tpl, img_gen, threshold=args.threshold
    )
    img_diff.save(diff_path)

    print(f"  diff score: {diff_percent:.2f}% mismatch (threshold={args.threshold})")
    print("Outputs:")
    print(f"  overlay: {overlay_path}")
    print(f"  diff:    {diff_path}")


if __name__ == "__main__":
    main()
