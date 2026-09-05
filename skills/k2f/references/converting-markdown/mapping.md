# Markdown ↔ K2F mapping (K2F-specific only)

Do not teach CommonMark/GFM here. Only K2F bridge quirks.

## Structural differences

| Markdown | K2F behavior |
|----------|----------------|
| `#`–`####` | Roles `h1`–`h4` |
| `#####`–`######` | Mapped to `h4`; warning `heading level N mapped to h4` |
| Fenced code | Role `code` (text node). Fence language tag discarded. Not a `code_block` content type |
| `>` blockquote | Role `quote` |
| `---` thematic break | Role `rule`, text `" "` |
| `~~strike~~` | Strikethrough modifier |
| `[t](url)` | Link modifier (+ underline in theme) |
| Modifiers | Capped at 50; excess → `truncated modifiers from N to 50` |
| `$$…$$` | `NodeContent::Math` (display) |
| `$…$` | Inline math modifier on body/table/list text |
| Math compile errors | Same codes as math role nodes (`MATH_UNSUPPORTED` / `MATH_PARSE` / `MATH_MISSING_GLYPH`) — see [writing.md](../writing.md) |

Ordinary paragraphs → `body`; lists → `list_item`; GFM tables → `table`. Agents already know those Markdown forms.

## Images

- Local paths only; declared width **80 mm**.
- `http://` / `https://` → skip (warning `skipped remote image '…'`).
- Empty src / missing file / read error → skip with matching warning.
- **CLI / Python / JS:** paths relative to process CWD (no `image_base` option on CLI/Python/JS).

## Fonts

SDK/CLI packages embed the named theme’s primary face plus a monochrome emoji companion. Missing codepoints fail with `FONT_MISSING_GLYPH` (inventory in the error). Pass `--font` on CLI with a covering TTF/OTF; Python/JS `markdown_to_k2f` have no font override — fix text or use CLI. Never use system fonts.

## Skip warnings (exact strings from import)

| Condition | Warning |
|-----------|---------|
| Footnote ref | `skipped footnote reference '{id}'` |
| Footnote def | `skipped footnote definition '{id}'` |
| Task checkbox | `skipped task list checkbox` |
| Raw HTML (non-k2f comment) | `skipped raw HTML` |
| YAML/metadata block | `skipped metadata block` |
| Empty table | `skipped empty table` |
| Empty image src | `skipped image with empty src` |
| Remote image | `skipped remote image '{dest}'` |
| Missing file | `skipped missing image '{dest}'` |
| Image add error | `skipped image '{dest}': {e}` |
| Inline math outside text buffer | `skipped math` |

MDX is not parsed as MDX; treat as ordinary MD + raw HTML skips.

## Who gets `ConversionReport`

| Surface | Report |
|---------|--------|
| Rust `markdown_to_k2f` | Yes — `MarkdownResult.report.warnings` |
| CLI `k2f markdown` | Discarded (package written only) |
| Python / JS | Discarded (bytes only) |

## `<!-- k2f: … -->` comments

### Import (applied)

| Body | Effect |
|------|--------|
| `header=…` | `set_running_header` |
| `footer=…` | `set_running_footer` |
| `variant=…` | Pending variant on next node |
| `keep_with_next=true` | Pending keep_with_next |
| `break_before=page` | Pending `break_before: page` on next node |
| `column_span=all` | Pending full column span |
| `role=warning` | Role + `break_inside: avoid` |
| `role=signature_block` \| `code` \| `quote` \| `body` \| `h4` \| `rule` | Role override |
| Other `role=` values | Ignored |

Comments may combine whitespace-separated tokens (`variant=glass keep_with_next=true`).

### Export-only (not re-imported)

Exporter may emit `columns={count} gap={gap}` for layout hints. **Import does not parse `columns=`.**

Also exported: running header/footer as `header=` / `footer=` comments; list-item hints as trailing `<!-- k2f: … -->` on the same line.

## Roundtrip expectations

- Preserve common semantic structure (headings, lists, tables, emphasis, code, quotes, links, local images, math).
- Theme, lock geometry, and signatures are **not** in Markdown.
- No CLI command for K2F → MD; use Python/JS/Rust `k2f_to_markdown`.
- Do **not** assert lock byte equality across roundtrip.
