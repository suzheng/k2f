# Markdown ↔ K2F Conversion

Conversion runs in the **SDK** (`markdown_to_k2f`, `k2f_to_markdown`). **Never** parse Markdown in the agent and emit raw K2F JSON — use the deterministic bridge. Roundtrip targets **semantic structure**, not lock bytes, theme, or signatures.

**Preferred agent path:** [writing.md](writing.md) (`init_package.py` or unpack, then JSON). Do not use named official templates (`report`, `legal`, …) — they are not in this skill folder.

If you call the Markdown bridge, pass an **author directory** as `--template` / `template` (starter or an unpacked package). The CLI and SDK accept a directory path.

Do not teach CommonMark/GFM here. Only K2F bridge quirks below.

## When to Use

- User has Markdown → needs `.K2F`
- Agent would otherwise output Markdown for a formal deliverable

**When NOT to use**

- PDF/HTML as input
- Full-fidelity MDX or footnotes — v0 skips; extend via [writing.md](writing.md) after import

## CLI (default)

```bash
python scripts/init_package.py --dir ./out/doc --title "From Markdown" --page a4
# dest may be new, empty, or notes-only; not an existing package
k2f markdown README.md -o readme.K2F --template ./out/doc
k2f markdown ./notes -o ./out --template ./out/doc   # directory: mirrors .md → .K2F
k2f verify readme.K2F
```

`--template` is an unpacked author directory (fonts + `styles/theme.json`). Optional covering font: `--font path/to/subset.otf`. CLI writes the package and **does not print warnings**.

## Other surfaces

| Surface | Call | Report |
|---------|------|--------|
| CLI `k2f markdown` | writes package; `--template` = author dir | Discarded |
| Python | `markdown_to_k2f(md, title=…, template="./out/doc")` / `k2f_to_markdown(bytes)` | Discarded (bytes only) |
| JS | `markdownToK2f(md, { title, template: "./out/doc" })` / `k2fToMarkdown(bytes)` after `initWasm` — filesystem surfaces only; otherwise [writing.md](writing.md) | Discarded (bytes only) |
| Rust | `markdown_to_k2f(md, opts) → MarkdownResult { bytes, report }` | Yes — `report.warnings` |

No CLI command for K2F → MD; use Python/JS/Rust `k2f_to_markdown`.

## Mapping

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
| Math compile errors | Same codes as math role nodes (`MATH_UNSUPPORTED` / `MATH_PARSE` / `MATH_MISSING_GLYPH`) — TeX whitelist + NotoSansMath: [errors.md](writing/errors.md) |

Ordinary paragraphs → `body`; lists → `list_item`; GFM tables → `table`.

**Images:** local paths only; declared width **80 mm**. `http(s)` / empty src / missing file → skip. Paths are relative to process CWD (no `image_base` on CLI/Python/JS).

**Fonts:** the author directory passed as `--template` / `template` supplies embedded faces. Missing codepoints fail with `FONT_MISSING_GLYPH`. Formulas need NotoSansMath in that template. Pass `--font` on CLI to a covering face (JP/KR/SC as needed; NotoSansSC ≠ Japanese). Python/JS `markdown_to_k2f` have no font override — use CLI. Never use system fonts. Do not rewrite the user's language to English.

## Skip warnings (exact strings)

Warn the user about these even when the CLI discards the report.

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

## Validation loop

1. Convert (`k2f markdown --template <author_dir>` / SDK), or skip the bridge and author JSON via [writing.md](writing.md).
2. Warn the user about known skips (table above) — do not claim full conversion.
3. `k2f verify` on the package.
4. `FONT_MISSING_GLYPH` → re-run with `k2f markdown --font` pointing at a covering TTF/OTF (not NotoSansSC for Japanese). Do not rewrite the user's language to English.
5. Patch nodes → [writing.md](writing.md). PDF → [exporting-pdf.md](exporting-pdf.md).

## Failure protocol

| Situation | Action |
|-----------|--------|
| Footnotes / MDX / raw HTML / task checkboxes | Skipped; do not claim full conversion |
| Remote `http(s)` image | Skipped — use a local path under CWD |
| Missing local image | Skipped |
| `FONT_MISSING_GLYPH` | Covering `--font` (JP/KR/SC as needed); no OS fonts; do not rewrite user language |
| Need footnotes as content | Keep in MD or extend tree with [writing.md](writing.md) |
| `unknown template '…'` | Do not look up official named templates. `init_package.py` ([writing.md](writing.md)), then pass that directory as `--template` |

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
