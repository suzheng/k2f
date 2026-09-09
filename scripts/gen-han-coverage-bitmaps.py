#!/usr/bin/env python3
"""Write BMP hanzi bitsets (U+4E00..U+9FFF) for pack-time CJK coverage.

Tables: GB2312, Big5 level 1 (常用繁体), JIS X 0208 kanji. Non-Han is kept
separately in the engine. Re-run after editing the ranges; do not hand-edit bins.
"""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "engine/k2f_package/src/fonts"

BMP_START = 0x4E00
BMP_END = 0x9FFF
NBYTES = ((BMP_END - BMP_START + 1) + 7) // 8  # 2624


def pack(codepoints: set[int]) -> bytes:
    bits = bytearray(NBYTES)
    for cp in codepoints:
        if cp < BMP_START or cp > BMP_END:
            continue
        i = cp - BMP_START
        bits[i >> 3] |= 1 << (i & 7)
    return bytes(bits)


def in_bmp(s: str) -> set[int]:
    return {ord(ch) for ch in s if BMP_START <= ord(ch) <= BMP_END}


def gb2312_hanzi() -> set[int]:
    out: set[int] = set()
    for b1 in range(0xB0, 0xF8):
        for b2 in range(0xA1, 0xFF):
            try:
                out |= in_bmp(bytes([b1, b2]).decode("gb2312"))
            except UnicodeDecodeError:
                continue
    return out


def big5_level1_hanzi() -> set[int]:
    """Big5 常用字: 0xA440–0xC67E."""
    out: set[int] = set()
    trail = list(range(0x40, 0x7F)) + list(range(0xA1, 0xFF))
    for b1 in range(0xA4, 0xC7):
        for b2 in trail:
            if b1 == 0xC6 and b2 > 0x7E:
                continue
            try:
                out |= in_bmp(bytes([b1, b2]).decode("big5"))
            except UnicodeDecodeError:
                continue
    return out


def jisx0208_kanji() -> set[int]:
    """JIS X 0208 kanji rows 16–84 via EUC-JP. Kana/symbols are non-Han in the engine."""
    out: set[int] = set()
    for row in range(16, 85):
        for col in range(1, 95):
            try:
                out |= in_bmp(bytes([0xA0 + row, 0xA0 + col]).decode("eucjp"))
            except UnicodeDecodeError:
                continue
    return out


def write(name: str, cps: set[int]) -> None:
    path = OUT / name
    path.write_bytes(pack(cps))
    print(f"wrote {path.name} ({path.stat().st_size} bytes, {len(cps)} hanzi)")


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    gb = gb2312_hanzi()
    big5 = big5_level1_hanzi()
    jis = jisx0208_kanji()
    write("gb2312_hanzi.bin", gb)
    write("big5_l1_hanzi.bin", big5)
    write("jisx0208_hanzi.bin", jis)
    union = gb | big5 | jis
    print(
        f"union {len(union)} (gb2312={len(gb)} big5-l1={len(big5)} jisx0208={len(jis)})"
    )


if __name__ == "__main__":
    main()
