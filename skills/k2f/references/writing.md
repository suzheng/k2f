# Writing K2F Documents

Deliverable is an **UNSIGNED** `.K2F` — signing stays a human step.

**Default path:** edit the semantic tree as JSON in an unpacked author directory, then `pack_verify.py`. Treat `content/*.json` and `styles/theme.json` like source code; `k2f pack` / `compile` / `verify` are the compiler. Do **not** hand-edit `document.K2F.lock`.

## When to use

| Situation | Path |
|-----------|------|
| New CV, flyer, cheatsheet, slide deck, poster, custom layout | Get a workspace (below), then this loop |
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

1. **Write a design spec (Markdown).** Before any K2F JSON, produce a concrete design document **inside** the author directory (e.g. `./out/doc/source/design.md`). If the user named a style or brand, follow it. If not, design to the highest aesthetic standard for this deliverable. Outline:

   - `## Intent` — audience, tone, one-page vs multi-page
   - `## Typography` — fonts, roles, size intent for this canvas
   - `## Spacing` — margins, stack/grid gaps, card padding
   - `## Color & surfaces` — palette intent, backgrounds, accents
   - `## Layout` — columns, hero, headers/footers, figure placement
   - `## Components` — cards, metrics, captions, tables as needed

   See [Design first](../SKILL.md#design-first) in the skill entry.
2. **Get a workspace.** Probe once, stop at the first hit. Kind mismatch, missing tools, or download failure → do not retry; go to the next row.
   1. User gave a `.K2F` or an unpacked author dir → `k2f unpack existing.K2F -o ./out/doc/source` (do **not** use `--include-lock`; author dirs must not contain lock or embedded `schema/`) or use the existing `source/` directory. For patches, skip step 1 unless the brief changes visual design.
   2. User gave a Gallery **package URL** → fetch it (`curl`/`fetch`) to `./out/doc/template.K2F`, then unpack as in (1). Do not load the ZIP into context.
   3. Site MCP already connected (`list_templates` / `download_template`) **and** a catalog `kind` matches the deliverable → download the package URL to a `.K2F`, then unpack as in (1). See [Optional Gallery](#optional-gallery-mcp).
   4. Otherwise: `python scripts/init_package.py --workspace ./out/doc --title "…" --page a4` — creates `source/` (starter tree + core theme + Roboto) and `tmp/`. Empty dirs and notes-only dirs (a lone `design.md` in `source/`) are OK; an existing package is not. `--dir` still inits a package tree directly when you already have the author path.
3. Implement the spec: edit `content/root.json` (+ optional `content/*.json` includes) and `styles/theme.json`. **Minimal diff** on patches — change only what the task requires.
4. **Copy shapes** from [`catalog/content/ex_*.json`](../catalog/content/) — one file per construct (stack, grid, table, …). See [`catalog/README.md`](../catalog/README.md). Do not ship the catalog as your document. The **current package** `styles/theme.json` must already define every role, modifier type, and font those nodes use. Starter already includes `image` and the modifier styles used by `ex_modifiers.json`. Copying [`ex_math.json`](../catalog/content/ex_math.json) still needs NotoSansMath from [`catalog/assets/fonts/`](../catalog/assets/fonts/) plus a `font_aliases` / `math` role font update — starter ships Roboto only.
5. **Unsure about a key?** Read [writing/fields.md](writing/fields.md), then open the matching file under [`schema/`](../schema/) before writing JSON.
6. `python scripts/pack_verify.py ./out/doc/source -o ./out/doc/doc.K2F --render preview.png` — bare `preview.png` is written to `./out/doc/tmp/preview.png` (next to the output `.K2F`, not next to `manifest.json`). Paths that contain a directory (`--render ./out/preview.png`) stay relative to the shell CWD. The script prints `ok: rendered <abs>`. Do not write `.K2F`, PDF, DOCX, PPTX, or preview PNGs into `source/`.
7. Expect `verify` → **`UNSIGNED`**. Then **open the PNG** — [visual check](#visual-check). If the image does not match the design spec, revise the spec or implementation and pack again.

Page presets: `a4` | `letter` | `a4-landscape` | `widescreen` | `widescreen-43`. Slides/posters: see [writing/package.md](writing/package.md).

### Examples

**New report**

```bash
# write ./out/report/source/design.md first (typography, spacing, layout, color)
python scripts/init_package.py --workspace ./out/report --title "Q3 Report" --page a4
# CJK/kana: --add-font /path/to/NotoSansJP.otf   (keeps Roboto; not NotoSansSC for Japanese)
# math: copy catalog NotoSansMath + font_aliases; or --add-font that ttf
# slides: --page widescreen --margin 0 | widescreen-43; copy ex_poster_shell.json (page_shell, height 540000)
# posters: --page a4 --margin 0; copy catalog/content/ex_poster_shell.json; set height to 842000
# --margin 36000  or  --margin 36000,48000,36000,48000
# edit source/content/root.json — copy nodes from catalog/content/ex_*.json
# edit source/styles/theme.json — implement the design spec
python scripts/pack_verify.py ./out/report/source -o ./out/report/report.K2F --render preview.png
# preview.png lands in ./out/report/tmp/ (not source/, not CWD)
k2f export-pdf ./out/report/report.K2F -o ./out/report/report.pdf
```

**Patch an existing package**

```bash
k2f unpack existing.K2F -o ./out/doc/source
# grep or read source/content/*.json for stable ids — never guess
# edit the text node (or cell id for tables); roles from this package's theme.json
python scripts/pack_verify.py ./out/doc/source -o ./out/doc/doc.K2F --render preview.png
```

**Single-page poster check** (`compile` prints `pages=N`; preview is page 0 only):

```bash
python scripts/pack_verify.py ./out/poster/source -o ./out/poster/poster.K2F --expect-pages 1 --render preview.png
```

### Includes (long documents)

Keep structure in `content/root.json`; add `{ "include": "content/ch01.json" }` stubs in container `children`. Each fragment is one `SemanticNode`. When patching, edit the **fragment file**, not only `root.json`. See [`catalog/content/root.json`](../catalog/content/root.json) for the include pattern.

### Quick reference

| Task | Call |
|------|------|
| New workspace | `init_package.py --workspace … --title … --page …` (creates `source/` + `tmp/`) |
| New author dir | `init_package.py --dir … --title … --page …` (package tree only) |
| Unpack `.K2F` | `k2f unpack file.K2F -o ./dir/source` |
| Pack + compile + verify + PNG | `pack_verify.py <source> -o <workspace>/<name>.K2F --render preview.png` (PNG → `<workspace>/tmp/`) |
| Extra pages | written as `preview-1.png` … next to `--render` (or `k2f render --page N`) |
| Single-page poster check | `pack_verify.py … --expect-pages 1 --render preview.png` |
| Custom font | `init_package.py --font /path/to/Covering.ttf` (replaces Roboto) or `--add-font` (fallback beside Roboto). Readable `.ttf`/`.otf` only — not `/System/Library/Fonts` |
| Modifier byte ranges | `python scripts/modifier_range.py --text "…" --find "…"` |
| Allowed JSON keys | [writing/fields.md](writing/fields.md), then [`schema/`](../schema/) |

## Visual check

`verify` is not a layout review. After every pack, render the lock to PNG and **open the image**. Do not ship a file you have not looked at.

```bash
python scripts/pack_verify.py ./out/doc/source -o ./out/doc/doc.K2F --render preview.png
# compile prints pages=N; bare preview.png → ./out/doc/tmp/preview.png; extra pages → preview-1.png …
```

Inspect every page — especially the **bottom third**. Do not millipt-budget “this is a 3-page contract”; `pages=N` is compile output.

- **Composed** (invoice, CV, flyer, poster, slide) — [`ex_filled_page.json`](../catalog/content/ex_filled_page.json) or [`ex_poster_shell.json`](../catalog/content/ex_poster_shell.json). `height` = content box. `--expect-pages` = page count. `{fr:1}` eats leftover whenever outer height is known (not a poster feature). Leftover → table/notes/figure or dense `{fr:1}` siblings ([`ex_poster_growers.json`](../catalog/content/ex_poster_growers.json)), not a short quote. `{fr:1}` stretches the **box**, not type. No spacers / `space-between`. Short letter may stay a top-packed stack.
- **Flow** (contract, report, thesis, paper) — **one** tree (vertical stack or one unpadded `columns`). Do not wrap each page in `p1.container` / `p2.container`. `break_before: page` on a chapter, annex, or signature page is correct. Last page may be short. Padded/grid/overlay sections do not split.
- **Type size** — `font_size` on the **role** in `styles/theme.json`, never on the node.

`PAGE_UNDERFILL` / `LAYOUT_SLACK` are compile warnings (`pack_verify.py` still exits 0). Invoice/CV/flyer/poster: treat `PAGE_UNDERFILL` as must-fix. Auto-height stacks never produced `LAYOUT_SLACK`; the page check is `PAGE_UNDERFILL`.

If the PNG does not match the design spec, update the spec or JSON/theme and run `pack_verify.py --render` again.

## Rules (non-negotiable)

1. **Stable dotted ids** on every line, clause, party, and total you may edit later. **Never guess ids** — read the tree or grep `content/`.
2. **Roles must exist** in **this package's** `styles/theme.json` (legal/contract packs often use `critical_warning`, not the SDK catalog name `warning`).
3. **Theme-only styling** — no inline colors/font sizes on nodes.
4. Image width only via declared millipt on image nodes; files under `assets/images/` (PNG/JPEG/WebP/SVG). SVG must be paths only — no `<text>` / `<tspan>` / `<textPath>` / `<foreignObject>` (XML comments/CDATA mentioning those tags are ignored); put labels in a K2F text node beside the image. To swap an image, replace bytes under `assets/images/` and update the node path — there is no in-place byte swap API.
5. **Tables:** edit **cell** text node ids, never the table root id.
6. **Text with modifiers:** required fields are `range`, `type`, **`intent`**. After changing `content.value`, recompute `range` with `modifier_range.py` — stale or hand-counted byte ranges fail compile or render wrong. `\n` in the value is 1 UTF-8 byte.
7. Do not use system `unzip` on the ZIP and edit in place — use `k2f unpack` so lock/schema are omitted and includes stay on disk.
8. Do not hold org signing keys.
9. **Open the rendered PNG** after every pack. `UNSIGNED` is not a visual pass.

## Failure protocol

| Situation | Action |
|-----------|--------|
| Missing `k2f` CLI | `pip install k2f` or set `K2F_CLI`. CWD does not matter. The script does not search a git checkout or build dir |
| `error: exists as a package` | Dest already has `manifest.json` / `content/` / `styles/` / `assets/`. Edit in place, or another `--dir` / `--workspace`. Notes-only dirs (`source/design.md`) are OK. Spec file: `./out/doc/source/design.md` |
| validate / compile fails | [writing/errors.md](writing/errors.md); never edit lock |
| `SCHEMA_INVALID` | Open [`schema/`](../schema/); key in schema but rejected → `pip install -U k2f`; key not in schema → remove |
| `UNKNOWN_ID` | Read `content/` or grep for the id; never invent ids |
| `UNKNOWN_ROLE` | Use a role/variant from **this package's** theme |
| `WRONG_CONTENT` | Wrong node kind for a text edit — use cell ids for tables; see [writing/errors.md](writing/errors.md) |
| Poster/invoice spilled to page 2 | `compile` prints `pages=N`; use `--expect-pages 1` |
| `--font` PermissionError / cannot copy | Copy the `.ttf`/`.otf` to a readable path; do not use locked OS font dirs. Script fails closed (no skip) |
| Viewer `BROKEN_INTEGRITY` | Content changed without relock — run `pack_verify.py` |
| Want Word-like layout | Theme + full relock, not per-node x/y |
| Used `Editor.insert_node` on author dir | Wrong API — edit JSON files, then pack |

## Optional Gallery (MCP)

Not required. Do not install or configure MCP for this skill. If the user already gave a Gallery package URL, fetch it directly — skip `list_templates`. If `list_templates` and `download_template` are already available:

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
| Expect whole paragraphs to jump to next page | Text `break_inside: auto` splits **by line**; a zero-padding stack splits **by child** when the remainder is too small. A padded/grid/overlay section stays atomic — split into sibling nodes |
| `break_before: page` on every figure | Only when the figure must start a page. Flowing article: **one** unpadded `columns` container; full-width figure/table as a child with `column_span: "all"`. Do not split into `p0.columns` / `p1.columns`. Padded columns stay atomic (whole block moves) |
| `$...$` inline math in author JSON | U+FFFC + `{ "type": "math", "intent": "<tex>" }` modifier — see `catalog/content/ex_modifiers.json`; `$` works Markdown only |
| Expect `\mathbb` / `\forall` / `\prime` / `\text` unsupported | They are in the TeX whitelist ([errors.md](writing/errors.md)); add NotoSansMath. No `\color`/`\textcolor`/`\tag`/`\mathbf`/`\sqrt[n]` — role color + `ex_math_numbered.json` |
| Multiple fonts but `"default":"default"` only | Map `font_aliases` in **`styles/theme.json`** (not manifest) to each file stem; two+ fonts have no auto-`default`. Point each role `font_family` at a stem. |
| `font_aliases` on `manifest.json` | Theme-only. See [package.md](writing/package.md#theme). |
| Clone `th_light_*` / `th_dark_*` roles | Same semantic roles + `variant: "on_dark"` (`ex_on_dark.json`). Cover chrome: dedicated cover roles or that variant — not a second role tree. |
| Skip modifier `intent` | Required by schema. Use a key from `theme.modifiers.styles[type]` (`default`, `strong`, URL, …). |
| Require `rows: [{"auto":true}]` on a one-row grid | Optional. Omit `rows` → engine fills `{auto:true}`. Write `rows` only for `fr`/`pt` or a fixed track list (`ex_split_bar.json` / `ex_poster_shell.json`). |
| Symmetric `ex_grid.json` for a magazine image+copy row | Copy `ex_media_row.json` — `columns: [{pt:N},{fr:1}]`. Title+logo hug-right → `ex_split_bar.json`. |
| Dense dashboard table from `ex_table.json` only | Copy `ex_table_dense.json` (weighted `fr` + cell `compact`). Wrap in `card` / `on_dark` / `ex_glass.json` as needed. Long English tokens still need a wider column or U+00AD. |
| `--render preview.png` missing in CWD | Bare name lands in **`<output.K2F parent>/tmp/`**, not CWD and not `source/`. Open the path printed as `ok: rendered …`. |
| `fr` rows without fixed grid height | Fails: `Cannot resolve fr tracks with infinite available size`. Not CSS Grid — `fr` ≠ content-auto height. Set grid `layout.height`, use `pt`/`auto` rows, or nest under a fixed-height stack (`ex_grid.json` / `ex_poster_shell.json`) |
| Omit grid `rows` like CSS implicit tracks | Allowed only as content-auto wrapping (`ceil(n/cols)` `{auto:true}`). `fr`/`pt` must be written; declared `rows` do not grow (`ex_split_bar.json` / `ex_poster_shell.json`) |
| Poster/slide/invoice shell is a vertical stack | Content piles at the top. Copy `ex_filled_page.json` / `ex_poster_shell.json`: pinned `height` + `{fr:1}` body row |
| `{fr:1}` on a short quote / last thin card; no `LAYOUT_SLACK` | Box grew; type did not. Leftover → figure or **equal** `{fr:1}` siblings with enough copy (`ex_poster_growers.json`). Silence / footer at the bottom ≠ interiors filled |
| `p1`/`p2` page containers | Don’t invent pages. One flow tree; engine fills. `break_before: page` on a chapter / annex / signature node is fine |
| Cover year in the footer / vertical space-between | Copy `ex_cover.json` / `ex_poster_shell.json` (`{auto:true}` + `{fr:1}` + `{auto:true}`), not padding guesses or empty spacers. Flow-only (footer not at page bottom) → vertical stack, not the `{fr:1}` shell |
| Letter sender / right-flush cell | Copy `ex_end_block.json` (horizontal `justify_content: end` wrapping a content-width vertical stack). Left+right pair → `ex_split_bar.json`. Do not `text_align: end` on each line |
| Expect small-caps / `font_variant` / drop cap | Not in v0.1. Small-caps: content uppercase + role `letter_spacing_pt`. Drop cap: large first-letter text node beside body in a 2-col grid — not a modifier |
| Badge stretched across a grid cell | `{auto:true}` sizes the **track**, default `cell_align.x` is still stretch. Copy `ex_badge.json` (stack wrapper + role `self_align: start`). Whole grid hug → `cell_align.x: "start"`. Grid does **not** read `self_align` on a direct child. |
| Empty `role: "rule"` is a square dot / width 0 | Stack `align_items` **defaults to stretch**, not start. Collapse = parent `align_items: start` / horizontal stack / overlay without `width`. Keep the rule in a vertical stretch stack (`ex_rule.json`). |
| Hide header on cover / odd-even page numbers | `running_blocks` repeat on **every** page (no skip-first / odd-even). Cover-only: omit them, chrome in `ex_cover.json`. Signature: content `signature_block`, not a last-page footer. Split title + page: copy `catalog/manifest.json` Grid. No `{{chapter}}` placeholder. |
| Child paint past a rounded parent | `corner_radius` clips **that box's** fill only — no `overflow`. Same corner name on the full-bleed child, or parent `padding_pt`. |
| Square outline around a rounded fill | Four-edge `border` follows `corner_radius`. Do not wrap a second box. Stale CLI: `pip install -U k2f`. Partial `edges` (`subtle_bottom` / `hbar`) stay square by design. |
| Expect per-cell grid align or baseline | Nest stack / theme `self_align` (stack/table only) / `ex_end_block.json` in the right cell; `cell_align.y: start` — no first-line baseline |
| Binding gutter + title centered on the sheet | `margin` 4-tuple is the gutter; `text_align: center` is the **content box**. Overlay or equal padding on that title role |
| Academic serif missing from starter | `--add-font` a serif TTF; starter ships Roboto only |
| Inline code pills / modifier background | Sibling `role: code`, or a **text** node with a dedicated role (`ex_badge.json`) — not an inline background patch and not a wrapper just for fill |
| Variant `text_align` / `bold` rejected | Put those under `text_overrides` on the variant, not at the variant root |
| Require `row_gap`/`column_gap`/`cell_align` | Optional on **grid and table** — see `schema/nodes.schema.json`; omit unused keys (`null` ok) |
| Expect table `colspan` / `vertical_align` | Extra columns + cell `variant: "hbar"` / `"bottom"` (`ex_table_edges.json`); table `variant: "ruled"` + header `bottom` for three-line; `variant: "end"` for numeric; `variant: "center"` for vertical middle |
| Expect a native `Divider` node | Use `role: "rule"` + small `layout.height` + surface fill (or bottom border). Parent vertical stack must keep default `align_items` stretch |
| Noise / vignette / radial glow / dot matrix / organic blob / per-corner radii | Not in core — SVG `<path>` under `assets/images/` (labels as `<path>`; no `<text>`/`<tspan>`/`<textPath>`/`<foreignObject>`); `corners` are one named radius for all four corners. Glass: `ex_glass.json` (catalog theme), not a radial fill |
| Unicode superscript (`²`) for notes | Ordinary char + `superscript`/`subscript` modifier; formulas → math |
| SVG `<text>` labels | Convert to `<path>` — `compile` rejects `<text>`/`<tspan>`/`<textPath>`/`<foreignObject>` (not only render). XML comments / CDATA mentioning those tags are ignored. Paint still fail-closed |
| Compact table splits `LATENCY` mid-word | Word wider than the cell is force-split. Widen `column_widths`, lower that cell role's `font_size`, or insert U+00AD. No hyphenation dictionary, no auto-shrink |
| One text node with `\n\n` for paragraphs | One paragraph = one text node; `\n` is a hard line break (each line still takes `line_height_mult`). Paragraph spacing = sibling `gap` |
| Simulate margin with padding / empty spacer stacks | No node margin or node `padding_pt`. Even rhythm: parent `gap`. Uneven: nested stacks with different `gap`, or a **dedicated** role's `padding_pt`. Shared `h1`/`body` padding applies to every such node |
| Dingbat/arrow/CJK glyphs (★ ◆ → ↗ ↑ 中文 かな) in Roboto | `FONT_MISSING_GLYPH` — `--add-font` a covering face (JP/KR/SC as needed). NotoSansSC ≠ Japanese. Math formulas → NotoSansMath. Do not rewrite user language to English |
| Expect `canvas_mode: "slide"` | v0.1 is `paged` only — use `--page widescreen --margin 0` + `ex_poster_shell.json` (`page_shell`, height `540000`) + `break_inside: avoid` |
| Expect `justify_content: space-between` or page `background` | Copy `ex_split_bar.json` / `ex_end_block.json` / `ex_poster_shell.json` / `ex_overlay.json` — not Flexbox or `page_config` |
| Overlay nested stack `align_items: end` not on the page right | Overlay children shrink to content unless that layer sets `width`; left/right bars → `ex_split_bar.json`; trailing-edge block (letter sender) → `ex_end_block.json` |
| Hand-count modifier ranges across `\n` | `\n` is 1 UTF-8 byte — run `modifier_range.py --text` with the exact node `value` |
| Pixel formula for cover padding vs line-height | Copy `ex_cover.json`; iterate the PNG. Line boxes + `padding_pt` + `gap` **add**; do not invent spacer nodes or cancel line boxes with padding math |
| `layout.height` + padding overflowing the page | Height is min outer; padding is inside. `inner_h = height − pad_t − pad_b`; `{fr:1}` uses that. Copy `ex_poster_shell.json` (`page_shell`) — do not nest another full-page-height child. |
| Trust `preview.png` alone | `--render` also writes `preview-1.png` …; check `pages=N` / `--expect-pages` |
| Ship after `UNSIGNED` without opening the PNG | `PAGE_UNDERFILL` is warning-only — still open every PNG — [visual check](#visual-check) |
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
