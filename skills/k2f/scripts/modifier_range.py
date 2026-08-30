#!/usr/bin/env python3
"""Print UTF-8 byte range [start, end) for a substring of a text node's value.

modifier `range` is byte offsets (on character boundaries), not character indices.
ASCII matches both; CJK / curly quotes / emoji do not.
`\\n` in the text is one byte — pass the exact node `value` (including newlines).

  python scripts/modifier_range.py --text "Do not sign." --find "Do not"
  # [0, 6]

  python scripts/modifier_range.py --text $'Line one\\nLine two' --find "Line two"
  # [9, 17]  (the newline between lines is byte index 8)
"""

from __future__ import annotations

import argparse
import json
import sys


def byte_range(text: str, find: str, occurrence: int = 0) -> list[int]:
    if not find:
        raise ValueError("--find must be non-empty")
    encoded = text.encode("utf-8")
    needle = find.encode("utf-8")
    start = -1
    cursor = 0
    for _ in range(occurrence + 1):
        start = encoded.find(needle, cursor)
        if start < 0:
            raise ValueError(f"substring not found (occurrence {occurrence}): {find!r}")
        cursor = start + 1
    end = start + len(needle)
    return [start, end]


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Compute UTF-8 byte [start, end) for a K2F text modifier range"
    )
    parser.add_argument("--text", required=True, help="Exact text node value")
    parser.add_argument("--find", required=True, help="Substring to mark")
    parser.add_argument(
        "--nth",
        type=int,
        default=0,
        help="0-based occurrence if the substring repeats (default 0)",
    )
    args = parser.parse_args()
    try:
        rng = byte_range(args.text, args.find, args.nth)
    except ValueError as e:
        print(f"error: {e}", file=sys.stderr)
        return 1
    print(json.dumps(rng))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
