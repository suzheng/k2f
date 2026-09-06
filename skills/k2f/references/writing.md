# Writing K2F Documents

Deliverable is an **UNSIGNED** `.K2F` — signing stays a human step.

**Default path:** edit the semantic tree as JSON in an unpacked author directory, then `pack_verify.py`. Treat `content/*.json` and `styles/theme.json` like source code; `k2f pack` / `compile` / `verify` are the compiler. Do **not** hand-edit `document.K2F.lock`.

## When to use

| Situation | Path |
|-----------|------|
| New CV, flyer, cheatsheet, slide deck, poster, custom layout | Get a working directory (below), then this loop |
| Existing `.K2F` to patch | Same loop — unpack (or use the open dir) at step 2 |
| Already have an unpacked author directory | Edit JSON → `pack_verify.py` |
| Source is Markdown | [converting-markdown.md](converting-markdown.md), then patch JSON here if needed |
| PDF-only request | Write `.K2F` first, then [exporting-pdf.md](exporting-pdf.md) |

**When NOT to use:** PDF as source (`PDF_IS_NOT_A_SOURCE`); pixel/CSS layout edits; per-node `x`/`y` or drag-box layout (change theme + full relock); inventing JSON without catalog + schema lookup.

## Writing loop

### Prerequisites

```bash
pip install k2f    # unpack, pack, compile, verify, render, schema dump — on PATH
```

`pack_verify.py` uses that CLI (or `K2F_CLI`). The author directory can be anywhere; pass its path. The script does not search a source tree or cargo `target/` for a binary.

### Steps

