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
| `assets/fonts/*` | At least one `.ttf`/`.otf` (starter ships Roboto-Regular). License `.txt` under `fonts/licenses/` may stay; load skips non-faces. |
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

There is **no** `canvas_mode: "slide"` and no separate `slide_deck_starter`. Use paged pages as slides:

1. `python scripts/init_package.py --workspace ./out/deck --title "Deck" --page widescreen --margin 0` → `960000×540000`, zero page margin. Author JSON lives in `./out/deck/source/`.
2. Under `root`, each top-level sibling is one slide.
3. Each slide: copy [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json) — role `page_shell`, `break_inside: "avoid"`, `layout.height: 540000` (replace the demo `240000`). Rows: `{auto:true}` header, `{fr:1}` body (the grower), `{auto:true}` footer. Do **not** use a vertical stack as the slide shell. Inside the grower: leftover on a **figure** or dense `{fr:1}` siblings ([`ex_poster_growers.json`](../../catalog/content/ex_poster_growers.json)) — not a short quote; `{fr:1}` does not enlarge type.
4. Safe inset = role `page_shell` `padding_pt` (starter `36000`). Keep **root** padding at 0 — root padding is added into `page_config.margin` and shrinks the content box. Do not pad shared `section`/`body`.
5. Do not nest another full-page-height child inside the padded shell (see box model below). `--expect-pages` equals the number of slides.

Two-column body: nested grid `{fr:1},{fr:1}` **inside** the grower row (already in `ex_poster_shell.json`). Overlay backgrounds: `ex_overlay.json` wrapping that shell.

### Single-page poster / flyer

1. Pick a canvas: `--page a4` (or letter / a4-landscape), or `--page a4 --width W --height H` for non-standard sizes; full-bleed `--margin 0`. Use the millipt table above (`595000×842000`), not ISO `595280×841890`.
2. Under `root`, **one** child: copy [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json) (role `page_shell`). Set `layout.height` = content box (A4 / margin 0 → `842000`). Same grid as slides: `{auto:true}` + `{fr:1}` + `{auto:true}`.
3. Header/footer are measured; leftover height goes to `{fr:1}`. Do **not** pre-assign millipt to every band — that fights `auto`+`fr`. Do not use a vertical stack as the page shell; no empty spacer containers.
4. Full-bleed background: wrap the shell in `overlay` with the background child first (`ex_overlay.json`). The shell still carries the inset; the image child sets `layout.height` to the page.
5. Verify with `python scripts/pack_verify.py <source> -o <workspace>/<name>.K2F --expect-pages 1 --render preview.png`. Bare `preview.png` is written to `<workspace>/tmp/preview.png`. `compile`/`verify` print `pages=N`. Open the PNG: leftover must sit on a figure or dense `{fr:1}` siblings, not a hollow card. `LAYOUT_SLACK` is warning-only and can miss interiors. `preview.png` is **only page 0**. A page-height `break_inside: avoid` **stack** with large `padding_pt` is the usual `UNSPLITTABLE_OVERFLOW` path — use the grid shell first, then tune inner `gap` / padding.

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
| Poster / slide page shell | `ex_poster_shell.json` — role `page_shell` (safe inset), pinned height + `{auto:true}` header/footer + `{fr:1}` grower |
| Grower vertical fill (stacked `{fr:1}` rows) | `ex_poster_growers.json` — leftover on a **figure** / dense card rows, not a short quote. `{fr:1}` stretches the box, not type |
| Poster / dashboard cells | `grid` with `pt` tracks, or `fr` **after** a finite outer height — `ex_grid.json` |
| Full-page background + content | `overlay` (background child first) — `ex_overlay.json` |
| Title left + logo/meta right | `ex_split_bar.json` — `{fr:1}` + `{auto:true}` (not overlay, not Flexbox) |
| Fixed-width figure + adaptive copy | `ex_media_row.json` — `{pt:N}` + `{fr:1}` (magazine image+text). Opposite of `ex_split_bar.json`. |
| Trailing-edge block (letter sender; right-flush cell) | `ex_end_block.json` — horizontal stack `justify_content: "end"` wrapping a content-width vertical stack. Nest **inside** an equal-width grid cell. Left+right pair → `ex_split_bar.json` |
| Dense metric table | `ex_table_dense.json` — weighted `fr` columns + cell `variant: "compact"`. Wrap in `card` / `on_dark` / `ex_glass.json`. |
| Dark cover / light interior | `ex_on_dark.json` — same roles + `variant: "on_dark"`. Do not clone `th_light_*` / `th_dark_*` roles. |
| Thesis / report cover | `ex_cover.json` — page-height grid as `ex_poster_shell`; header nests stacks with different `gap` |

