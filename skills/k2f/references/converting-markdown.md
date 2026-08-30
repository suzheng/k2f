# Markdown ↔ K2F Conversion

## Overview

Conversion runs in the **SDK** (`markdown_to_k2f`, `k2f_to_markdown`). **Never** parse Markdown in the agent and emit raw K2F JSON — use the deterministic bridge. Roundtrip targets **semantic structure**, not lock bytes, theme, or signatures.

**Default theme for import:** `report` (embedded in the Python/JS package — not in this skill folder). Fallback: `legal`.

K2F-specific mapping, skip strings, and `<!-- k2f: … -->` comments: [converting-markdown/mapping.md](converting-markdown/mapping.md).

## Prerequisites

```bash
pip install k2f    # CLI on PATH + markdown bridge
```

## When to Use

- User has Markdown → needs `.K2F`
- Agent would otherwise output Markdown for a formal deliverable

**When NOT to use**

- PDF/HTML as input
- Full-fidelity MDX or footnotes — v0 skips; extend via [writing.md](writing.md) after import

## CLI (default)

```bash
k2f markdown README.md -o readme.K2F --theme report
k2f markdown ./notes -o ./out --theme report   # directory: mirrors .md → .K2F
k2f verify readme.K2F
```

Optional covering font: `--font path/to/subset.otf`.

## Other surfaces

| Surface | Call | Notes |
|---------|------|-------|
| Python | `markdown_to_k2f(md, title=…, template="report")` / `k2f_to_markdown(bytes)` | **bytes only** — no report |
| JS | `markdownToK2f(md, { title, theme })` / `k2fToMarkdown(bytes)` after `initWasm` | **bytes only** |
| Rust | `markdown_to_k2f(md, opts) → MarkdownResult { bytes, report }` | Full report |

CLI writes the package and **does not print warnings**.

## Validation loop

1. Convert (`k2f markdown` / SDK).
2. Warn user about known skips (footnotes, task boxes, raw HTML, remote images) per [mapping.md](converting-markdown/mapping.md).
3. `k2f verify` on the package.
4. `FONT_MISSING_GLYPH` → change the text, or re-run with `k2f markdown --font` pointing at a covering TTF/OTF. SDK packages embed the theme primary face plus an emoji companion; **never** fall back to OS fonts.
5. Patch nodes → [writing.md](writing.md). PDF → [exporting-pdf.md](exporting-pdf.md).

## Failure protocol

| Situation | Action |
|-----------|--------|
| Footnotes / MDX / raw HTML / task checkboxes | Skipped (warning when report exists); do not claim full conversion |
| Remote `http(s)` image | Skipped — use a local path under CWD |
| Missing local image | Skipped |
| `FONT_MISSING_GLYPH` | Covering `--font`, or change text; no OS fonts |
| Need footnotes as content | Keep in MD or extend tree with [writing.md](writing.md) |

## Common mistakes

| Mistake | Reality |
|---------|---------|
| Hand-build JSON from MD | Call the bridge |
| Expect signature / lock in MD | Export is unsigned semantics; sign the `.K2F` package |
| Emit `code_block` content type | Fence → role `code` |
| Assert lock byte equality after roundtrip | Compare structure, not lock |

## See also

- [writing.md](writing.md) — unpacked author directory when Markdown is not the source; patch after import
- [exporting-pdf.md](exporting-pdf.md) — PDF from lock
