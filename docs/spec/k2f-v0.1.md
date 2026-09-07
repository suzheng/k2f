# K2F format specification v0.1

**Version:** `0.1` (tracks `k2f_core` crate `0.1.0` at release)

K2F (Key-to-Flow) is a ZIP archive containing a semantic document tree, theme, embedded fonts, optional assets, and a compiled geometry lock. Agents and tools write **meaning**; the reference engine owns layout and pixels.

## Contents

- [Container](#container)
- [State A — semantic tree](#state-a--semantic-tree)
- [Theme](#theme-stylesthemejson)
- [State B — reference engine](#state-b--reference-engine)
- [State C — documentk2flock](#state-c--documentk2flock)
- [Integrity verification](#integrity-verification)
- [Viewer banners](#viewer-banners)
- [Signing](#signing)
- [Prohibitions](#prohibitions)
- [Embedded schemas](#embedded-schemas)

## Container

A `.K2F` file is a ZIP archive. Canonical paths (from `engine/k2f_package/src/paths.rs`):

| Path | Required | Purpose |
|------|----------|---------|
| `manifest.json` | yes | Package metadata |
| `content/root.json` | yes | State A semantic tree entry (may reference other content JSON via `include`) |
| `content/**/*.json` | when referenced | Semantic subtree fragments; only paths referenced from `root.json` (directly or transitively) are allowed |
| `styles/theme.json` | yes | Role-based appearance |
| `styles/tokens.json` | no | Design tokens |
| `changelog.json` | yes | Edit history (`{"entries":[]}` when empty) |
| `document.K2F.lock` | after compile | State C geometry + render plan |
| `signatures/v1.json` | no | Ed25519 signature record |
| `schema/*.json` | yes | Embedded JSON schemas |
| `assets/fonts/*` | yes | Embedded font binaries |
| `assets/images/*` | no | PNG, JPEG, WebP, or SVG (engine rasterizes SVG at paint time) |
| `assets/data/*` | no | Structured data (e.g. table rows) |

Unexpected paths are rejected (`UNEXPECTED_PATH`).

## State A — semantic tree

The semantic tree lives in `content/root.json`. At pack time, `{ "include": "content/...." }` stubs in container `children` are replaced by the referenced file's single `SemanticNode` (which may contain nested includes). The engine compiles one expanded tree; `content_hash` binds that tree, not individual files. Unreferenced files under `content/` are rejected (`UNEXPECTED_PATH`).

Nodes have:

- **Stable dotted ids** — pattern `^[A-Za-z0-9][A-Za-z0-9_]*(\.[A-Za-z0-9][A-Za-z0-9_]*)*$`
- **Roles** — strings defined by this package's `styles/theme.json` (`theme.roles`). Official SDK themes share a common set (`document`, `section`, `h1`–`h4`, `body`, `warning`, `card`, `table`, `table_header_cell`, `table_row_cell`, `list_item`, `code`, `code_block`, `quote`, `rule`, `math`, `running_header`, `running_footer`, `signature_block`). `card` is optional; SDK `Document` helpers do not emit it. Agent authoring uses a narrower dialect in `engine/k2f_sdk/profiles/agent_v0.schema.json` (not embedded in the package). Nodes have no `font_variant`; monospace is `role: "code"` / `role: "code_block"` plus `font_aliases`.
- **Modifiers** — closed `type` enum: `emphasis`, `link`, `underline`, `strikethrough`, `subscript`, `superscript`, `math`, `syntax_highlight`. `intent` is a free string (URL, TeX, highlight token). `range` is UTF-8 **byte** offsets `[start, end)` on character boundaries (not character indices). There are no built-in `font-size` / `font-family` / `color` modifier types. For `superscript` / `subscript`, the engine applies script sizing (7/10 of the surrounding run) and a deterministic baseline shift (raise/lower); theme patches may change color/bold on those types but not absolute `font_size`.
- **Content** — text, code blocks, display math (TeX subset), native tables (`inline` or `asset` data), table references, containers, or images; no inline geometry
- **Pagination hints** — optional `break_inside` (`auto` | `avoid`), `keep_with_next` (boolean), and `break_before` (`auto` | `page`). `auto` splits at line or child boundaries when the remainder of the page is too small; `avoid` refuses to split a node across pages; `keep_with_next` keeps this node with the following sibling when both fit; `break_before: page` starts the node on a new page.

`manifest.json` may include `running_blocks`: an array of `{ "position": "header" | "footer", "node": <SemanticNode> }` repeated on each page. Valid only with `canvas_mode: "paged"`.

A code block is `role: "code_block"` with `content: { "type": "code_block", "value": <string | string[]> }`. Role and content type must agree; `layout` is forbidden; modifiers are limited to `syntax_highlight`. See [semantic_code_blocks.md](../instructions/semantic_code_blocks.md).

Native tables may load rows from `data: { "type": "asset", "source": "assets/data/..." }` instead of inline cell trees.

## Theme (`styles/theme.json`)

`palette` is the only color table. Role and primitive colors are palette keys or `#RRGGBB` / `#RRGGBBAA`. There is no `primitives.colors`.

Theme `box_decoration` names primitives (`background`, `border`, `corner_radius`, `shadow`, `blur` are strings). `padding_pt` stays numeric. Unknown names fail compile (`UNKNOWN_PRIMITIVE: {kind} '{name}'`). Inline fill/shadow/blur objects in theme JSON fail schema.

Named maps under `primitives`: `surfaces`, `gradients`, `shadows`, `blurs`, `corners`, `borders`. Compile inlines them into lock `BoxDecoration`. Official recipes: `corners` none/small/medium/large/full; `borders` subtle/contrast (optional `edges` for single-side strokes and `style` solid/dashed/dotted); `elevation.1`–`3`; `blurs.background`; `card` variants `flat` / `raised` / `glass`. Default published trees must not apply shadow or blur unless PDF stamp export is intended.

Roles may set `letter_spacing_pt` (signed millipt), optional `first_line_indent_pt` (non-negative millipt; first wrapped line only), and `text_align` (`start` | `center` | `end` | `justify`). Justify expands U+0020 gaps on non-final wrapped lines only. Official `h1` is `-500`. Tracking is extra advance on every glyph except the last in a shaped run. Role `box_decoration.padding_pt` applies to text, code, math, containers, tables, and image nodes.

State A must not contain layout coordinates, arbitrary vector paths, inline CSS, or executable content.

Containers may use `layout.type = "columns"` (`count` 2..=4, optional `gap`) for continuous multi-column flow. Nodes inside that container may set `column_span: "all"` so a figure or table spans the full content width. Title/abstract stay full-width by living **outside** the columns container.

Grid layouts accept optional `rows` (omit → `{auto:true}` tracks `ceil(n_children / n_columns)`; declared rows do not grow), optional `width`/`height` (millipt fixed outer size, same as stack/overlay — needed for `fr` tracks when the parent axis is unbounded), optional `row_gap` and `column_gap` (millipt; when omitted or `null`, both axes use `gap`), and optional `cell_align` (omit or `null` defaults to stretch). Tracks are `{pt}`, `{fr}`, or `{auto:true}` (content-sized from measured cells; leftover goes to `fr`). `{auto:true}` is not CSS `auto-fit`. Tables accept the same optional `row_gap` / `column_gap` (omit or `null` falls back to table `gap`). Table `column_widths` remain `{pt}`/`{fr}` only; every row has exactly that many cells (no colspan/rowspan). Pack rejects asset files outside `assets/fonts/`, `assets/images/`, and `assets/data/` (`UNEXPECTED_PATH`). License/sidecar files under `assets/fonts/` may remain in the package; only `.ttf`/`.otf` are loaded as faces.

Display math is `role: "math"` with `content: { "type": "math", "value": "<tex>" }`. Inline math stays on a text node as U+FFFC plus modifier `{ "type": "math", "intent": "<tex>" }`. The engine compiles a TeX subset (`frac`, `sqrt`, scripts, sums/integrals, Greek/symbols including `\hbar` `\langle` `\mathbb` `\mathcal`, stretchy `\left\right`, `matrix`/`pmatrix`/`bmatrix`/`align`/`cases`) into glyphs plus solid fraction/radical/delimiter rules. Unknown commands fail compile (`MATH_UNSUPPORTED`).

## State B — reference engine

The reference layout engine (`k2f_layout`) compiles State A + theme + fonts (+ optional assets) into a lock. The lock records:

- `engine_version` — semver of the compiling engine
- `engine_commit_sha` — git commit of the engine build (release builds must not use `UNKNOWN`)

## State C — `document.K2F.lock`

The lock is immutable for viewers. It contains:

- **Geometry** — page tree with millipt positions and sizes
- **Render plan** — ordered paint operations (`DrawText`, `DrawImage`, fills, etc.)
- **Hashes** — `content_hash` (expanded semantic tree) and `appearance_hash` (semantic + theme + fonts + page config + engine identity)

Opening a `.K2F` file is **not** compiling. Viewers paint the lock only.

## Integrity verification

`k2f verify` and `inspect_package` return status codes:

| Code | Meaning |
|------|---------|
| `VALID` | Lock matches semantic content, theme, fonts, and engine |
| `UNSIGNED` | Valid hash chain, no signature file |
| `SIGNED` | Valid hash chain and valid Ed25519 signature |
| `SIGNED_BUT_BROKEN` | Signature present but hash or crypto check failed |
| `UNLOCKED` | No `document.K2F.lock` |
| `CONTENT_CHANGED` | Semantic tree changed since lock |
| `APPEARANCE_CHANGED` | Theme, fonts, or page config changed since lock |
| `ENGINE_MISMATCH` | `engine_version` or `engine_commit_sha` differs from reader |
| `FONT_MISSING` | Required embedded fonts absent |

Additional package errors: `SCHEMA_INVALID`, `NODE_ID`, `UNEXPECTED_PATH`, `UNKNOWN_PAINT_OP`, `PDF_IS_NOT_A_SOURCE`.

## Viewer banners

Viewers must surface integrity status derived from `IntegrityStatus`. The banner **code** is always available via `banner()` / `k2f-open`; default UI uses **tiered visibility** (like PDF viewers):

| Banner | When | Default UI |
|--------|------|------------|
| `SIGNED` | Valid signature and hash chain | Compact positive strip |
| `UNSIGNED` | Valid, unsigned | Hidden (quiet read) |
| `SIGNED_BUT_BROKEN` | Signature file invalid or tampered | Prominent warning |
| `BROKEN_INTEGRITY` | Content, appearance, engine, or font failure | Prominent warning |
| `UNLOCKED` | Package not compiled | Compact draft strip |

Embedders may pass `banner: "full"` for the legacy verbose strip on every state, or `banner: "off"` / `no-banner` to hide chrome entirely. Broken states must never be silently treated as signed.

`BROKEN_INTEGRITY` and `SIGNED_BUT_BROKEN` must **not** be treated as signed.

## Signing

Signing is a human or organizational step. Agents produce `UNSIGNED` packages by default. `signatures/v1.json` records Ed25519 public key, signature over canonical lock JSON, `signed_by`, and `signed_at`.

## Prohibitions

- **No scripts** — no JavaScript, WASM, or other executable content inside `.K2F`
- **No inline CSS** in semantic nodes
- **No PDF as source** — `PDF_IS_NOT_A_SOURCE`; PDF export is a raster/vector drawing of the lock
- **No arbitrary vector paths** in State A

## Embedded schemas

Each package embeds copies under `schema/` (format contract only):

- `manifest.schema.json`
- `nodes.schema.json`
- `styles.schema.json`
- `visual_primitives.schema.json`
- `signatures.schema.json`

Agent writing dialects live outside the format (`engine/k2f_sdk/profiles/`). Schema changes require a spec version bump and matching engine release.
