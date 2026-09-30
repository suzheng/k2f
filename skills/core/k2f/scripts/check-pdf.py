#!/usr/bin/env python3
"""Sanity-check a K2F-exported PDF. Does not parse xref or decompress streams."""

from __future__ import annotations

import sys
from pathlib import Path


def fail(msg: str) -> int:
    print(f"check-pdf: {msg}", file=sys.stderr)
    return 1


def check(path: Path) -> int:
    if not path.is_file():
        return fail(f"not a file: {path}")
    data = path.read_bytes()
    if not data:
        return fail(f"empty file: {path}")
    if data.startswith(b"PK"):
        return fail(f"looks like a ZIP/.K2F, not a PDF: {path}")
    if not data.startswith(b"%PDF-"):
        return fail(f"missing %PDF- magic: {path}")
    official = (
        b"K2F PDF bridge" in data
        or b"appearance_hash=" in data
        or b"Official source is K2F" in data
    )
    if not official:
        return fail(
            "missing K2F Producer (K2F PDF bridge); use k2f export-pdf, not html2pdf/jsPDF. "
            "Default export is a clean PDF — do not pass --trust-pack just to satisfy this check"
        )
    if b"/Count " not in data:
        return fail("missing /Count (page dictionary); export may be truncated")
    print(f"check-pdf: ok {path} ({len(data)} bytes)")
    return 0


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: check-pdf.py <out.pdf>", file=sys.stderr)
        return 2
    return check(Path(argv[1]))


if __name__ == "__main__":
    sys.exit(main(sys.argv))
