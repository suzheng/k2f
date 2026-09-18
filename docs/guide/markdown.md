# Markdown conversion

Turn Markdown into a packed `.K2F` with `markdown_to_k2f`, or export semantics back with `k2f_to_markdown`. The bridge is **deterministic** and lives in the CLI and SDK — do not hand-parse Markdown into K2F JSON.

Roundtrip preserves **semantic structure** (roles, modifiers, layout hints). It does not preserve lock bytes, theme edits, or signatures.

For full control (footnotes, fillable fields, custom layout), author JSON after import or skip the bridge — see [Getting started](getting-started.md).

## When to use

| Use the bridge | Prefer JSON authoring |
| --- | --- |
| You already have Markdown notes or READMEs | You need footnotes, MDX, or task lists as real content |
| A quick first `.K2F` before patching nodes | PDF or HTML is the source (not Markdown) |
| Export semantics to Markdown for diff or copy | Fillable blanks — use [`ex_form.json`](../../skills/k2f/catalog/content/ex_form.json) in the tree, not `____` in MD |

The bridge accepts normal paragraphs, headings, lists, GFM tables, fenced code, blockquotes, thematic breaks, strikethrough, links, and `$…$` / `$$…$$` math. It **skips** footnotes, YAML front matter, raw HTML, task-list checkboxes, remote images, and several edge cases (listed below).

## Author directory (`--template`)

Every conversion needs an **unpacked author directory**: `manifest.json`, `styles/theme.json`, and embedded fonts under `assets/fonts/`. That directory is the visual shell; Markdown only supplies content.

There are **no named official templates** (`report`, `legal`, …). Create a shell with [Getting started](getting-started.md) (`init_package.py` or unpack an existing `.K2F`), then pass that `source/` path as `--template`.

Optional CLI-only override: `--font path/to/subset.otf` replaces the shell primary font when glyphs are missing (`FONT_MISSING_GLYPH`). Python and JavaScript have no font override — use the CLI or add faces to the author directory first. K2F never uses system fonts.

## CLI

```bash
pip install k2f
# From the skill folder (starter + scripts):
python scripts/init_package.py --workspace ./out/doc --title "From Markdown" --page a4

k2f markdown README.md -o ./out/doc/doc.K2F --template ./out/doc/source
k2f markdown ./notes -o ./out/doc --template ./out/doc/source   # dir: one .K2F per .md
k2f verify ./out/doc/doc.K2F
```

`--template` must be an author directory. The output path may be a new `.K2F` file or a directory (batch mode). The CLI writes the package and **does not print** conversion warnings.

K2F → Markdown has **no CLI command**; use Python, JavaScript, or Rust `k2f_to_markdown` on packed bytes.

## SDK

| Surface | Markdown → K2F | K2F → Markdown | Warnings |
| --- | --- | --- | --- |
| CLI `k2f markdown` | `--template` = author dir; optional `--font` | — | Discarded |
| Python `k2f.markdown_to_k2f(md, template="…", title="…")` | Same template dir | `k2f.k2f_to_markdown(bytes)` | Discarded |
| JS `@openk2f/k2f` | `markdownToK2f(md, { title, templateBytes })` after `initWasm` — pass a **packed** `.K2F` as `templateBytes` | `k2fToMarkdown(bytes)` | Discarded |
| Rust `k2f_sdk::markdown_to_k2f` | `MarkdownOptions::new(title, template_dir)` | `k2f_to_markdown` | `MarkdownResult.report.warnings` |

