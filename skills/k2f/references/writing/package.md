# Authoring an unpacked K2F package

Domain conventions for fixed layouts (CV, flyer, cheatsheet). **Allowed JSON keys:** [`schema/`](../../schema/). **Golden shapes:** [`catalog/content/ex_*.json`](../../catalog/content/) ([index](../../catalog/README.md)). Copy those files; do not invent syntax.

**Do not use in this skill's writing workflow** (schema-legal but unsupported here): `content.type: "table_reference"`, table `data.type: "asset"`, `content.type: "code_block"` (use `role: "code"` + text — see `ex_code.json` and [converting-markdown/mapping.md](../converting-markdown/mapping.md)).

## Required loose files

| Path | Notes |
|------|-------|
| `manifest.json` | `canvas_mode: "paged"`, `page_config` (width/height/margin in millipt) |
| `content/root.json` | Semantic tree entry; may reference fragments via `{ "include": "content/...." }` |
| `content/**/*.json` | Optional; one node per file, referenced from root (see below) |
| `styles/theme.json` | Roles, palette, named primitives (do not split across files) |
| `assets/fonts/*` | At least one font (starter ships Roboto-Regular) |
| `assets/images/*` | Optional PNG/WebP/SVG (not `assets/` root; not JPEG) |
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
5. Verify with `python scripts/pack_verify.py <dir> -o out.K2F --expect-pages 1 --render preview.png`. `compile`/`verify` print `pages=N`. `LAYOUT_SLACK` is a warning only (exit 0) — open the PNG; leave the gap if it is intentional. `preview.png` is **only page 0**.

## Layout

Allowed layout types and fields: `schema/nodes.schema.json` → `layout`. Copy from catalog rather than inventing fields.

| Need | Use |
|------|-----|
| Continuous article columns | `columns` — `ex_columns.json` |
| Fixed side-by-side (sidebar, header bar) | horizontal `stack` or 2-col `grid` |
| Poster / slide page shell | `ex_poster_shell.json` — pinned height + `{auto:true}` header/footer + `{fr:1}` grower |
| Poster / dashboard cells | `grid` with `pt` tracks, or `fr` **after** a finite outer height — `ex_grid.json` |
| Full-page background + content | `overlay` (background child first) — `ex_overlay.json` |

- Grid tracks: `{ "pt": N }`, `{ "fr": N }`, or `{ "auto": true }` — never bare integers. `{auto:true}` is layout-grid only (not table `column_widths`).
- Do **not** put `grid` directly on the root node (awkward cross-page behavior). Nest grid under a child container, e.g. `root` → `root.grid`.
- **`fr` rows need a finite outer height.** In a vertical flow with unbounded height, `rows: [{fr:1}]` fails with `Cannot resolve fr tracks with infinite available size`. `fr` divides a **known** outer size — it is **not** content-auto height (`{auto:true}` is). Fix: set the grid's own `height`, use `pt`/`auto` rows, nest under a fixed-height stack/overlay, or prefer `stack` / native `table`.
- **Pin content to column bottom** (footnotes / correspondence): fixed-height 2-row grid — `height` + `rows: [{"fr": 1}, {"pt": N}]` with body in row 0 and footer text in row 1. No footnote node and no `space-between`.
- Titles for multi-column flow: keep outside a `columns` container; use `column_span: "all"` for full-width figures inside.
- **Overlay is in-flow stacking**, not absolute positioning: children share one origin; height = max(children); later children paint on top (no `z-index`). The block is unsplittable across pages. Optional `width`/`height` pin the box; full-page backgrounds use content height (page height minus margins). Overlay has **no** `align_items` / `justify_content` — to center a layer, nest a **fixed-height** stack inside that child with `justify_content: "center"` and `align_items: "center"`.
- **No `justify_content: space-between`.** For left/right split use a two-column grid (`columns: [{fr:1},{fr:1}]` + `cell_align`). Cover vertical centering: fixed-height stack + `justify_content: center`.
- **No empty spacer containers.** Do not insert a child whose only job is `layout.height` to invent gaps — that is geometry, not semantics. Uneven rhythm: nest stacks with different `gap`, or put `padding_pt` on section roles. `role: "rule"` with a small height is a semantic divider, not a spacer.
- **Box model (`layout.height` + `padding_pt`):** `layout.height` / `width` is a **minimum outer** size. Children are laid out in the **inner** box (`outer − padding`). Measured outer is `max(content+padding, hint)`. If an inner child is also set to full page height, outer grows to `content + padding` and with `break_inside: "avoid"` → `UNSPLITTABLE_OVERFLOW`. Recipe: outer shell = page size; padding on that role (or an inner wrapper); children size to the **remaining** inner area — do not re-declare full page height on padded descendants. **Root** `padding_pt` inflates page margins (see slide recipe).
- **Paged flow splits at sibling boundaries.** A node's measured height **includes** role `padding_pt` (nested padding stacks). If content + padding exceeds remaining page space, that sibling moves to the next page — shrink padding/gap or split into siblings. Multiple top-level children under `root` become separate pages when they overflow. Single-page posters: one child from `ex_poster_shell.json`.
- **`manifest.running_blocks`:** repeating header/footer nodes (`position: "header"|"footer"`). Each `node` is a normal semantic node — `stack` / `grid` / text all work. Horizontal alignment: role `self_align` in theme (`start`/`center`/`end`). Placeholders `{{page_current}}` / `{{page_total}}` only. Overflow past the margin band fails compile — keep running blocks short. Example: `catalog/manifest.json`.

