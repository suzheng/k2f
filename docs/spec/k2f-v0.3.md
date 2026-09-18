# K2F format specification v0.3

**Version:** `0.3` (on-disk package contract)

K2F (Key-to-Flow) is a ZIP document format. Authors write **meaning** as JSON. One reference engine compiles that meaning into geometry. Every viewer and every export paints the compiled lock, so output matches.

This page is the readable contract: container layout, processing, integrity, and rules JSON Schema cannot express. Exact fields live in the format JSON Schemas (`schema/*.json` in the repo). How to write a package: [Authoring](../authoring/text.md) and [Allowed keys](../../skills/k2f/references/writing/fields.md). Engine matching: [COMPATIBILITY.md](../../COMPATIBILITY.md).

This spec line is released with **K2F SDK 0.3.x** (Rust engine, Python `k2f`, npm `@openk2f/k2f`). SDK patch releases keep this document unless embedded schemas or the lock format change. Each compiled lock still records **`engine_version`** (full compiler semver) for `appearance_hash` — that is provenance, not a second public spec number.

A breaking change to schemas or the lock format bumps this spec version and ships in the same SDK release.

This spec describes the **packed** `.K2F` ZIP. An author directory may omit `schema/` (pack injects the five format schemas) and `changelog.json` (pack writes `{"entries":[]}`).

## Model

```
State A   content/root.json      semantic tree — you edit this
   ↓      pack / compile
State C   document.K2F.lock      geometry + paint — viewers paint this
```

Opening a `.K2F` file is **not** compiling. The engine is not a third file in the package.

Lengths are **millipt** integers (1 pt = 1000): page size, margins, font size, padding, gaps, and image size.

v0.3 is paged only (`canvas_mode` is always `"paged"`). Slide and infinite-canvas modes are not in this spec.

## Contents

