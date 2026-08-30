# Writing K2F Documents

Deliverable is an **UNSIGNED** `.K2F` — signing stays a human step.

**Default path:** edit the semantic tree as JSON in an unpacked author directory, then `pack_verify.py`. Treat `content/*.json` and `styles/theme.json` like source code; `k2f pack` / `compile` / `verify` are the compiler. Do **not** hand-edit `document.K2F.lock`.

## When to use

| Situation | Path |
|-----------|------|
| New CV, flyer, cheatsheet, slide deck, poster, custom layout | Start from blank (below) |
| Existing `.K2F` to patch | `k2f unpack` → edit JSON → `pack_verify.py` |
| Already have an unpacked author directory | Edit JSON → `pack_verify.py` |
| Source is Markdown | [converting-markdown.md](converting-markdown.md), then patch JSON here if needed |
| PDF-only request | Write `.K2F` first, then [exporting-pdf.md](exporting-pdf.md) |

**When NOT to use:** PDF as source (`PDF_IS_NOT_A_SOURCE`); pixel/CSS layout edits; per-node `x`/`y` or drag-box layout (change theme + full relock); inventing JSON without catalog + schema lookup.

## Writing loop

### Prerequisites

```bash
pip install k2f    # unpack, pack, compile, verify, render, schema dump — on PATH
```

### Steps

1. **Get a working directory**
   - **Blank:** `python scripts/init_package.py --dir ./out/doc --title "…" --page a4` — copies [`starter/`](../starter/) (empty tree + core theme + Roboto).
   - **Packaged `.K2F`:** `k2f unpack contract.K2F -o ./out/contract` — do **not** use `--include-lock` (author dirs must not contain lock or embedded `schema/`).
   - **Already unpacked:** use the existing directory.
