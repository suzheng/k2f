# Authoring an unpacked K2F package

Domain conventions for fixed layouts (CV, flyer, cheatsheet). **Allowed JSON keys:** [`schema/`](../../schema/). **Golden shapes:** [`catalog/content/ex_*.json`](../../catalog/content/) ([index](../../catalog/README.md)). Copy those files; do not invent syntax.

**Do not use in this skill's writing workflow** (schema-legal but unsupported here): `content.type: "table_reference"`, table `data.type: "asset"`, `content.type: "code_block"` (use `role: "code"` + text — see `ex_code.json` and [converting-markdown.md](../converting-markdown.md#mapping)).

## Required loose files

| Path | Notes |
|------|-------|
| `manifest.json` | `canvas_mode: "paged"`, `page_config` (width/height/margin in millipt) |
| `content/root.json` | Semantic tree entry; may reference fragments via `{ "include": "content/...." }` |
| `content/**/*.json` | Optional; one node per file, referenced from root (see below) |
| `styles/theme.json` | Roles, palette, named primitives (do not split across files) |
| `assets/fonts/*` | At least one font (starter ships Roboto-Regular) |
| `assets/images/*` | Optional PNG/JPEG/WebP/SVG (not `assets/` root; not GIF) |
| `changelog.json` | Optional; pack fills `{"entries":[]}` if missing |

**`k2f pack` injects the five format schemas** from the engine. Do **not** put files under `schema/` in the author directory (`UNEXPECTED_PATH`).

## Splitting long content

For theses, long reports, or large slide decks, split by chapter or slide group:

1. Keep cover/title/running structure in `content/root.json`
2. Add `{ "include": "content/ch01.json" }` stubs in container `children`
3. Each fragment file is one `SemanticNode` (container nodes may wrap a chapter's children)
4. Every file under `content/` must be referenced — no orphan JSON, no directory globbing

Pattern: [`catalog/content/root.json`](../../catalog/content/root.json) (includes `ex_*.json`).

## Units and page sizes

All lengths are **millipt** integers (1/1000 pt).

| Page | `--page` | width × height |
|------|----------|----------------|
| A4 | `a4` | `595000` × `842000` |
| Letter | `letter` | `612000` × `792000` |
| A4 landscape | `a4-landscape` | `842000` × `595000` |
| 16:9 (PPT-like) | `widescreen` | `960000` × `540000` |
| 4:3 | `widescreen-43` | `720000` × `540000` |

v0.1 is still `canvas_mode: "paged"` — widescreen is a page size, not a slide mode. `margin` is `[top, right, bottom, left]`. `init_package.py --margin 36000` (or four millipt values) overrides the preset. Custom canvas: `--width` / `--height` millipt together (override the `--page` preset). mm → millipt: `round(mm * 72000 / 254)` (e.g. 700×1000mm → `1984252×2834646`). Images must declare width/height in millipt (engine does not probe binary size for layout).

### 16:9 multi-page slide deck (paged)

There is **no** `canvas_mode: "slide"`. Use paged pages as slides:

1. `python scripts/init_package.py --dir ./out/deck --title "Deck" --page widescreen --margin 0` → `960000×540000`, zero page margin.
2. Under `root`, each top-level sibling is one slide.
3. Each slide: copy [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json) — `break_inside: "avoid"` + page-size grid (`width`/`height` `960000×540000`). Rows: `{auto:true}` header, `{fr:1}` body (the grower), `{auto:true}` footer. Do **not** use a vertical stack as the slide shell.
4. Safe inset = **inner** role `box_decoration.padding_pt` (not page margin). Keep **root** padding at 0 — root padding is added into `page_config.margin` and shrinks the content box.
5. Do not nest another full-page-height child inside a padded slide shell (see box model below).

Two-column body: nested grid `{fr:1},{fr:1}` **inside** the grower row (already in `ex_poster_shell.json`). Overlay backgrounds: `ex_overlay.json` wrapping that shell.

### Single-page poster / flyer

1. Pick a canvas: `--page a4` (or letter / a4-landscape), or `--page a4 --width W --height H` for non-standard sizes; full-bleed `--margin 0`.
2. Under `root`, **one** child: copy [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json). Set `layout.height` = page height − margins (A4 / margin 0 → `842000`). Same grid as slides: `{auto:true}` + `{fr:1}` + `{auto:true}`.
3. Header/footer are measured; leftover height goes to `{fr:1}`. Do not use a vertical stack as the page shell; no empty spacer containers.
4. Full-bleed background: wrap the shell in `overlay` with the background child first (`ex_overlay.json`).
5. Verify with `python scripts/pack_verify.py <dir> -o out.K2F --expect-pages 1 --render preview.png`. `compile`/`verify` print `pages=N`. `LAYOUT_SLACK` is a warning only (exit 0) — open the PNG; leave the gap if it is intentional. `preview.png` is **only page 0**. A page-height `break_inside: avoid` **stack** with large `padding_pt` is the usual `UNSPLITTABLE_OVERFLOW` path — use the grid shell first, then tune inner `gap` / padding.

### Thesis / report cover

Pick the shell from the intent:

1. **Year / affiliation pinned to the page bottom** — copy [`ex_cover.json`](../../catalog/content/ex_cover.json) (or `ex_poster_shell.json`). Set `layout.height` to page height − margins. Rows: `{auto:true}` header, `{fr:1}` leftover, `{auto:true}` footer. The `{fr:1}` **is** the large vertical span — do not fake it with empty spacers or by padding every text role.
2. **Flow only** (blocks from the top; footer not at the page bottom) — a vertical `stack`. Do **not** use the 3-row `{fr:1}` shell.

Uneven rhythm **inside** a band: nest stacks with different `gap` (the header in `ex_cover.json`: tight titles group vs larger gap to author). Extra space around **one** line: a dedicated role or variant in `theme.json` with `box_decoration.padding_pt`. Never put `padding_pt` on the node. Do not put it on shared `h1`/`body` unless every such node should get it. Then open the PNG; there is no padding-vs-line-height formula.

## Layout

Allowed layout types and fields: `schema/nodes.schema.json` → `layout`. Copy from catalog rather than inventing fields.

| Need | Use |
|------|-----|
| Continuous article columns | `columns` — `ex_columns.json` |
| Fixed side-by-side (sidebar, header bar) | horizontal `stack` or 2-col `grid` |
| Poster / slide page shell | `ex_poster_shell.json` — pinned height + `{auto:true}` header/footer + `{fr:1}` grower |
| Poster / dashboard cells | `grid` with `pt` tracks, or `fr` **after** a finite outer height — `ex_grid.json` |
| Full-page background + content | `overlay` (background child first) — `ex_overlay.json` |
| Title left + logo/meta right | `ex_split_bar.json` — `{fr:1}` + `{auto:true}` (not overlay, not Flexbox) |
| Trailing-edge block (letter sender; right-flush cell) | `ex_end_block.json` — horizontal stack `justify_content: "end"` wrapping a content-width vertical stack. Nest **inside** an equal-width grid cell. Left+right pair → `ex_split_bar.json` |
| Thesis / report cover | `ex_cover.json` — page-height grid as `ex_poster_shell`; header nests stacks with different `gap` |

- Grid tracks: `{ "pt": N }`, `{ "fr": N }`, or `{ "auto": true }` — never bare integers. `{auto:true}` is layout-grid only (not table `column_widths`).
- Do **not** put `grid` directly on the root node (awkward cross-page behavior). Nest grid under a child container, e.g. `root` → `root.grid`.
- **`fr` rows need a finite outer height.** In a vertical flow with unbounded height, `rows: [{fr:1}]` fails with `Cannot resolve fr tracks with infinite available size`. `fr` divides a **known** outer size — it is **not** content-auto height (`{auto:true}` is). Fix: set the grid's own `height`, use `pt`/`auto` rows, nest under a fixed-height stack/overlay, or prefer `stack` / native `table`.
- **Pin content to column bottom** (footnotes / correspondence): fixed-height 2-row grid — `height` + `rows: [{"fr": 1}, {"pt": N}]` with body in row 0 and footer text in row 1. No footnote node and no `space-between`.
- Titles for multi-column flow: keep outside a `columns` container; use `column_span: "all"` for full-width figures inside.
- **Overlay is in-flow stacking**, not absolute positioning: children share one origin; height = max(children); later children paint on top (no `z-index`). The block is unsplittable across pages. Optional `width`/`height` pin the **overlay box**; children still measure **independently** and shrink to their content unless that child sets its own `layout.width`/`height` (or is a grid/`fr` filling a pinned overlay). A nested stack without `width` is only as wide as its text — `align_items: "end"` then cannot pin to the page/overlay right edge. Overlay has **no** `align_items` / `justify_content` — to center a layer, nest a stack whose `width`/`height` match the overlay, with `justify_content: "center"` and `align_items: "center"`. Full-page backgrounds: pin overlay to content-box size (page minus margins); image child first (`ex_overlay.json`).
- **No `justify_content: space-between`.** Left/right split (title + logo, header logos): copy [`ex_split_bar.json`](../../catalog/content/ex_split_bar.json) — `columns: [{fr:1},{auto:true}]`. The `{fr:1}` track consumes leftover width, so the `{auto:true}` column sits on the trailing edge. Grid has no per-column `align_items`; do **not** switch to overlay + `align_items: start/end` for this. Right cell may be text or an image node. **Block on the trailing edge, lines still left-aligned** (letter sender, right-flush image in an equal-width cell): copy [`ex_end_block.json`](../../catalog/content/ex_end_block.json) — outer horizontal stack `justify_content: "end"` wrapping a content-width vertical stack. Nest that shape **inside** the right grid cell; do not `text_align: end` on each line (that ragged-left the block). Cover / slide vertical fill: [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json) or [`ex_cover.json`](../../catalog/content/ex_cover.json) (`{auto:true}` + `{fr:1}` + `{auto:true}`), not padding guesses. Pin a footer to the page bottom: same 2-row/3-row grid (`{fr:1}` grower + `{auto:true}` or `{pt:N}` footer).
- **Grid `cell_align` is one default for every cell.** Different x/y per cell: nest a stack with `align_items` / `justify_content` in that cell (right-flush → `ex_end_block.json`), or give the child a theme `self_align`. Do not add per-cell fields on the node. No first-line **baseline** align — use `y: "start"` (or keep both cells single-line).
- **`text_align: center` is the content box**, not the physical page. `page_config.margin` `[top,right,bottom,left]` already does binding gutters. A title optically centered on the sheet with a wider inner margin needs overlay at content width or equal left/right padding on that title role — not a second centering origin.
- **No empty spacer containers.** Do not insert a child whose only job is `layout.height` to invent gaps — that is geometry, not semantics. Even rhythm: parent stack `gap`. Uneven rhythm: nest stacks with different `gap`, or `padding_pt` on a **dedicated role/variant** in `theme.json` (not on the node; not on shared `h1`/`body` unless every instance should pad). `role: "rule"` with a small height is a semantic divider, not a spacer — copy [`ex_rule.json`](../../catalog/content/ex_rule.json) as a **sibling child** in the parent stack/grid (`content.container` + empty `children` + `layout.height`). Cover/title pages: see [Thesis / report cover](#thesis--report-cover).
- **Line box vs gap vs `\n` (no conversion formula).** Role `line_height_mult` is thousandths (`1350` = 1.35 × `font_size`). Each wrapped or `\n`-broken line in **one** text node occupies that line box. A 3-line `h1` is about `3 × font_size × line_height_mult/1000` tall; the next sibling starts after that whole box + parent `gap` + both roles' `padding_pt`. `\n` is a hard line break, **not** paragraph spacing — one paragraph = one text node; paragraph rhythm = sibling `gap`. Sequential padded containers add `padding.bottom` + parent `gap` + next `padding.top` **outside** the line boxes. Do not subtract line-height from padding. After pack, open the PNG and adjust `gap` / `font_size` / `line_height_mult` / dedicated-role `padding_pt`.
- **Box model (`layout.height` + `padding_pt`):** `layout.height` / `width` is a **minimum outer** size. Children are laid out in the **inner** box (`outer − padding`). Measured outer is `max(content+padding, hint)`. If an inner child is also set to full page height, outer grows to `content + padding` and with `break_inside: "avoid"` → `UNSPLITTABLE_OVERFLOW`. Recipe: outer shell = page size (`ex_poster_shell.json` / `ex_cover.json`); padding on that role (or an inner wrapper); children size to the **remaining** inner area — do not re-declare full page height on padded descendants, and do not use a page-height `avoid` stack with large paddings as the shell. **Root** `padding_pt` inflates page margins (see slide recipe).
- **Paged flow:** `break_inside: auto` (default) splits text **by wrapped line** and stacks **by child** when the page remainder is too small. `break_inside: avoid` never splits. `break_before: "page"` starts the node on a new page. `keep_with_next` keeps this node with the next sibling when both fit. Unsplittable siblings (overlay, `avoid`, padded card) move whole to the next page.
- **`manifest.running_blocks`:** repeating header/footer on **every** page including page 1 (no skip-first / odd-even). Each `node` is a normal semantic node. **Box** position in the margin band: role `self_align` (`start`/`center`/`end`). **Text** inside the box: role `text_align`. Placeholders `{{page_current}}` / `{{page_total}}` only. Keep running blocks short. Example: `catalog/manifest.json`.

## Theme

**Theme-only styling.** Style fields live in `styles/theme.json` roles/variants only — never on nodes in `root.json`. Nodes may set `role`, `variant`, `layout`, `modifiers`, `break_inside`, `break_before`, `keep_with_next`, `column_span`. Allowed role/theme keys: `schema/styles.schema.json` + `schema/visual_primitives.schema.json`.

Typography, spacing, and layout intent for slides, posters, and styled reports belong in the **design spec** ([writing.md](../writing.md)); implement them as roles and variants in `styles/theme.json`. Layout recipes (box model, poster shell, `ex_poster_shell.json`) stay in this file — do not copy a pre-made skin unchanged.

- Colors only via `palette` (keys or `#RRGGBB` / `#RRGGBBAA`).
- `box_decoration` fields are **named primitive strings** only → unknown name = `UNKNOWN_PRIMITIVE`.
- `padding_pt` counts toward measured outer height; with a fixed `layout.height`, children see the inner box (outer − padding). See box model above.
- **Paragraphs:** one text node per paragraph. `\n` inside a node is a hard line break only — not paragraph spacing. Use sibling `gap` between text nodes for paragraph spacing. Line-box math: see Layout above.
- **Multi-line titles:** one heading node with `\n`. Do not split one title into several heading nodes. Each `\n` line still takes a full `line_height_mult` box, which pushes later siblings down. Modifier `range` is UTF-8 **bytes** on the full `value` (`\n` is 1 byte) — always run `python scripts/modifier_range.py --text "<exact value>" --find "…"`; do not hand-count character indices.
- **Display math:** `role` must be `"math"` with `content.type = "math"`. Custom roles cannot wrap math content; use `variant` for visual skins. Any formula (display, inline modifier, or math glyphs in body) needs **NotoSansMath** in `assets/fonts/` plus `font_aliases` and the `math` role `font_family` — starter ships Roboto only; copy from [`catalog/assets/fonts/`](../../catalog/assets/fonts/). TeX is a **whitelist** ([errors.md](errors.md)); unknown commands → `MATH_UNSUPPORTED` — rewrite to the subset, do not expand the engine. No `\color` / `\textcolor` (the `math` role color applies to the whole formula). Numbered equations: copy [`ex_math_numbered.json`](../../catalog/content/ex_math_numbered.json), not `\\tag`.
- **Inline math in author JSON:** U+FFFC in the text value plus a node modifier `{ "type": "math", "intent": "<tex>", "range": [byte_start, byte_end] }`. `$...$` in JSON is **not** parsed — that syntax is Markdown only. See `ex_modifiers.json`. Modifier types and `range` rules: `schema/nodes.schema.json`.
- **Modifiers:** visual patches come from `theme.modifiers.styles[type][intent]`. Run `modifier_range.py` for byte offsets (max 50). Example node + theme patch: `ex_modifiers.json`.
- **Full-bleed bars:** set `page_config.margin` to `[0,0,0,0]`, keep root padding 0, then wrap content in an inner container with padding. There is no separate bleed primitive.
- **Dividers:** copy [`ex_rule.json`](../../catalog/content/ex_rule.json) into a stack/grid `children` list — `role: "rule"`, empty container, small `layout.height`, surface fill from the `rule` role. Alternatively a content box whose border uses `edges: ["bottom"]` / dashed style (named primitive, same as table cells below).
- **Fills / glass:** `primitives.gradients` support **linear** fills only (no radial). Named `box_decoration.blur` + `primitives.blurs` give backdrop blur / glass on boxes. Radial glow or soft vignettes → SVG under `assets/images/`. `box_decoration.shadow` is **box** elevation only — there is no text-shadow on glyphs.
- **Native table (inline):** each cell is a full semantic node. Minimal shape: `catalog/content/ex_table.json`. No `colspan`/`rowspan` — extra columns and hide the middle stroke with a named border `edges` (starter: cell `variant: "hbar"` = top+bottom, `"bottom"` = underline only). Copy [`ex_table_edges.json`](../../catalog/content/ex_table_edges.json). **Vertical middle** in a tall row: cell `variant: "center"` (starter maps that to role `self_align: "center"`). Do not invent `vertical_align` on the node.
- **Composite table cells:** a cell may be a `container` + `stack` holding text / `list_item` children. `list_item` must be **text** content with a non-empty `list_id` (not a nested list container). `list_style` on `list_item` is optional — omit for starter defaults; set only to override marker width/gap/indent.
- **`font_aliases`:** maps role `font_family` names to embedded font **keys** (filename stems under `assets/fonts/`). One font file also registers as `"default"`. **Two or more fonts:** no auto-`default` — every file stem must appear in `font_aliases`, and each role's `font_family` must be one of those stems. CSS generic families fail compile. `--add-font` copies the file and adds the alias; it does **not** retarget roles — point `math` (or headings) at the new stem yourself. Catalog already does this for Roboto + NotoSansMath (`catalog/styles/theme.json`):

  ```json
  "font_aliases": {
    "Roboto-Regular": "Roboto-Regular",
    "NotoSansMath-Regular": "NotoSansMath-Regular"
  }
  ```

  Copy `catalog/assets/fonts/NotoSansMath-Regular.ttf` (or `--add-font` that file), set the `math` role `font_family` to `NotoSansMath-Regular`. Body text with math glyphs still needs that face embedded (package fonts fall back to each other).
- Format `role` is an open string but **must exist** in `theme.roles`. Custom roles are expected.

Default published trees should avoid shadow/blur unless PDF export with stamp is intended. Linear gradients, shadows, and backdrop blur on a page trigger the PDF stamp path (`k2f_paint` at scale 2/3/4, default 2).

**Bundled font:** starter ships `Roboto-Regular.ttf` only. Latin + common punctuation. **Not** a serif face, arrows (`→` `↗` `↑`), stars/dingbats (★ ◆), math (`∈`), or **CJK/kana**. Missing glyphs → `FONT_MISSING_GLYPH` (hard fail; no OS fallback). Embedded package fonts **do** fall back to each other.

- Academic / book serif: `init_package.py --add-font /path/to/Serif.ttf` (keeps Roboto) and point heading roles at that stem via `font_aliases`. Do not expect a bundled Libertine/DejaVu Serif.

- Japanese / Korean / full CJK: `init_package.py --add-font /path/to/NotoSansJP.otf` (keeps Roboto; kana uses the extra face). `--font` **replaces** Roboto and retargets theme roles — use a covering face that includes Latin. **NotoSansSC is Simplified Chinese**, not Japanese kana.
- Math symbols in body text also need a math/CJK face, or put them in `role: "math"` with NotoSansMath.
- Do **not** rewrite the user's language to English to dodge a missing glyph.

## Images

Embed **PNG, JPEG, WebP, or SVG** under `assets/images/` only (not `assets/` root, **not GIF**). SVG is rasterized **without system fonts**: `<text>` / `<tspan>` **fail** (no silent drop) — convert labels to `<path>` (or a K2F text node beside the image). Declare width/height in millipt on the image node.

Default stack `align_items` is `stretch`: the image **box** fills the cross axis and paint letterboxes (contain, centered) inside it. Left/right: set the image role's `self_align` to `start`/`end` in `theme.json` (never on the node). See `catalog/content/ex_image.json`.

## Workflow

From this skill directory. Default font is `starter/assets/fonts/Roboto-Regular.ttf`.

Always use `scripts/pack_verify.py`. It uses `k2f` on PATH or `K2F_CLI`. A stale binary may reject documented keys — `pip install -U k2f`; do not strip valid JSON. The author directory can live outside this skill folder; pass its path. The script does **not** search a git checkout or cargo build directory.

```bash
python scripts/init_package.py --dir <out_dir> --title "..." --page a4|letter|a4-landscape|widescreen|widescreen-43
# slides: --page widescreen --margin 0
# posters: --page a4 --margin 0
# custom size: --width 1984252 --height 2834646  (with --page for margin defaults)
# optional: --margin 36000   or   --margin 56000,56000,56000,56000
# edit content/root.json and styles/theme.json (copy from catalog/content/ex_*.json)
python scripts/pack_verify.py <out_dir> -o <out.K2F>
# preview + single-page gate (compile stderr includes pages=N)
python scripts/pack_verify.py <out_dir> -o <out.K2F> --expect-pages 1 --render preview.png
```

`init_package.py` copies [`starter/`](../../starter/). `--font` replaces Roboto and rewrites role `font_family`. `--add-font` copies extra faces beside Roboto for glyph fallback (CJK, math).

## Capability limits

No arbitrary vectors in State A, no native geometry/divider nodes (use decorated boxes, `role: "rule"`, or SVG), no node-level margin/bleed, no Flexbox `space-between`, no `z-index`/absolute coordinates, no slide/infinite `canvas_mode` (v0.1 is `paged` only — use the 16:9 / poster recipes above), no page/node animations or transitions, no native chart/graph nodes (use `table` or PNG/WebP/SVG), no radial gradients, no noise/grain/vignette/pattern fills, no text-shadow or glyph blur (box `shadow` / box `blur` only), no `font_variant`/small-caps, no rotation/`writing_mode`, no inline text background/border patches (inline code = sibling `role: "code"` node, not a modifier pill), no ruby/chord layout, no grid first-line baseline align. Shapes are decorated boxes or image/SVG assets. If a design needs missing engine features, record gaps; **do not** change engine code or schemas to force a match.