- Grid tracks: `{ "pt": N }`, `{ "fr": N }`, or `{ "auto": true }` — never bare integers. `{auto:true}` is layout-grid only (not table `column_widths`). **`rows` is optional** — omit for uniform content-auto wrapping (including a **one-row** split): engine fills `ceil(n_children / n_columns)` `{auto:true}` tracks. Declared `rows` do **not** grow (extra children → error). Write `rows` when you need `fr`/`pt` or a fixed list. Grower / poster shells still copy [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json) with explicit `{auto,fr,auto}`. Magazine figure+copy: [`ex_media_row.json`](../../catalog/content/ex_media_row.json) (`{pt}`+`{fr:1}`; `rows` omitted).
- Do **not** put `grid` directly on the root node (awkward cross-page behavior). Nest grid under a child container, e.g. `root` → `root.grid`.
- **`fr` rows need a finite outer height.** In a vertical flow with unbounded height, `rows: [{fr:1}]` fails with `Cannot resolve fr tracks with infinite available size`. `fr` divides a **known** outer size — it is **not** content-auto height (`{auto:true}` is). Fix: set the grid's own `height`, use `pt`/`auto` rows, nest under a fixed-height stack/overlay, or prefer `stack` / native `table`.
- **Pin content to column bottom** (footnotes / correspondence): fixed-height 2-row grid — `height` + `rows: [{"fr": 1}, {"pt": N}]` with body in row 0 and footer text in row 1. No footnote node and no `space-between`.
- Titles for multi-column flow: keep outside a `columns` container; use `column_span: "all"` for full-width figures **inside the same** unpadded columns node (`ex_columns.json`). Do not `break_before: page` on the figure and do not split the article into several columns containers. **Padded columns are atomic** (same as padded/grid/overlay). Flowing paper body uses `columns`, not a 2-col grid.
- **Overlay is in-flow stacking**, not absolute positioning: children share one origin; height = max(children); later children paint on top (no `z-index`). The block is unsplittable across pages. Optional `width`/`height` pin the **overlay box**; children still measure **independently** and shrink to their content unless that child sets its own `layout.width`/`height` (or is a grid/`fr` filling a pinned overlay). A nested stack without `width` is only as wide as its text — `align_items: "end"` then cannot pin to the page/overlay right edge. Overlay has **no** `align_items` / `justify_content` — to center a layer, nest a stack whose `width`/`height` match the overlay, with `justify_content: "center"` and `align_items: "center"`. Full-page backgrounds: pin overlay to content-box size (page minus margins); image child first (`ex_overlay.json`).
- **No `justify_content: space-between`.** Left/right split (title + logo, header logos): copy [`ex_split_bar.json`](../../catalog/content/ex_split_bar.json) — `columns: [{fr:1},{auto:true}]`. The `{fr:1}` track consumes leftover width, so the `{auto:true}` column sits on the trailing edge. Grid has no per-column `align_items`; do **not** switch to overlay + `align_items: start/end` for this. Right cell may be text or an image node. **Block on the trailing edge, lines still left-aligned** (letter sender, right-flush image in an equal-width cell): copy [`ex_end_block.json`](../../catalog/content/ex_end_block.json) — outer horizontal stack `justify_content: "end"` wrapping a content-width vertical stack. Nest that shape **inside** the right grid cell; do not `text_align: end` on each line (that ragged-left the block). Cover / slide vertical fill: [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json) or [`ex_cover.json`](../../catalog/content/ex_cover.json) (`{auto:true}` + `{fr:1}` + `{auto:true}`), not padding guesses. Pin a footer to the page bottom: same 2-row/3-row grid (`{fr:1}` grower + `{auto:true}` or `{pt:N}` footer). Hollow grower: leftover on a figure or dense `{fr:1}` siblings ([`ex_poster_growers.json`](../../catalog/content/ex_poster_growers.json)) — `{fr:1}` stretches the box, not type; not an auto-height stack.
- **Grid `cell_align` defaults to stretch/stretch** (omit/`null` same). `{auto:true}` sizes the **track** from the widest cell — it does not hug the child; stretch still fills that track. Whole-grid hug (pills, logos): `cell_align.x: "start"`. One cell different from the rest: nest a stack in that cell (`ex_end_block.json` / `ex_badge.json`) and set the child's role `self_align` — grid does **not** read `self_align` on a direct child. Do not add per-cell fields on the node. No first-line **baseline** align — use `y: "start"` (or keep both cells single-line).
- **`text_align: center` is the content box**, not the physical page. `page_config.margin` `[top,right,bottom,left]` already does binding gutters. A title optically centered on the sheet with a wider inner margin needs overlay at content width or equal left/right padding on that title role — not a second centering origin.
- **No empty spacer containers.** Do not insert a child whose only job is `layout.height` to invent gaps — that is geometry, not semantics. Even rhythm: parent stack `gap`. Uneven rhythm: nest stacks with different `gap`, or `padding_pt` on a **dedicated role/variant** in `theme.json` (not on the node; not on shared `h1`/`body` unless every instance should pad). `role: "rule"` with a small height is a semantic divider, not a spacer — copy [`ex_rule.json`](../../catalog/content/ex_rule.json) as a **sibling child** in the parent stack/grid (`content.container` + empty `children` + `layout.height`). Cover/title pages: see [Thesis / report cover](#thesis--report-cover).
- **Line box vs gap vs `\n` (no conversion formula).** Role `line_height_mult` is thousandths (`1350` = 1.35 × `font_size`). Each wrapped or `\n`-broken line in **one** text node occupies that line box. A 3-line `h1` is about `3 × font_size × line_height_mult/1000` tall; the next sibling starts after that whole box + parent `gap` + both roles' `padding_pt`. `\n` is a hard line break, **not** paragraph spacing — one paragraph = one text node; paragraph rhythm = sibling `gap`. Sequential padded containers add `padding.bottom` + parent `gap` + next `padding.top` **outside** the line boxes. Do not subtract line-height from padding. After pack, open the PNG and adjust `gap` / `font_size` / `line_height_mult` / dedicated-role `padding_pt`.
- **Box model (`layout.height` + `padding_pt`):** `layout.height` / `width` is a **minimum outer** size. Children lay out in the **inner** box (`outer − padding`).

  Content-box height = `page_h − margin_top − margin_bottom` (A4 / 56pt margins → `842000−2×56000 = 730000`; A4 / margin 0 → `842000`; 16:9 / margin 0 → `540000`). Shell `layout.height` = that content box. If the shell role has padding: `inner_h = height − pad_t − pad_b`; `{fr:1}` shares `inner_h − sum(auto rows) − gap×(n−1)`. Starter `page_shell` padding is `36000` → 16:9 inner `540000−2×36000 = 468000`. Measured outer is `max(content+padding, hint)`. If an inner child is also set to full page/content-box height, outer grows to `content + padding` and with `break_inside: "avoid"` → `UNSPLITTABLE_OVERFLOW`. Recipe: outer shell = content-box size (`ex_poster_shell.json` / `ex_cover.json`); padding only on `page_shell` (or an inner wrapper role); children size to the **remaining** inner area — do not re-declare full page height on padded descendants, and do not use a page-height `avoid` stack with large paddings as the shell. **Root** `padding_pt` inflates page margins (see slide recipe).
- **Paged flow:** `break_inside: auto` (default) splits text **by wrapped line** and zero-padding vertical stacks **by child** when the page remainder is too small. `break_inside: avoid` never splits. `break_before: "page"` starts the node on a new page — do not put it on a figure unless that figure must start a page. `keep_with_next` keeps this node with the next sibling when both fit. Unsplittable siblings (overlay, `avoid`, padded card, grid, **padded columns**) move whole to the next page. Unpadded `columns` fill leftover height then continue. **Composed page** (one `avoid` shell per page) vs **flow** (root stack): see [writing.md](../writing.md#visual-check).
- **`manifest.running_blocks`:** repeating header/footer on **every** page including page 1 (no skip-first / odd-even / `exclude_pages`). Cover package: omit `running_blocks` and put chrome in [`ex_cover.json`](../../catalog/content/ex_cover.json). Last-page signature: content `signature_block`, not a running footer. Each `node` is a normal semantic node (text **or** a Grid/stack container). Split title + page number: copy the Grid in [`catalog/manifest.json`](../../catalog/manifest.json) (`{fr:1}`+`{auto:true}`, trailing cell `variant: "end"`). Placeholders `{{page_current}}` / `{{page_total}}` only — no `{{chapter}}` / section token; put a static string in the left cell. **Box** position in the margin band: role `self_align` (`start`/`center`/`end`). **Text** inside the box: role `text_align`. Keep running blocks short.

## Theme

**Theme-only styling.** Style fields live in `styles/theme.json` roles/variants only — never on nodes in `root.json`. Nodes may set `role`, `variant`, `layout`, `modifiers`, `break_inside`, `break_before`, `keep_with_next`, `column_span`. Allowed role/theme keys: `schema/styles.schema.json` + `schema/visual_primitives.schema.json`.

Typography, spacing, and layout intent for slides, posters, and styled reports belong in the **design spec** ([writing.md](../writing.md)); implement them as roles and variants in `styles/theme.json`. Layout recipes (box model, poster shell, `ex_poster_shell.json`) stay in this file — do not copy a pre-made skin unchanged.

- Colors only via `palette` (keys or `#RRGGBB` / `#RRGGBBAA`).
- `box_decoration` fields are **named primitive strings** only → unknown name = `UNKNOWN_PRIMITIVE`.
- `padding_pt` counts toward measured outer height; with a fixed `layout.height`, children see the inner box (outer − padding). See box model above.
- **Paragraphs:** one text node per paragraph. `\n` inside a node is a hard line break only — not paragraph spacing. Use sibling `gap` between text nodes for paragraph spacing. Line-box math: see Layout above.
- **Multi-line titles:** one heading node with `\n`. Do not split one title into several heading nodes. Each `\n` line still takes a full `line_height_mult` box, which pushes later siblings down. Modifier `range` is UTF-8 **bytes** on the full `value` (`\n` is 1 byte) — **before** writing modifiers, run `python scripts/modifier_range.py --text "<exact value>" --find "…"`; do not hand-count character indices. Multi-line example: `ex_modifiers.json` (`ex.modifiers.multiline`).
- **Display math:** `role` must be `"math"` with `content.type = "math"`. Custom roles cannot wrap math content; use `variant` for visual skins. Any formula (display, inline modifier, or math glyphs in body) needs **NotoSansMath** in `assets/fonts/` plus `font_aliases` and the `math` role `font_family` — starter ships Roboto only; copy from [`catalog/assets/fonts/`](../../catalog/assets/fonts/). TeX is a **whitelist** ([errors.md](errors.md)); unknown commands → `MATH_UNSUPPORTED` — rewrite to the subset, do not expand the engine. No `\color` / `\textcolor` (the `math` role color applies to the whole formula). Numbered equations: copy [`ex_math_numbered.json`](../../catalog/content/ex_math_numbered.json), not `\\tag`.
- **Inline math in author JSON:** U+FFFC in the text value plus a node modifier `{ "type": "math", "intent": "<tex>", "range": [byte_start, byte_end] }`. `$...$` in JSON is **not** parsed — that syntax is Markdown only. See `ex_modifiers.json`. Modifier types and `range` rules: `schema/nodes.schema.json`.
- **Modifiers:** required `range`, `type`, **`intent`**. Visual patches come from `theme.modifiers.styles[type][intent]` — missing `intent` → `SCHEMA_INVALID`. Run `modifier_range.py` for byte offsets (max 50). Example node + theme patch: `ex_modifiers.json`.
- **Full-bleed bars:** set `page_config.margin` to `[0,0,0,0]`, keep root padding 0, then wrap content in an inner container with padding. There is no separate bleed primitive.
- **Dividers:** copy [`ex_rule.json`](../../catalog/content/ex_rule.json) into a **vertical** stack/grid `children` list — `role: "rule"`, empty container, small `layout.height`, surface fill from the `rule` role. Stack `align_items` **defaults to `stretch`** (not `start`) — that is what gives an empty rule its width. Collapse to a dot/`0` when the parent is `align_items: start|center|end`, a horizontal stack, an overlay child without `width`, or a grid `{auto:true}` column whose only content is the empty rule. Fix: leave the parent at default stretch, or nest the rule as a sibling of a hugging group — do not put it inside the hugging stack. Alternatively a content box whose border uses `edges: ["bottom"]` / dashed style (named primitive, same as table cells below).
- **Fills / glass:** `primitives.gradients` support **linear** fills only (no radial). Named `box_decoration.blur` + `primitives.blurs` give backdrop blur / glass on boxes — copy [`ex_glass.json`](../../catalog/content/ex_glass.json) (catalog theme `glass_light` / `blurs.background`, not starter). Radial glow, noise, or vignettes → SVG under `assets/images/`. `box_decoration.shadow` is **box** elevation only — there is no text-shadow on glyphs.
- **Rounded full-bleed children:** `corner_radius` clips **this** box's fill/shadow/blur, not descendants. Give the overflowing child the same corner name, or pad the parent. No `overflow` / clip field.
- **Native table (inline):** each cell is a full semantic node. Minimal shape: `catalog/content/ex_table.json` (Qty uses cell `variant: "end"` → theme `text_overrides.text_align`). Dense / dashboard: [`ex_table_dense.json`](../../catalog/content/ex_table_dense.json) — weighted `{fr}` columns + cell `variant: "compact"` (starter). Optional `row_gap` / `column_gap` (omit → `gap`). No `colspan`/`rowspan` — extra columns and hide the middle stroke with a named border `edges` (starter: cell `variant: "hbar"` = top+bottom, `"bottom"` = underline only). Three-line: table `variant: "ruled"` (top+bottom on the table role) + header `bottom`. Copy [`ex_table_edges.json`](../../catalog/content/ex_table_edges.json). **Vertical middle** in a tall row: cell `variant: "center"` (starter maps that to role `self_align: "center"`). Do not invent `vertical_align` or `colspan` on the node. A word wider than its column is force-split — widen that `{fr}` / `{pt}` track, lower the cell role `font_size`, or insert U+00AD; no hyphenation dictionary.
- **Badge / pill:** copy [`ex_badge.json`](../../catalog/content/ex_badge.json) — a **stack wrapper** around a text node whose role has `box_decoration` + `self_align: "start"`. Grid `cell_align.x` defaults to stretch and **does not** read `self_align` on a direct child; dropping the inner text node into `ex_split_bar.json` stretches the pill. Do not add another box just for fill; do not put `box_decoration` on the node; do not invent a modifier pill.
- **Composite table cells:** a cell may be a `container` + `stack` holding text / `list_item` children. `list_item` must be **text** content with a non-empty `list_id` (not a nested list container). `list_style` on `list_item` is optional — omit for starter defaults; set only to override marker width/gap/indent.
- **`font_aliases`:** maps role `font_family` names to embedded font **keys** (filename stems under `assets/fonts/`). Lives in **`styles/theme.json`**, never `manifest.json`. One font file also registers as `"default"`. **Two or more fonts:** no auto-`default` (file order must not pick a silent fallback) — every file stem must appear in `font_aliases`, and each role's `font_family` must be one of those stems. CSS generic families fail compile. `--add-font` copies the file and adds the alias; it does **not** retarget roles — point `math` (or headings / body) at the new stem yourself. Catalog already does this for Roboto + NotoSansMath (`catalog/styles/theme.json`):

  ```json
  "font_aliases": {
    "Roboto-Regular": "Roboto-Regular",
    "NotoSansMath-Regular": "NotoSansMath-Regular"
  }
  ```

  Copy `catalog/assets/fonts/NotoSansMath-Regular.ttf` (or `--add-font` that file), set the `math` role `font_family` to `NotoSansMath-Regular`. Body text with math glyphs still needs that face embedded (package fonts fall back to each other).

  Academic serif body + sans headings (after `--add-font /path/to/SourceSerif4-Regular.ttf`, which keeps Roboto):

  ```json
  "font_aliases": {
    "Roboto-Regular": "Roboto-Regular",
    "SourceSerif4-Regular": "SourceSerif4-Regular"
  }
  ```

  Then set `body` (and `default`) `font_family` to `SourceSerif4-Regular`; keep `h1`–`h4` on `Roboto-Regular` (or the reverse). Do not leave leftover `"default": "default"` from an old theme.
- **Mixed dark cover / light interior:** keep the same semantic roles (`h1`, `body`, `card`, `table_row_cell`). Skin the dark band with `variant: "on_dark"` (starter + catalog). Copy [`ex_on_dark.json`](../../catalog/content/ex_on_dark.json). Do **not** clone `th_light_h1` / `th_dark_body` role trees — that leaks presentation into the semantic layer. Cover-only chrome can still be dedicated roles (`ex_cover.json`) if those nodes never appear in the interior.
- Format `role` is an open string but **must exist** in `theme.roles`. Custom roles are expected.

Default published trees should avoid shadow/blur unless PDF export with stamp is intended. Linear gradients, shadows, and backdrop blur on a page trigger the PDF stamp path (`k2f_paint` at scale 2/3/4, default 2).

**Bundled font:** starter ships `Roboto-Regular.ttf` only. Latin + common punctuation. **Not** a serif face, arrows (`→` `↗` `↑`), stars/dingbats (★ ◆), math (`∈`), or **CJK/kana**. Missing glyphs → `FONT_MISSING_GLYPH` (hard fail; no OS fallback). Embedded package fonts **do** fall back to each other.

- Academic / book serif: `init_package.py --add-font /path/to/Serif.ttf` (keeps Roboto) and point heading roles at that stem via `font_aliases`. Do not expect a bundled Libertine/DejaVu Serif.

- Japanese / Korean / full CJK: `init_package.py --add-font /path/to/NotoSansJP.otf` (keeps Roboto; kana uses the extra face). `--font` **replaces** Roboto and retargets theme roles — use a covering face that includes Latin. **NotoSansSC is Simplified Chinese**, not Japanese kana.
- Math symbols in body text also need a math/CJK face, or put them in `role: "math"` with NotoSansMath.
- Do **not** rewrite the user's language to English to dodge a missing glyph.

## Images

Embed **PNG, JPEG, WebP, or SVG** under `assets/images/` only (not `assets/` root, **not GIF**). SVG is rasterized **without system fonts**: `<text>` / `<tspan>` / `<textPath>` / `<foreignObject>` **fail at pack** (no silent drop) — convert labels to `<path>` (or a K2F text node beside the image). XML comments and CDATA that merely mention those tags are ignored. Declare width/height in millipt on the image node.

Default stack `align_items` is `stretch`: the image **box** fills the cross axis and paint letterboxes (contain, centered) inside it. Left/right: set the image role's `self_align` to `start`/`end` in `theme.json` (never on the node). See `catalog/content/ex_image.json`.

## Workflow

From this skill directory. Default font is `starter/assets/fonts/Roboto-Regular.ttf`.

Always use `scripts/pack_verify.py`. It uses `k2f` on PATH or `K2F_CLI`. A stale binary may reject documented keys — `pip install -U k2f`; do not strip valid JSON. The author directory can live outside this skill folder; pass its path. The script does **not** search a git checkout or cargo build directory.

```bash
python scripts/init_package.py --workspace <workspace> --title "..." --page a4|letter|a4-landscape|widescreen|widescreen-43
# or --dir <source> for an existing author path
# slides: --page widescreen --margin 0
# posters: --page a4 --margin 0
# custom size: --width 1984252 --height 2834646  (with --page for margin defaults)
# optional: --margin 36000   or   --margin 56000,56000,56000,56000
# edit source/content/root.json and source/styles/theme.json (copy from catalog/content/ex_*.json)
python scripts/pack_verify.py <workspace>/source -o <workspace>/<name>.K2F
# preview + single-page gate (compile stderr includes pages=N)
# bare --render preview.png → <workspace>/tmp/preview.png
python scripts/pack_verify.py <workspace>/source -o <workspace>/<name>.K2F --expect-pages 1 --render preview.png
```

`init_package.py --workspace` copies [`starter/`](../../starter/) into `PATH/source` and creates `PATH/tmp`. Empty dirs and notes-only dirs (a lone `design.md` in `source/`) are OK; an existing package is not. `--font` replaces Roboto and rewrites role `font_family`. `--add-font` copies extra faces beside Roboto for glyph fallback (CJK, math). Both copy bytes into `assets/fonts/` — pass a **readable** `.ttf`/`.otf` you can embed. Do not `--font` locked OS dirs (`/System/Library/Fonts`); copy the face to a writable path first. Copy failure prints the error and rolls back starter files only (it does **not** skip the font). Render never uses system fonts.

## Capability limits

No arbitrary vectors in State A, no native geometry/divider nodes (use decorated boxes, `role: "rule"`, or SVG), no node-level margin/bleed, no Flexbox `space-between`, no `z-index`/absolute coordinates, no slide/infinite `canvas_mode` (v0.1 is `paged` only — use the 16:9 / poster recipes above), no page/node animations or transitions, no native chart/graph nodes (use `table` or PNG/WebP/SVG), no radial gradients, no noise/grain/vignette/pattern fills, no text-shadow or glyph blur (box `shadow` / box `blur` only), no `font_variant`/small-caps/drop-cap, no per-corner radii or organic blob primitives, no hyphenation dictionary or auto-shrink type, no rotation/`writing_mode`, no inline text background/border patches (inline code = sibling `role: "code"` node, not a modifier pill), no ruby/chord layout, no grid first-line baseline align, no CSS-style `fr` without a finite outer size. Shapes are decorated boxes or image/SVG assets. If a design needs missing engine features, record gaps; **do not** change engine code or schemas to force a match.