2. Edit `content/root.json` (+ optional `content/*.json` includes) and `styles/theme.json`. **Minimal diff** — change only the nodes you mean to change.
3. **Copy shapes** from [`catalog/content/ex_*.json`](../catalog/content/) — one file per construct (stack, grid, table, …). See [`catalog/README.md`](../catalog/README.md). Do not ship the catalog as your document.
4. **Design (optional looks):** if the user **already named a style or design**, follow that. If they did **not**, pick the **one** look under [`looks/`](../looks/README.md) that fits this content (if not sure which to use, you can use `quiet-light`), read that guide, and **rewrite** `styles/theme.json` from the example. You can that look's composition recipes.
5. **Unsure about a key?** Open the matching file under [`schema/`](../schema/) before writing JSON.
6. `python scripts/pack_verify.py ./out/doc -o ./out/doc.K2F --render preview.png`
7. Expect `verify` → **`UNSIGNED`**. Then **open the PNG** — [visual check](#visual-check). If you used a look, also check that look's **Don'ts**.

Page presets: `a4` | `letter` | `a4-landscape` | `widescreen` | `widescreen-43`. Slides/posters: see [writing/package.md](writing/package.md).

### Examples

**New report**

```bash
python scripts/init_package.py --dir ./out/report --title "Q3 Report" --page a4
# slides: --page widescreen --margin 0 | widescreen-43
# posters: --page a4 --margin 0; copy catalog/content/ex_poster_shell.json; set height to page
# --margin 36000  or  --margin 36000,48000,36000,48000
# edit content/root.json — copy nodes from catalog/content/ex_*.json
# edit styles/theme.json — if the user gave no design, rewrite from looks/<name>/ (see looks/README.md)
python scripts/pack_verify.py ./out/report -o ./out/report.K2F --render preview.png
```

**Patch an existing package**

```bash
k2f unpack contract.K2F -o ./out/contract
# grep or read content/*.json for stable ids — never guess
# edit the text node (or cell id for tables); roles from this package's theme.json
python scripts/pack_verify.py ./out/contract -o ./out/contract-edited.K2F --render preview.png
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
| Custom font | `init_package.py --font /path/to/Covering.ttf` or add under `assets/fonts/` + `font_aliases` |
| Modifier byte ranges | `python scripts/modifier_range.py --text "…" --find "…"` |
| Allowed JSON keys | [`schema/`](../schema/) |

## Visual check

`verify` is not a layout review. After every pack, render the lock to PNG and **open the image**. Do not ship a file you have not looked at.

```bash
python scripts/pack_verify.py ./out/doc -o ./out/doc.K2F --render preview.png
# compile prints pages=N; --render is page 0 only:
k2f render ./out/doc.K2F --page 1 -o preview-1.png
```

Inspect every page. In particular:

- **Poster/slide empty bottom** — copy [`catalog/content/ex_poster_shell.json`](../catalog/content/ex_poster_shell.json) (`pt` header/footer + `{fr:1}` body). Do not use a vertical stack as the page shell; do not invent spacer nodes.
- **Report empty bands** — shrink `page_config.margin`, role `padding_pt`, or stack `gap`.
- **Type size** — body text too large or headings too small for the canvas. Change `font_size` on the **role** in `styles/theme.json`, never on the node.

If the PNG looks wrong, edit JSON or theme and run `pack_verify.py --render` again.

## Rules (non-negotiable)

1. **Stable dotted ids** on every line, clause, party, and total you may edit later. **Never guess ids** — read the tree or grep `content/`.
2. **Roles must exist** in **this package's** `styles/theme.json` (legal/contract packs often use `critical_warning`, not the SDK catalog name `warning`).
3. **Theme-only styling** — no inline colors/font sizes on nodes.
4. Image width only via declared millipt on image nodes; files under `assets/images/` (PNG/WebP/SVG, not JPEG). To swap an image, replace bytes under `assets/images/` and update the node path — there is no in-place byte swap API.
5. **Tables:** edit **cell** text node ids, never the table root id.
6. **Text with modifiers:** after changing `content.value`, recompute modifier `range` with `modifier_range.py` — stale byte ranges fail compile or render wrong.
7. Do not use system `unzip` on the ZIP and edit in place — use `k2f unpack` so lock/schema are omitted and includes stay on disk.
8. Do not hold org signing keys.
9. **Open the rendered PNG** after every pack. `UNSIGNED` is not a visual pass.

## Failure protocol

| Situation | Action |
|-----------|--------|
| Missing `k2f` CLI | `pip install k2f` (or set `K2F_CLI`) |
| validate / compile fails | [writing/errors.md](writing/errors.md); never edit lock |
| `SCHEMA_INVALID` | Open [`schema/`](../schema/); key in schema but rejected → `pip install -U k2f`; key not in schema → remove |
| `UNKNOWN_ID` | Read `content/` or grep for the id; never invent ids |
| `UNKNOWN_ROLE` | Use a role/variant from **this package's** theme |
| `WRONG_CONTENT` | Wrong node kind for a text edit — use cell ids for tables; see [writing/errors.md](writing/errors.md) |
| Poster spilled to page 2 | `compile` prints `pages=N`; use `--expect-pages 1` |
| Viewer `BROKEN_INTEGRITY` | Content changed without relock — run `pack_verify.py` |
| Want Word-like layout | Theme + full relock, not per-node x/y |
| Used `Editor.insert_node` on author dir | Wrong API — edit JSON files, then pack |

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
| SVG or images in `assets/` root | Move to `assets/images/` (PNG/WebP/SVG, **not JPEG**); root files → `UNEXPECTED_PATH` |
| JPEG / `.jpg` image | Unsupported — convert to PNG/WebP, or embed SVG |
| `self_align` (or any theme field) on a node | Put `self_align`, `text_align`, fonts, padding in `styles/theme.json` roles only |
| `$...$` inline math in author JSON | U+FFFC + `{ "type": "math", "intent": "<tex>" }` modifier — see `catalog/content/ex_modifiers.json`; `$` works Markdown only |
| Multiple fonts but `"default":"default"` only | Map `font_aliases` to each file stem; two+ fonts have no auto-`default` |
| `fr` rows without fixed grid height | Fails: `Cannot resolve fr tracks with infinite available size`. `fr` ≠ content-auto height — set grid `layout.height`, use `pt` rows, or nest under a fixed-height stack (`ex_grid.json`) |
| Poster/slide shell is a vertical stack | Content piles at the top. Copy `ex_poster_shell.json`: pinned `height` + `{fr:1}` body row |
| Expect a native `Divider` node | Use `role: "rule"` + small `layout.height` + surface fill (or bottom border) |
| Noise / vignette / radial glow / dot matrix | Not in core — SVG under `assets/images/` (labels as `<path>`); size in millipt |
| Require `row_gap`/`column_gap`/`cell_align` | Optional — see `schema/nodes.schema.json`; omit unused keys (`null` ok) |
| Unicode superscript (`²`) for notes | Ordinary char + `superscript`/`subscript` modifier; formulas → math |
| SVG `<text>` labels | Convert to `<path>` — paint has no system fonts; `<text>` is dropped |
| One text node with `\n\n` for paragraphs | One paragraph = one text node; use stack `gap` |
| Simulate margin with padding / empty spacer stacks | No node margin — use parent `gap` + role padding; empty `height`-only containers are geometry |
| Dingbat/arrow/CJK glyphs (★ ◆ → ↗ ↑ 中文) in Roboto | `FONT_MISSING_GLYPH` — ASCII/`->`, SVG icon, or `init_package.py --font` covering TTF; bundled face is Latin-only |
| Expect `canvas_mode: "slide"` | v0.1 is `paged` only — use `--page widescreen --margin 0` + per-slide fixed height + `break_inside: avoid` |
| `layout.height` + padding overflowing the page | Height is min outer; padding is inside. Do not nest another full-page-height child inside a padded shell |
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

- [writing/package.md](writing/package.md) — slides, posters, fonts, capability limits
- [writing/errors.md](writing/errors.md) — error codes
- [converting-markdown.md](converting-markdown.md) — Markdown source
- [exporting-pdf.md](exporting-pdf.md) — PDF after pack
- [publishing.md](publishing.md) — permanent `/v/{appearance_hash}`
- [embedding-viewer.md](embedding-viewer.md) — human click → node id