## Theme

**Theme-only styling.** Style fields live in `styles/theme.json` roles/variants only — never on nodes in `root.json`. Nodes may set `role`, `variant`, `layout`, `modifiers`, `break_inside`, `keep_with_next`, `column_span`. Allowed role/theme keys: `schema/styles.schema.json` + `schema/visual_primitives.schema.json`.

**Looks (optional, posters / slides / styled reports):** bundled **examples** under [`looks/`](../../looks/) — a complete `theme.json` plus a composition guide. Use **only when the user did not specify a style**. If they named a design, ignore bundled looks and author `styles/theme.json` to match. When you do use a look: read **one** `look.md`, then **rewrite** the package theme from that example (adapt tokens and roles; do not ship the file unchanged). Shared extension roles: `display`, `kicker`, `caption`, `metric`, `shell` — see [`looks/README.md`](../../looks/README.md). Starter does not define those roles.

- Colors only via `palette` (keys or `#RRGGBB` / `#RRGGBBAA`).
- `box_decoration` fields are **named primitive strings** only → unknown name = `UNKNOWN_PRIMITIVE`.
- `padding_pt` counts toward measured outer height; with a fixed `layout.height`, children see the inner box (outer − padding). See box model above.
- **Paragraphs:** one text node per paragraph. `\n` inside a node is a hard line break only — not paragraph spacing. Use sibling `gap` between text nodes for paragraph spacing.
- **Multi-line titles:** one heading node with `\n`. Do not split one title into several heading nodes. Modifier `range` is UTF-8 **bytes** on the full `value` (`\n` is 1 byte) — always run `python scripts/modifier_range.py --text "<exact value>" --find "…"`; do not hand-count character indices.
- **Display math:** `role` must be `"math"` with `content.type = "math"`. Custom roles cannot wrap math content; use `variant` for visual skins. TeX subset: see [errors.md](errors.md). Catalog includes NotoSansMath; starter does not — add the font if you copy `ex_math.json`.
- **Inline math in author JSON:** U+FFFC in the text value plus a node modifier `{ "type": "math", "intent": "<tex>", "range": [byte_start, byte_end] }`. `$...$` in JSON is **not** parsed — that syntax is Markdown only. See `ex_modifiers.json`. Modifier types and `range` rules: `schema/nodes.schema.json`.
- **Modifiers:** visual patches come from `theme.modifiers.styles[type][intent]`. Run `modifier_range.py` for byte offsets (max 50). Example node + theme patch: `ex_modifiers.json`.
- **Full-bleed bars:** set `page_config.margin` to `[0,0,0,0]`, keep root padding 0, then wrap content in an inner container with padding. There is no separate bleed primitive.
- **Dividers:** `role: "rule"` on a container with a small `layout.height` and a surface fill, or a content box whose border uses `edges: ["bottom"]` / dashed style.
- **Fills / glass:** `primitives.gradients` support **linear** fills only (no radial). Named `box_decoration.blur` + `primitives.blurs` give backdrop blur / glass on boxes. Radial glow or soft vignettes → SVG under `assets/images/`. `box_decoration.shadow` is **box** elevation only — there is no text-shadow on glyphs.
- **Native table (inline):** each cell is a full semantic node. Minimal shape: `catalog/content/ex_table.json`.
- **Composite table cells:** a cell may be a `container` + `stack` holding text / `list_item` children. `list_item` must be **text** content with a non-empty `list_id` (not a nested list container).
- **`font_aliases`:** maps role `font_family` names to embedded font **keys** (filename stems under `assets/fonts/`). One font file also registers as `"default"`. **Two or more fonts:** no auto-`default` — set `font_aliases` and role `font_family` to each stem. CSS generic families fail compile.
- Format `role` is an open string but **must exist** in `theme.roles`. Custom roles are expected.