Inspect warnings in Rust when you need an exact skip list; otherwise treat the [skipped syntax](#skipped-syntax) table as the contract.

## Markdown → K2F mapping {#mapping}

| Markdown | K2F |
| --- | --- |
| `#`–`####` | Roles `h1`–`h4` |
| `#####`–`######` | Mapped to `h4`; warning `heading level N mapped to h4` |
| Paragraphs | Role `body` |
| Lists | Role `list_item` |
| GFM tables | Role `table` |
| Fenced code | Role `code` (text node). Language tag discarded — not a `code_block` content type |
| `>` blockquote | Role `quote` |
| `---` | Role `rule`, text `" "` |
| `~~strike~~` | Strikethrough modifier |
| `[text](url)` | Link modifier (underline from theme) |
| `$…$` / `$$…$$` | Inline math modifier / display `NodeContent::Math` |
| Modifiers on one node | Capped at 50; excess → `truncated modifiers from N to 50` |

Math compile errors use the same codes as math role nodes (`MATH_UNSUPPORTED`, `MATH_PARSE`, `MATH_MISSING_GLYPH`). Whitelist and Noto Sans Math: [Text](../authoring/text.md) and [Theme and fonts](../authoring/theme.md).

### Images

- **Local paths only**, resolved relative to the process working directory (no `image_base` on CLI, Python, or JS).
- Declared width **80 mm**.
- `http(s)` URLs, empty `src`, or missing files → skipped with a warning (see below).

### Fonts

The author directory supplies embedded faces. Missing codepoints fail closed with `FONT_MISSING_GLYPH`. Display math needs Noto Sans Math in that template. Do not substitute Noto Sans SC for Japanese — pick a covering face (`--font` on CLI or `--add-font` when building the shell).

## `<!-- k2f: … -->` comments

HTML comments with a `k2f:` prefix attach layout and role hints on **import**.

**Import (applied)**

| Body | Effect |
| --- | --- |
| `header=…` / `footer=…` | Running header / footer |
| `variant=…` | Pending variant on the next node |
| `keep_with_next=true` | Pending `keep_with_next` |
| `break_before=page` | Pending page break before next node |
| `column_span=all` | Pending full column span |
| `role=warning` | Role + `break_inside: avoid` |
| `role=signature_block` \| `code` \| `quote` \| `body` \| `h4` \| `rule` | Role override |
| Other `role=` values | Ignored |

Tokens may be combined with spaces (`variant=glass keep_with_next=true`).

**Export only (not re-imported)**

- Layout hints such as `columns={count} gap={gap}` — import ignores `columns=`.
- Form fields as `<!-- k2f: form_field kind=text|multiline|checkbox id=… -->` plus value lines — import does **not** parse `form_field` or treat `[____]` / underscore runs as fields.
- Running header/footer and list-item hints may appear as comments on export.

## Skipped syntax

These constructs are dropped with predictable warning strings (Rust `report.warnings`; CLI discards them but behavior is the same):

| Condition | Warning |
| --- | --- |
| Footnote reference | `skipped footnote reference '{id}'` |
| Footnote definition | `skipped footnote definition '{id}'` |
| Task checkbox | `skipped task list checkbox` |
| Raw HTML (non-`k2f` comment) | `skipped raw HTML` |
| YAML / metadata block | `skipped metadata block` |
| Empty table | `skipped empty table` |
| Empty image src | `skipped image with empty src` |
| Remote image | `skipped remote image '{dest}'` |
| Missing local image | `skipped missing image '{dest}'` |
| Image attach error | `skipped image '{dest}': {e}` |
| Inline math outside text buffer | `skipped math` |

MDX is not parsed as MDX — expect ordinary Markdown plus raw-HTML skips.

Do not tell users the conversion was lossless when any of the above applied. Extend the tree in JSON ([Allowed keys](../reference/keys.md), [catalog](../reference/catalog.md)) for content the bridge skipped.

## After import

1. Run `k2f verify` on the package.
2. Open a rendered preview if layout matters ([Getting started](getting-started.md#write-json-yourself) — `pack_verify.py --render` or `k2f render`).
3. On `FONT_MISSING_GLYPH`, re-run CLI `k2f markdown` with `--font` or add fonts to the author shell — do not rewrite the document language to English.
4. Patch roles, theme, and layout in `content/` and `styles/theme.json` as needed ([Authoring](../authoring/text.md)).
5. Export from the lock: [PDF](exporting-pdf.md) (fillable AcroForm), [PowerPoint](exporting-pptx.md), [Word](exporting-docx.md), [InDesign](exporting-idml.md). Markdown does not create fillable fields — add `form_field` nodes from [`ex_form.json`](../../skills/k2f/catalog/content/ex_form.json) before packing.

## Common mistakes

| Mistake | Reality |
| --- | --- |
| Hand-build JSON from Markdown | Call `markdown_to_k2f` / `k2f markdown` |
| Expect signatures or lock in exported MD | Export is unsigned semantics; sign the `.K2F` separately |
| Use `content.type: "code_block"` for fences | Fences map to role `code` + text |
| Assert lock byte equality after roundtrip | Compare structure, not lock |
| Use `http://` images | Download locally and use a relative path |
| Named template `k2f markdown --template report` | Pass an author **directory** from `init_package.py` or unpack |

## See also

- [Getting started](getting-started.md) — create the author shell
- [Text](../authoring/text.md) — modifiers and math after import
- [Theme and fonts](../authoring/theme.md) — roles and embedded fonts
- [Allowed keys](../reference/keys.md) — node and theme fields
- [Export](exporting.md) — PDF, PowerPoint, Word, InDesign from the lock
- [K2F Skill](../../skills/k2f/SKILL.md) — agent workflows (`writing.md` for JSON-only authoring)