1. **Write a design spec (Markdown).** Before any K2F JSON, produce a concrete design document (e.g. `./out/doc/design.md`). If the user named a style or brand, follow it. If not, design to the highest aesthetic standard for this deliverable. Outline:

   - `## Intent` — audience, tone, one-page vs multi-page
   - `## Typography` — fonts, roles, size intent for this canvas
   - `## Spacing` — margins, stack/grid gaps, card padding
   - `## Color & surfaces` — palette intent, backgrounds, accents
   - `## Layout` — columns, hero, headers/footers, figure placement
   - `## Components` — cards, metrics, captions, tables as needed

   See [Design first](../SKILL.md#design-first) in the skill entry.
2. **Get a working directory.** Probe once, stop at the first hit. Kind mismatch, missing tools, or download failure → do not retry; go to the next row.
   1. User gave a `.K2F` or an unpacked author dir → `k2f unpack existing.K2F -o ./out/doc` (do **not** use `--include-lock`; author dirs must not contain lock or embedded `schema/`) or use the existing directory. For patches, skip step 1 unless the brief changes visual design.
   2. Site MCP already connected (`list_templates` / `download_template`) **and** a catalog `kind` matches the deliverable → download the package URL to a `.K2F`, then unpack as in (1). See [Optional Gallery](#optional-gallery-mcp).
   3. Otherwise: `python scripts/init_package.py --dir ./out/doc --title "…" --page a4` — copies [`starter/`](../starter/) (empty tree + core theme + Roboto).
3. Implement the spec: edit `content/root.json` (+ optional `content/*.json` includes) and `styles/theme.json`. **Minimal diff** on patches — change only what the task requires.
4. **Copy shapes** from [`catalog/content/ex_*.json`](../catalog/content/) — one file per construct (stack, grid, table, …). See [`catalog/README.md`](../catalog/README.md). Do not ship the catalog as your document. The **current package** `styles/theme.json` must already define every role, modifier type, and font those nodes use. Starter already includes `image` and the modifier styles used by `ex_modifiers.json`. Copying [`ex_math.json`](../catalog/content/ex_math.json) still needs NotoSansMath from [`catalog/assets/fonts/`](../catalog/assets/fonts/) plus a `font_aliases` / `math` role font update — starter ships Roboto only.
5. **Unsure about a key?** Read [writing/fields.md](writing/fields.md), then open the matching file under [`schema/`](../schema/) before writing JSON.
6. `python scripts/pack_verify.py ./out/doc -o ./out/doc.K2F --render preview.png`
7. Expect `verify` → **`UNSIGNED`**. Then **open the PNG** — [visual check](#visual-check). If the image does not match the design spec, revise the spec or implementation and pack again.

Page presets: `a4` | `letter` | `a4-landscape` | `widescreen` | `widescreen-43`. Slides/posters: see [writing/package.md](writing/package.md).

### Examples

**New report**

```bash
python scripts/init_package.py --dir ./out/report --title "Q3 Report" --page a4
# CJK/kana: --add-font /path/to/NotoSansJP.otf   (keeps Roboto; not NotoSansSC for Japanese)
# math: copy catalog NotoSansMath + font_aliases; or --add-font that ttf
# slides: --page widescreen --margin 0 | widescreen-43
# posters: --page a4 --margin 0; copy catalog/content/ex_poster_shell.json; set height to page
# --margin 36000  or  --margin 36000,48000,36000,48000
# write design.md first (typography, spacing, layout, color)
# edit content/root.json — copy nodes from catalog/content/ex_*.json
# edit styles/theme.json — implement the design spec
python scripts/pack_verify.py ./out/report -o ./out/report.K2F --render preview.png
```

**Patch an existing package**

```bash
k2f unpack existing.K2F -o ./out/doc
# grep or read content/*.json for stable ids — never guess
# edit the text node (or cell id for tables); roles from this package's theme.json
python scripts/pack_verify.py ./out/doc -o ./out/doc-edited.K2F --render preview.png
```

**Single-page poster check** (`compile` prints `pages=N`; preview is page 0 only):

```bash
python scripts/pack_verify.py ./out/poster -o ./out/poster.K2F --expect-pages 1 --render preview.png
```

### Includes (long documents)

Keep structure in `content/root.json`; add `{ "include": "content/ch01.json" }` stubs in container `children`. Each fragment is one `SemanticNode`. When patching, edit the **fragment file**, not only `root.json`. See [`catalog/content/root.json`](../catalog/content/root.json) for the include pattern.

### Quick reference

| Task | Call |
|------|------|
| New author dir | `init_package.py --dir … --title … --page …` |
| Unpack `.K2F` | `k2f unpack file.K2F -o ./dir` |
| Pack + compile + verify + PNG | `pack_verify.py <dir> -o out.K2F --render preview.png` |
| Extra pages | `k2f render file.K2F --page 1 -o preview-1.png` |
| Single-page poster check | `pack_verify.py … --expect-pages 1 --render preview.png` |
| Custom font | `init_package.py --font /path/to/Covering.ttf` (replaces Roboto) or `--add-font` (fallback beside Roboto) |
| Modifier byte ranges | `python scripts/modifier_range.py --text "…" --find "…"` |
| Allowed JSON keys | [writing/fields.md](writing/fields.md), then [`schema/`](../schema/) |

## Visual check

`verify` is not a layout review. After every pack, render the lock to PNG and **open the image**. Do not ship a file you have not looked at.

```bash
python scripts/pack_verify.py ./out/doc -o ./out/doc.K2F --render preview.png
# compile prints pages=N; --render is page 0 only:
k2f render ./out/doc.K2F --page 1 -o preview-1.png
```

Inspect every page. In particular:

- **Poster/slide empty bottom** — copy [`catalog/content/ex_poster_shell.json`](../catalog/content/ex_poster_shell.json). Compile may print `LAYOUT_SLACK` → [errors.md](writing/errors.md). Do not invent spacer nodes.
- **Report empty bands** — shrink `page_config.margin`, role `padding_pt`, or stack `gap`.
- **Type size** — body text too large or headings too small for the canvas. Change `font_size` on the **role** in `styles/theme.json`, never on the node.

If the PNG does not match the design spec, update the spec or JSON/theme and run `pack_verify.py --render` again.

## Rules (non-negotiable)

1. **Stable dotted ids** on every line, clause, party, and total you may edit later. **Never guess ids** — read the tree or grep `content/`.
2. **Roles must exist** in **this package's** `styles/theme.json` (legal/contract packs often use `critical_warning`, not the SDK catalog name `warning`).
3. **Theme-only styling** — no inline colors/font sizes on nodes.
4. Image width only via declared millipt on image nodes; files under `assets/images/` (PNG/JPEG/WebP/SVG). To swap an image, replace bytes under `assets/images/` and update the node path — there is no in-place byte swap API.
5. **Tables:** edit **cell** text node ids, never the table root id.
6. **Text with modifiers:** after changing `content.value`, recompute modifier `range` with `modifier_range.py` — stale byte ranges fail compile or render wrong.
7. Do not use system `unzip` on the ZIP and edit in place — use `k2f unpack` so lock/schema are omitted and includes stay on disk.
8. Do not hold org signing keys.
9. **Open the rendered PNG** after every pack. `UNSIGNED` is not a visual pass.

## Failure protocol

| Situation | Action |
|-----------|--------|
| Missing `k2f` CLI | `pip install k2f` or set `K2F_CLI`. CWD does not matter. The script does not search a git checkout or build dir |
| validate / compile fails | [writing/errors.md](writing/errors.md); never edit lock |
| `SCHEMA_INVALID` | Open [`schema/`](../schema/); key in schema but rejected → `pip install -U k2f`; key not in schema → remove |
| `UNKNOWN_ID` | Read `content/` or grep for the id; never invent ids |
| `UNKNOWN_ROLE` | Use a role/variant from **this package's** theme |
| `WRONG_CONTENT` | Wrong node kind for a text edit — use cell ids for tables; see [writing/errors.md](writing/errors.md) |
| Poster spilled to page 2 | `compile` prints `pages=N`; use `--expect-pages 1` |
| Viewer `BROKEN_INTEGRITY` | Content changed without relock — run `pack_verify.py` |
| Want Word-like layout | Theme + full relock, not per-node x/y |
| Used `Editor.insert_node` on author dir | Wrong API — edit JSON files, then pack |

## Optional Gallery (MCP)

Not required. Do not install or configure MCP for this skill. If `list_templates` and `download_template` are already available:

1. `list_templates` (optional filters: `kind`, `style`) — metadata only, not package bytes.
2. Use an entry only when `kind` matches the deliverable. Otherwise skip to `init_package.py`.
3. `download_template` returns `{ slug, packageUrl }`. Fetch that URL to a `.K2F` on disk, then `k2f unpack` as in step 2. Do not load the ZIP into context.

Missing tools, empty list, kind mismatch, or fetch error → `init_package.py`. After unpack, the loop is the same JSON + `pack_verify.py` path — do not use `Editor.insert_node` on that directory.

## Common mistakes

| Mistake | Reality |
|---------|---------|
| Copy entire `catalog/` as deliverable | Catalog is reference only — start from `starter/` |
| Output Markdown "for now" | Author JSON, or [converting-markdown.md](converting-markdown.md) |
| Amounts without ids | Every total/line needs a stable id |
| `compile` on a source directory | Directory → `pack`; existing `.K2F` → `compile` |
| System `unzip` + edit `content/root.json` only | Use `k2f unpack`; edit includes in their fragment files |
| Guess node ids | Read or grep `content/` — never invent ids |
| `set_role(..., "warning")` on a custom legal theme | Inspect package theme — it may use `critical_warning` |
| Edit table root id for cell text | Edit **cell** text node ids |
| Expect signature after pack | Pack/relock strips signatures; human re-signs |
| SVG or images in `assets/` root | Move to `assets/images/` (PNG/JPEG/WebP/SVG); root files → `UNEXPECTED_PATH` |
| GIF / BMP / TIFF image | Unsupported — convert to PNG/JPEG/WebP, or embed SVG |
| `self_align` (or any theme field) on a node | Put `self_align`, `text_align`, fonts, padding in `styles/theme.json` roles only |
| `text_align: center` on `running_footer` for page-number centering | Running block **box** position uses role `self_align`; glyph alignment inside the box uses `text_align` |
| `^22^` or Unicode superscript for citations | Use `superscript` modifier + `modifier_range.py`; see `ex_modifiers.json` — Markdown `^` is not parsed |
| `---` thematic break for a new chapter page | Renders as `role: "rule"` — use `break_before: "page"` on the next node, or `<!-- k2f: break_before=page -->` in Markdown |
| Expect whole paragraphs to jump to next page | `break_inside: auto` splits **by line** when the page remainder is too small — shrink padding/gap or split into sibling nodes |
| `$...$` inline math in author JSON | U+FFFC + `{ "type": "math", "intent": "<tex>" }` modifier — see `catalog/content/ex_modifiers.json`; `$` works Markdown only |
| Expect `\mathbb` / `\forall` / `\prime` / `\text` unsupported | They are in the TeX whitelist ([errors.md](writing/errors.md)); add NotoSansMath. No `\color`/`\textcolor`/`\tag`/`\mathbf`/`\sqrt[n]` — role color + `ex_math_numbered.json` |
| Multiple fonts but `"default":"default"` only | Map `font_aliases` to each file stem; two+ fonts have no auto-`default` |
| `fr` rows without fixed grid height | Fails: `Cannot resolve fr tracks with infinite available size`. `fr` ≠ content-auto height — set grid `layout.height`, use `pt`/`auto` rows, or nest under a fixed-height stack (`ex_grid.json`) |
| Poster/slide shell is a vertical stack | Content piles at the top. Copy `ex_poster_shell.json`: pinned `height` + `{fr:1}` body row |
| Cover year in the footer / vertical space-between | Copy `ex_cover.json` / `ex_poster_shell.json` (`{auto:true}` + `{fr:1}` + `{auto:true}`), not padding guesses or empty spacers. Flow-only (footer not at page bottom) → vertical stack, not the `{fr:1}` shell |
| Letter sender / right-flush cell | Copy `ex_end_block.json` (horizontal `justify_content: end` wrapping a content-width vertical stack). Left+right pair → `ex_split_bar.json`. Do not `text_align: end` on each line |
| Expect small-caps / `font_variant` | Not in v0.1 — role uppercase + `letter_spacing_pt` |
| Expect per-cell grid align or baseline | Nest stack / theme `self_align` / `ex_end_block.json` in the right cell; `cell_align.y: start` — no first-line baseline |
| Binding gutter + title centered on the sheet | `margin` 4-tuple is the gutter; `text_align: center` is the **content box**. Overlay or equal padding on that title role |
| Academic serif missing from starter | `--add-font` a serif TTF; starter ships Roboto only |
| Inline code pills / modifier background | Sibling `role: code` (or a decorated container), not an inline background patch |
| Expect a native `Divider` node | Use `role: "rule"` + small `layout.height` + surface fill (or bottom border) |
| Noise / vignette / radial glow / dot matrix | Not in core — SVG under `assets/images/` (labels as `<path>`); size in millipt |
| Require `row_gap`/`column_gap`/`cell_align` | Optional — see `schema/nodes.schema.json`; omit unused keys (`null` ok) |
| Unicode superscript (`²`) for notes | Ordinary char + `superscript`/`subscript` modifier; formulas → math |
| SVG `<text>` labels | Convert to `<path>` — paint has no system fonts; `<text>` now **fails** instead of dropping silently |
| One text node with `\n\n` for paragraphs | One paragraph = one text node; `\n` is a hard line break (each line still takes `line_height_mult`). Paragraph spacing = sibling `gap` |
| Simulate margin with padding / empty spacer stacks | No node margin or node `padding_pt`. Even rhythm: parent `gap`. Uneven: nested stacks with different `gap`, or a **dedicated** role's `padding_pt`. Shared `h1`/`body` padding applies to every such node |
| Dingbat/arrow/CJK glyphs (★ ◆ → ↗ ↑ 中文 かな) in Roboto | `FONT_MISSING_GLYPH` — `--add-font` a covering face (JP/KR/SC as needed). NotoSansSC ≠ Japanese. Math formulas → NotoSansMath. Do not rewrite user language to English |
| Expect `canvas_mode: "slide"` | v0.1 is `paged` only — use `--page widescreen --margin 0` + per-slide fixed height + `break_inside: avoid` |
| Expect `justify_content: space-between` or page `background` | Copy `ex_split_bar.json` / `ex_end_block.json` / `ex_poster_shell.json` / `ex_overlay.json` — not Flexbox or `page_config` |
| Overlay nested stack `align_items: end` not on the page right | Overlay children shrink to content unless that layer sets `width`; left/right bars → `ex_split_bar.json`; trailing-edge block (letter sender) → `ex_end_block.json` |
| Hand-count modifier ranges across `\n` | `\n` is 1 UTF-8 byte — run `modifier_range.py --text` with the exact node `value` |
| Pixel formula for cover padding vs line-height | Copy `ex_cover.json`; iterate the PNG. Line boxes + `padding_pt` + `gap` **add**; do not invent spacer nodes or cancel line boxes with padding math |
| Expect table `colspan` / `vertical_align` | Extra columns + cell `variant: "hbar"` / `"bottom"` (`ex_table_edges.json`); `variant: "center"` for vertical middle |
| `layout.height` + padding overflowing the page | Height is min outer; padding is inside. Do not nest another full-page-height child inside a padded shell. Single-page: grid shell first (`ex_poster_shell.json`), then inner `gap` — a page-height `avoid` stack with large padding → `UNSPLITTABLE_OVERFLOW` |
| Trust `preview.png` alone for single-page posters | Default render is page 0; check `pages=1` / `--expect-pages 1` |
| Ship after `UNSIGNED` without opening the PNG | `verify` does not catch empty margins or oversized type — [visual check](#visual-check) |
| Used `Editor.insert_node` to add grid/stack | Agent dialect — edit author JSON then pack |
| Stale modifier ranges after text edit | Run `modifier_range.py` on the new `value` |

## Optional SDK path

This skill's **default** is JSON + CLI. Python `Editor`, [edit_and_verify.py](../scripts/edit_and_verify.py), and MCP tools are optional shortcuts for scripts or environments without file editing — not the primary agent workflow.

- One-line text patch: [../scripts/edit_and_verify.py](../scripts/edit_and_verify.py)
- Full Editor API, binding names, `save_with`, dialect limits: [writing/sdk.md](writing/sdk.md)

Requires `pip install k2f` only when using those shortcuts.

## See also

- [writing/fields.md](writing/fields.md) — node vs theme allowed keys
- [writing/package.md](writing/package.md) — slides, posters, fonts, capability limits
- [writing/errors.md](writing/errors.md) — error codes
- [converting-markdown.md](converting-markdown.md) — Markdown source
- [exporting-pdf.md](exporting-pdf.md) — PDF after pack
- [publishing.md](publishing.md) — permanent `/v/{appearance_hash}`
- [embedding-viewer.md](embedding-viewer.md) — human click → node id