Default published trees should avoid shadow/blur unless PDF export with stamp is intended. Linear gradients, shadows, and backdrop blur on a page trigger the PDF stamp path (`k2f_paint` at scale 2/3/4, default 2).

**Bundled font:** starter ships `Roboto-Regular.ttf` only. Latin + common punctuation. **Not** arrows (`→` `↗` `↑`), stars/dingbats (★ ◆), or **CJK**. Missing glyphs → `FONT_MISSING_GLYPH` (hard fail; no OS fallback). Fix: ASCII (`->`), SVG icon, rewrite CJK to English, or `init_package.py --font` / add a covering TTF under `assets/fonts/`.

## Images

Embed **PNG, WebP, or SVG** under `assets/images/` only (not `assets/` root, **not JPEG/GIF**). SVG is rasterized at paint time **without system fonts**: any SVG `<text>` is dropped or invisible — convert labels to `<path>` (or use a K2F text node beside the image). Declare width/height in millipt on the image node.

Default stack `align_items` is `stretch`: the image **box** fills the cross axis and paint letterboxes (contain, centered) inside it. Left/right: set the image role's `self_align` to `start`/`end` in `theme.json` (never on the node). See `catalog/content/ex_image.json`.

## Workflow

From this skill directory. Default font is `starter/assets/fonts/Roboto-Regular.ttf`.

Always use `scripts/pack_verify.py`. It uses `k2f` on PATH or `K2F_CLI`. A stale binary may reject documented keys — `pip install -U k2f`; do not strip valid JSON.

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

`init_package.py` copies [`starter/`](../../starter/). Pass `--font` only to override the bundled face.

## Capability limits

No arbitrary vectors in State A, no native geometry/divider nodes (use decorated boxes, `role: "rule"`, or SVG), no node-level margin/bleed, no Flexbox `space-between`, no `z-index`/absolute coordinates, no slide/infinite `canvas_mode` (v0.1 is `paged` only — use the 16:9 / poster recipes above), no page/node animations or transitions, no native chart/graph nodes (use `table` or PNG/WebP/SVG), no radial gradients, no noise/grain/vignette/pattern fills, no text-shadow or glyph blur (box `shadow` / box `blur` only), no `font_variant`/small-caps, no rotation/`writing_mode`, no inline text background/border patches, no ruby/chord layout. Shapes are decorated boxes or image/SVG assets. If a design needs missing engine features, record gaps; **do not** change engine code or schemas to force a match.
