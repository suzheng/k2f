#!/usr/bin/env python3
"""Pack-verify a minimal smoke tree for each bundled look theme.

Themes are not copied into author packages as deliverables — this script
embeds a throwaway content tree to ensure every look theme compiles.
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

SKILL_ROOT = Path(__file__).resolve().parents[1]
STARTER = SKILL_ROOT / "starter"
LOOKS = SKILL_ROOT / "looks"
PACK_VERIFY = SKILL_ROOT / "scripts" / "pack_verify.py"
LOOK_NAMES = ("quiet-light", "vivid-blocks", "night-wash")

# Minimal tree exercising look extension roles + standard headings/body/card.
SMOKE_ROOT = {
    "id": "root",
    "role": "document",
    "content": {
        "type": "container",
        "value": {
            "children": [
                {
                    "id": "smoke.shell",
                    "role": "shell",
                    "content": {
                        "type": "container",
                        "value": {
                            "children": [
                                {
                                    "id": "smoke.kicker",
                                    "role": "kicker",
                                    "content": {"type": "text", "value": "Series"},
                                },
                                {
                                    "id": "smoke.display",
                                    "role": "display",
                                    "variant": "poster",
                                    "content": {"type": "text", "value": "Headline"},
                                },
                                {
                                    "id": "smoke.metric",
                                    "role": "metric",
                                    "variant": "poster",
                                    "content": {"type": "text", "value": "42"},
                                },
                                {
                                    "id": "smoke.card",
                                    "role": "card",
                                    "content": {
                                        "type": "container",
                                        "value": {
                                            "children": [
                                                {
                                                    "id": "smoke.card.body",
                                                    "role": "body",
                                                    "content": {
                                                        "type": "text",
                                                        "value": "Supporting point.",
                                                    },
                                                }
                                            ]
                                        },
                                    },
                                },
                                {
                                    "id": "smoke.h1",
                                    "role": "h1",
                                    "content": {"type": "text", "value": "Chapter"},
                                },
                                {
                                    "id": "smoke.body",
                                    "role": "body",
                                    "content": {"type": "text", "value": "Body line."},
                                },
                                {
                                    "id": "smoke.caption",
                                    "role": "caption",
                                    "content": {"type": "text", "value": "Caption"},
                                },
                            ]
                        },
                    },
                    "layout": {
                        "type": "stack",
                        "direction": "vertical",
                        "gap": 12000,
                        "height": 730000,
                    },
                    "break_inside": "avoid",
                }
            ]
        },
    },
}


def run_pack_verify(pkg_dir: Path, out_k2f: Path) -> None:
    cmd = [
        sys.executable,
        str(PACK_VERIFY),
        str(pkg_dir),
        "-o",
        str(out_k2f),
        "--expect-pages",
        "1",
    ]
    print("+", " ".join(cmd), flush=True)
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.stdout:
        print(proc.stdout, end="" if proc.stdout.endswith("\n") else "\n")
    if proc.stderr:
        print(proc.stderr, end="" if proc.stderr.endswith("\n") else "\n", file=sys.stderr)
    if proc.returncode != 0:
        raise SystemExit(proc.returncode)


def verify_look(look: str, tmp: Path) -> None:
    theme_src = LOOKS / look / "theme.json"
    if not theme_src.is_file():
        print(f"error: missing theme: {theme_src}", file=sys.stderr)
        raise SystemExit(1)

    pkg = tmp / look
    shutil.copytree(STARTER, pkg)
    shutil.copy2(theme_src, pkg / "styles" / "theme.json")

    manifest_path = pkg / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["title"] = f"Look smoke — {look}"
    manifest["page_config"]["margin"] = [0, 0, 0, 0]
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    root_path = pkg / "content" / "root.json"
    root_path.write_text(json.dumps(SMOKE_ROOT, indent=2) + "\n", encoding="utf-8")

    out_k2f = tmp / f"{look}.K2F"
    print(f"verify look: {look}", flush=True)
    run_pack_verify(pkg, out_k2f)
    print(f"ok: {look}", flush=True)


def main() -> int:
    if not STARTER.is_dir():
        print(f"error: starter missing: {STARTER}", file=sys.stderr)
        return 1
    if not PACK_VERIFY.is_file():
        print(f"error: pack_verify missing: {PACK_VERIFY}", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="k2f-looks-") as tmp_name:
        tmp = Path(tmp_name)
        for look in LOOK_NAMES:
            verify_look(look, tmp)

    print(f"all looks verified ({len(LOOK_NAMES)})", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