- [Model](#model)
- [Container](#container)
- [Manifest](#manifest)
- [Semantic tree](#semantic-tree)
- [Theme](#theme)
- [Compile and lock](#compile-and-lock)
- [Integrity verification](#integrity-verification)
- [Viewer banners](#viewer-banners)
- [Signing](#signing)
- [Prohibitions](#prohibitions)
- [Embedded schemas](#embedded-schemas)
- [Related documents](#related-documents)

## Container

A `.K2F` file is a ZIP archive. Unexpected paths are rejected (`UNEXPECTED_PATH`).

| Path | Required | Purpose |
|------|----------|---------|
| `manifest.json` | yes | Title, page size, margins, engine version |
| `content/root.json` | yes | Semantic tree entry (may `include` other content JSON) |
| `content/**/*.json` | when referenced | One semantic node per file; only paths referenced from `root.json` (directly or transitively) are allowed |
| `styles/theme.json` | yes | Role-based appearance |
| `styles/tokens.json` | no | Optional extra JSON stored with the package; not a format schema and not required to compile |
| `changelog.json` | yes | Edit history (`{"entries":[]}` when empty) |
| `document.K2F.lock` | after compile | Geometry and render plan |
| `signatures/v1.json` | no | Ed25519 signature record |
| `schema/*.json` | yes | The five embedded format schemas |
| `assets/fonts/*` | yes | Embedded font binaries (at least one `.ttf` / `.otf`) |
| `assets/images/*` | no | PNG, JPEG, WebP, or SVG |
| `assets/data/*` | no | Structured data (for example table rows) |

License or sidecar files may sit under `assets/fonts/`; only `.ttf` / `.otf` load as faces. Pack rejects files outside `assets/fonts/`, `assets/images/`, and `assets/data/` (`UNEXPECTED_PATH`).

## Manifest

Required: `title`, `canvas_mode: "paged"`, `page_config` (`width`, `height`, `margin` as `[top, right, bottom, left]` millipt), `engine_version`. Optional: `author`, `created_at` (UTC unix seconds; pack must not stamp wall-clock time), `generated_by`, `running_blocks`.

There is no `page_config.background`. Sheet fill is the **root node’s** role `box_decoration.background` (full page, including margins).

`running_blocks` is an array of `{ "position": "header" | "footer", "node": <SemanticNode> }`. Blocks repeat on every page. They are valid only with `canvas_mode: "paged"`. Form fields are forbidden inside them. Text may use `{{page_current}}` and `{{page_total}}` only.

Contract: [`manifest.schema.json`](../../schema/manifest.schema.json).

## Semantic tree

The tree lives in `content/root.json`. At pack time, `{ "include": "content/...." }` stubs in container `children` are replaced by the referenced file’s single semantic node (which may contain nested includes). The engine compiles one expanded tree; `content_hash` binds that tree, not individual files. Unreferenced files under `content/` are rejected (`UNEXPECTED_PATH`).

State A must not contain layout coordinates, arbitrary vector paths, inline CSS, or executable content. Style fields (`color`, `font_size`, `font_family`, `padding`, `text_align`, `box_decoration`, …) belong on the theme role, never on the node.

Contract: [`nodes.schema.json`](../../schema/nodes.schema.json). Human index: [Allowed keys](../../skills/k2f/references/writing/fields.md).

### Nodes

Every node has:

- **`id`** — stable dotted id, pattern `^[A-Za-z0-9][A-Za-z0-9_]*(\.[A-Za-z0-9][A-Za-z0-9_]*)*$`, unique in the document
- **`role`** — a name defined under this package’s `theme.roles`. There is no format-wide role enum. Nodes have no `font_variant`; monospace is a role plus `font_aliases`
- **`content`** — see [Content types](#content-types)
- **`variant`** — optional; must exist on that role in the theme
- **`modifiers`** — optional; see [Modifiers](#modifiers)
- **`layout`** — optional; containers only. See [Layout](#layout)

`list_item` also requires `list_id`; `depth` defaults to `0` and `marker_type` defaults to `bullet`.

### Content types

| `content.type` | Meaning |
|----------------|---------|
| `text` | String `value`. Headings, body, quotes, and inline code differ by `role` |
| `math` | Display TeX subset in `value`. Role must be `math` |
| `image` | `value.src` under `assets/images/`, plus millipt `width` and `height`. PNG, JPEG, WebP, or SVG (rasterized at paint). Not GIF. SVG `<text>` / `<tspan>` / `<textPath>` / `<foreignObject>` fail (`SVG_TEXT`) |
| `container` | Optional `children` (nodes or `{ "include": "content/...." }` stubs) |
| `table` | Native table: `column_widths` (`{pt}` / `{fr}` only) and `data` (`inline` rows of cell nodes, or `asset` under `assets/data/`). Optional `header_rows`, `gap`, `row_gap`, `column_gap`. Cell `colspan` (default 1) occupies that many tracks in the same row. No `rowspan` |
| `form_field` | Reserved fillable box — [Form fields](#form-fields) |
| `code_block` | Listing; role must be `code_block`, `layout` is forbidden, modifiers are `syntax_highlight` only. `preserve_whitespace` must be `true` or omitted. Inline code is `role: "code"` with `content.type: "text"` |
| `table_reference` | External table view with declared millipt size |

Display math is `role: "math"` with `content.type: "math"`. Inline math stays on a text node as U+FFFC plus a `math` modifier whose `intent` is the TeX. Unknown commands fail compile (`MATH_UNSUPPORTED`).

### Modifiers

On a text node, `modifiers` is a closed list (max 50). Each item needs `type`, `intent`, and `range`.

- **`type`:** `emphasis`, `link`, `underline`, `strikethrough`, `subscript`, `superscript`, `math`, `syntax_highlight`. There are no `font-size` / `font-family` / `color` modifier types
- **`intent`:** a string that must exist under `theme.modifiers.styles[type]` (URL for `link`, TeX for `math`, highlight token, …)
- **`range`:** UTF-8 **byte** offsets `[start, end)` on character boundaries — not character indices. `\n` is 1 byte

For `superscript` / `subscript`, the engine applies script sizing (7/10 of the surrounding run) and a deterministic baseline shift. Theme patches may change color or bold on those types, not absolute `font_size`.

### Layout

`layout` applies to containers. Omit it, or use a vertical `stack`, for ordinary flow. `content/root.json` must omit `layout` or use a **vertical** `stack` only — nest `grid`, `overlay`, `columns`, and horizontal stacks under a child.

| `layout.type` | Meaning |
|---------------|------|
| `stack` (default) | `direction` `vertical` or `horizontal`; `gap`; `align_items`; `justify_content` (`start` / `center` / `end` only — not CSS `space-between`); optional millipt `width` / `height` |
| `grid` | Required `columns`; optional `rows`, `gap`, `row_gap`, `column_gap`, `cell_align`, `width` / `height`. Tracks are `{pt}`, `{fr}`, or `{auto:true}` — never bare integers. Omit `rows` → `{auto:true}` tracks `ceil(n_children / n_columns)`. Declared rows do not grow. `{auto:true}` is content-sized, not CSS `auto-fit`. `fr` needs a finite outer size on that axis |
| `overlay` | Children share one origin; later children paint on top. Height is the max of children. No `z-index` |
| `columns` | Continuous multi-column flow: `count` 2..=4, optional `gap`. A child may set `column_span: "all"` |

Walkthroughs: [Layout](../authoring/layout.md).

### Pagination

Optional on any node:

| Field | Values | Meaning |
|-------|--------|---------|
| `break_inside` | `auto` (default), `avoid` | `auto` splits at line or child boundaries when the remainder of the page is too small. `avoid` never splits |
| `keep_with_next` | boolean | Keep this node with the following sibling when both fit |
| `break_before` | `auto` (default), `page` | `page` starts the node on a new page |

### Includes

An include stub is `{ "include": "content/...." }` — not a semantic node. The path must match `content/<segments>.json`, must not be `content/root.json`, and must point at a file that is a single node. Orphan JSON under `content/` fails pack.

### Form fields

A form field is a leaf like an image: the engine reserves a box from declared `width` / `height` / `lines` and role metrics. Filling must not reflow following nodes. Role must be `form_field` and `content.type` must be `form_field`. `layout` and `modifiers` are forbidden. Set `break_inside: "avoid"` so the box does not split across pages. `placeholder` is metadata and is not painted into the lock. `value` is NFC text and does not drive measure.

| JSON field | Meaning |
|------------|---------|
| `kind` | `text` \| `multiline` \| `checkbox` (required) |
| `value` | NFC string. Checkbox allows only `""` or `"true"` |
| `placeholder` | Viewer-only hint; not in lock ops |
| `width` / `height` | Optional millipt `> 0`. Omit width in a bounded vertical stack or `{fr:1}` cell; a horizontal stack child must set `width` or compile fails `FORM_FIELD_UNBOUNDED_WIDTH` |
| `lines` | Optional `>= 1`. Defaults: text 1, multiline 3. Checkbox ignores `lines` (square of role `font_size`) |
| `max_length` | Optional `>= 1` character cap on `value` |
| `required` | Optional; empty required fields still compile (blank templates must lock) |

Viewers overlay native controls on the reserved rectangle. Saving writes `value` and relocks. PDF export may emit AcroForm widgets; that is export, not a package script.

## Theme

Appearance lives only in `styles/theme.json`. `palette` is the only color table. Role and primitive colors are palette keys or `#RRGGBB` / `#RRGGBBAA`. There is no `primitives.colors`.

| Section | Purpose |
|---------|---------|
| `palette` | Named colors |
| `primitives` | Named `surfaces`, `gradients`, `shadows`, `blurs`, `corners`, `borders` |
| `roles` | Typography and decoration keyed by role name |
| `font_aliases` | Role `font_family` → embedded font stem under `assets/fonts/` |
| `font_faces` | Optional family → `{regular,bold,italic,bold_italic}` stems. Omitted slots stay synthetic. Not CSS `font-weight` numbers |
| `modifiers` | `precedence` plus `styles[type][intent]` patches |

Role `default` must set `font_family`, `font_size`, `line_height_mult`, and `color`. Other roles inherit omitted text fields from `default`. Optional on a role: `text_align` (`start` \| `center` \| `end` \| `justify`), `bold`, `italic`, `letter_spacing_pt`, `first_line_indent_pt`, `self_align`, `list_style`, `image_fit` (`contain` default, or `cover`), `box_decoration`, `variants`. Container, `rule`, and `image` roles need not repeat typography when they do not paint text.

A node’s `variant`, if set, must exist on that role. Variant keys are only `box_decoration`, `self_align`, `text_overrides`, `list_style`, and `image_fit`. Put `bold`, `color`, `text_align`, and `font_size` under `text_overrides`, not at the variant root.

`box_decoration` values are **named primitive strings** (`background`, `border`, `corner_radius`, `shadow`, `blur`) except `padding_pt` (millipt number or per-edge object). Unknown names fail compile (`UNKNOWN_PRIMITIVE`). Inline fill or border objects on a role fail schema.

Justify expands U+0020 gaps on non-final wrapped lines only. Tracking is extra advance on every glyph except the last in a shaped run. Role `box_decoration.padding_pt` applies to text, code, math, form fields, containers, tables, and image nodes.

K2F never uses system fonts. Missing glyphs fail closed (`FONT_MISSING_GLYPH`).

Contract: [`styles.schema.json`](../../schema/styles.schema.json) and [`visual_primitives.schema.json`](../../schema/visual_primitives.schema.json). Walkthrough: [Theme and fonts](../authoring/theme.md).

## Compile and lock

`pack` / `compile` turns State A + theme + fonts (+ optional assets) into `document.K2F.lock`. Do not hand-edit the lock: change the tree or theme, then relock.

The lock records:

- `engine_version` — semver of the compiling engine
- `engine_commit_sha` — git commit of the engine build (release builds must not use `UNKNOWN`)
- **Geometry** — page tree with millipt positions and sizes
- **Render plan** — ordered paint operations (`DrawText`, `DrawImage`, fills, …)
- **Hashes** — `content_hash` (expanded semantic tree) and `appearance_hash` (semantic + theme + fonts + page config + engine identity)

Viewers paint the lock only. PDF, PPTX, DOCX, and IDML are one-way drawings of that lock, not a second source.

## Integrity verification

`k2f verify` and `inspect_package` return status codes:

| Code | Meaning |
|------|---------|
| `VALID` | Lock matches semantic content, theme, fonts, and the **reader's** engine identity |
| `UNSIGNED` | Self-consistent unsigned package (hash `VALID` or `ENGINE_MISMATCH`) |
| `SIGNED` | Self-consistent package and valid Ed25519 signature |
| `SIGNED_BUT_BROKEN` | Signature present but hash or crypto check failed |
| `UNLOCKED` | No `document.K2F.lock` |
| `CONTENT_CHANGED` | Semantic tree changed since lock |
| `APPEARANCE_CHANGED` | Theme, fonts, page config, or unbound engine identity changed since lock |
| `ENGINE_MISMATCH` | Hash chain matches the lock's recorded engine; reader's `engine_version` / `engine_commit_sha` differs |
| `FONT_MISSING` | Required embedded fonts absent |

Additional package errors: `SCHEMA_INVALID`, `NODE_ID`, `UNEXPECTED_PATH`, `UNKNOWN_PAINT_OP`, `PDF_IS_NOT_A_SOURCE`, `IMAGE_SIZE`, `SVG_TEXT`.

In-browser check: [Verify](https://k2f.dev/verify). Banner rules: [Integrity](https://k2f.dev/integrity).

## Viewer banners

Viewers must surface integrity status derived from `IntegrityStatus`. The banner **code** is always available via `banner()` / `k2f-open`; default UI uses **tiered visibility** (like PDF viewers):

| Banner | When | Default UI |
|--------|------|------------|
| `SIGNED` | Valid signature and hash chain | Compact positive strip |
| `UNSIGNED` | Self-consistent, unsigned | Hidden (quiet read) |
| `SIGNED_BUT_BROKEN` | Signature file invalid or tampered | Prominent warning |
| `BROKEN_INTEGRITY` | Content, appearance, or font failure | Prominent warning |
| `UNLOCKED` | Package not compiled | Compact draft strip |

Embedders may pass `banner: "full"` for the legacy verbose strip on every state, or `banner: "off"` / `no-banner` to hide chrome entirely. Broken states must never be silently treated as signed.

`BROKEN_INTEGRITY` and `SIGNED_BUT_BROKEN` must **not** be treated as signed.

## Signing

Signing is a human or organizational step. Agents produce `UNSIGNED` packages by default. `signatures/v1.json` records Ed25519 public key, signature over canonical lock JSON, `signed_by`, and `signed_at`.

Contract: [`signatures.schema.json`](../../schema/signatures.schema.json).

## Prohibitions

- **No scripts** — no JavaScript, WASM, or other executable content inside `.K2F`
- **No inline CSS** in semantic nodes
- **No PDF, PPTX, DOCX, or IDML as source** — those files are drawings of the lock (`PDF_IS_NOT_A_SOURCE`, `PPTX_IS_NOT_A_SOURCE`, `DOCX_IS_NOT_A_SOURCE`, `IDML_IS_NOT_A_SOURCE`)
- **No arbitrary vector paths** in State A
- **No system fonts** — faces are embedded; missing glyphs fail closed

## Embedded schemas

Each packed package embeds copies under `schema/` (format contract only):

- `manifest.schema.json`
- `nodes.schema.json`
- `styles.schema.json`
- `visual_primitives.schema.json`
- `signatures.schema.json`

Agent writing dialects live outside the format (`engine/k2f_sdk/profiles/`). Schema changes require a spec version bump and matching engine release.

Repo copies: [`schema/nodes.schema.json`](../../schema/nodes.schema.json) and the four siblings above. The skill folder ships the same five files so agents without a checkout can look them up. Do not copy schemas into an author directory.

## Related documents

| Document | What it is |
|----------|------------|
| [Introduction](../README.md) | Package model and doc map |
| [Getting started](../guide/getting-started.md) | First `.K2F` |
| [Text](../authoring/text.md) · [Images](../authoring/images.md) · [Tables](../authoring/tables.md) · [Layout](../authoring/layout.md) · [Theme](../authoring/theme.md) | How to author |
| [Allowed keys](../../skills/k2f/references/writing/fields.md) | Field cheat sheet |
| [Catalog](../../skills/k2f/catalog/README.md) | Packable examples |
| [COMPATIBILITY.md](../../COMPATIBILITY.md) | Engine matching |
| [Integrity](https://k2f.dev/integrity) · [Verify](https://k2f.dev/verify) | Banner UI and in-browser check |
