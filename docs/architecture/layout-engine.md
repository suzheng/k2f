# Layout engine architecture

How the reference engine compiles semantic JSON into a deterministic render lock: two-pass layout, paint plan, fixed-point math, and modifier resolution.

**Status:** Shipped in v0.1 (paged documents)  
**Contract:** [k2f-v0.1.md](../spec/k2f-v0.1.md) · **Roadmap:** [status.md](../guide/status.md)

K2F treats document generation as a compilation process:

```
Semantic JSON (State A) → Compile (k2f_layout) → document.K2F.lock (State C) → Execute (k2f_paint / k2f_pdf)
```

- **Deterministic** — identical output on all devices
- **Efficient** — O(n) layout, no reflow or browser-like dirty bits
- **Simple** — purely functional pipeline; opening a locked file does not recompile

v0.1 implements paged documents only. Slide and infinite-canvas solvers are roadmap (there is no `LayoutSolver` trait in the codebase).

Crate map: [codebase.md](codebase.md). Design principles: [design.md](design.md).

## Contents

- [Module specifications](#module-specifications)
- [Execution flow](#execution-flow-compile-process)
- [Two-pass layout](#layoutenginelayout--two-pass-algorithm)
- [Render plan](#render-plan-visual-composition)
- [Lock file output](#lock-file-output)
- [Fixed-point layout](#fixed-point-layout)
- [Deterministic paint algorithms](#deterministic-paint-algorithms)
- [Layering and stacking contexts](#layering-and-stacking-contexts)

## Module specifications

### k2f_core — data model and lock structures

Defines `SemanticNode` (State A) and render lock structures (State C). Canonical JSON deserialization with serde. Hashes input for cryptographic binding.

```rust
struct SemanticNode {
    id: String,                  // e.g. "financials.q3.table"
    role: String,                // e.g. "warning"
    variant: Option<String>,     // e.g. "glass" (visual skin)
    content: NodeContent,
    modifiers: Vec<Modifier>,    // maxItems = 50
}

enum NodeContent {
    Text(String),
    CodeBlock(CodeBlockValue),
    Math(String),                // TeX subset; display nodes. Inline $...$ is a text modifier.
    Image { src: String, width: Pt, height: Pt },
    Container { children: Vec<SemanticNode> },
    Table(TableSpec),
    TableReference { source: String, view_mode: String, width: Pt, height: Pt },
}
```

**Shapes:** no explicit Shape node. Basic shapes are containers painted via role + variant box decoration (rect/rounded). Arbitrary paths/polygons are out of scope—use SVG or images in `assets/`.

```rust
struct GeometryNode {
    id: String,
    x: i128,       // fixed-point 1/1000 pt resolution
    y: i128,
    width: i128,
    height: i128,
    glyphs: Vec<GlyphPosition>,
    fill_rects: Vec<FillRect>,   // fraction bars, radical/delimiter rules
}
```

**Render lock (State C)** must include more than geometry:

- Positions, sizes, glyphs
- Visual composition plan (ordered paint ops): backgrounds, fills, borders, rounded corners, gradients, shadows, blur, transparency blending

This avoids delegating canonical rendering to browser-specific filters.

### k2f_text — deterministic text shaping

Handles text shaping independent of the host OS.

- Dependencies: `rustybuzz`, `unicode-bidi`, `unicode-segmentation`
- Inputs: text, font binary (`/assets/fonts`), font size, modifiers
- Outputs: glyph positions with absolute advances (fixed-point)

**Fixed-point:** all layout uses `Pt(i128)` = 1/1000 pt units. Line breaking follows Unicode Line Breaking Algorithm (UAX #14). No OS-dependent kerning or hardware smoothing.

### k2f_layout — the solver

Converts `SemanticNode` tree + theme → geometry + visual composition plan.

**Two-pass grid solver:**

| Pass | Direction | Action |
|------|-----------|--------|
| 1 — Measure | Bottom-up | Compute minimum widths/heights for text/images via `k2f_text` |
| 2 — Arrange | Top-down | Assign deterministic X/Y coordinates |

**Grid rules (Level 1):**

- Track sizes: fixed `pt`, `fr` (share leftover), or `{auto:true}` (max measured cell min-size, then leftover to `fr`)
- `{auto:true}` is not CSS `auto-fit` / `minmax` / `repeat()` / `subgrid`
- No `minmax`, `auto-fit`, `subgrid`, or `repeat()`
- No auto-placement beyond strict document order
- Alignment is constraint-based only (`start` / `center` / `end` / `stretch`); no manual x/y nudges

Pagination uses `manifest.page_config` (`width`, `height`, `margin: [top, right, bottom, left]` in millipt).

**Modifier engine:**

- Implements Modifier Absolute Rule (see [design.md](design.md))
- Max 50 modifiers per node
- Modifiers always override role styles for their byte range

### Visual engine (conceptual layer — not a crate)

v0.1 splits planning and execution:

- **Plan** — `k2f_layout/src/render_plan.rs` emits ordered paint ops into the lock
- **Execute** — `k2f_paint` rasterizes those ops; viewers never invent layout

There is no `k2f_visual` crate.

Responsibilities:

- Resolve role + variant → `box_decoration` + text styles via theme primitives
- Apply canvas/page background (manifest-level; some global background features are roadmap)
- Produce explicit draw order (stacking contexts) and blending rules for transparency
- Compute or bake blur and shadow deterministically

**Theme model:**

`palette` is the only color table. `primitives` holds named surfaces, gradients, corners, borders, shadows, blurs. Roles reference them by string.

```json
"roles": {
  "card": {
    "variants": {
      "flat":   { "box_decoration": { "background": "paper", "corner_radius": "medium" } },
      "raised": { "box_decoration": { "background": "paper", "corner_radius": "medium", "shadow": "elevation.1" } },
      "glass":  { "box_decoration": { "background": "glass_light", "blur": "background", "border": "subtle" } }
    }
  }
}
```

Theme `box_decoration` is named-only; compile inlines into lock `BoxDecoration`.

### k2f_wasm — language bindings

Three feature groups (all enabled by default):

**Compile** — produce lock JSON from semantic JSON (does not open a package):

```
compile_chunk(content_json, theme_json, font_blob) -> lock_json
compile_chunk_with_assets(...) -> lock_json
```

**Viewer** — `K2fViewer::open(package_bytes)` unpacks and paints the lock. `render_page` / `hit_test` / `export_pdf` do **not** recompile.

**SDK** — `Editor` (Rust/Python/JS), plus `markdown_to_k2f` / `k2f_to_markdown`.

Python uses `k2f_py` (`import k2f`), not WASM.

The lock embeds `engine_version` and `engine_commit_sha`. Viewers execute the lock they were given. A reader whose build identity differs reports `hash_code` `ENGINE_MISMATCH` and still paints that lock (`UNSIGNED` / `SIGNED`).

## Execution flow (compile process)

1. **Ingest and resolve styles** — map role → theme token; apply modifiers (sorted by range start) → patched base style
2. **Text shaping** — load fonts from `/assets/fonts`; compute line breaks with fixed-point math
3. **Geometry resolution** — vertical stacking with pagination (v0.1); column layouts pack children left-to-right across equal-width bands (`column_span: "all"` interrupts for full-width figures/tables). *Infinite mode: not in v0.1.*
4. **Visual composition** — resolve (role, variant) → decorations; build stacking contexts; compute/bake blur and shadows
5. **Hashing and locking** — serialize geometry; embed `content_hash`, `appearance_hash`, engine version; optional signature

## LayoutEngine::layout() — two-pass algorithm

`LayoutEngine::layout(&manifest, &ctx)` in `k2f_layout/src/lib.rs` is the core layout function.

### Step 1: Setup and validation

```rust
validate_page_config(&manifest.page_config)?;
let mut paginator = Paginator::new(page_config, canvas_mode);
```

`Paginator` (`pagination.rs`) manages paged mode (v0.1): fixed-size pages with automatic page breaks. Infinite mode is roadmap.

### Step 2: Root padding

Root node padding may inflate page margins. The function gets root padding via `padding_for_role_variant()`, adjusts margins, and re-validates.

### Step 3: Process document content

**Container root** (most documents):

```rust
for child in children {
    let measured_size = measure_node(child, constraint, ctx)?;           // Pass 1
    let (dx, child_width) = align_offset_and_size(align_mode, ...);
    let pos = paginator.allocate_space(measured.height);
    let geo = arrange_node(child, final_pos, arranged_size, ctx)?;       // Pass 2
    paginator.add_item(geo);
}
```

**Single root node:** measure, allocate space, arrange.

### Key functions

| Function | File | Pass | Role |
|----------|------|------|------|
| `measure_node()` | `measure.rs` | 1 | Size text, images, containers recursively |
| `paginator.allocate_space()` | `pagination.rs` | — | Page breaks; returns placement point |
| `align_offset_and_size()` | `alignment.rs` | — | Horizontal alignment within parent |
| `arrange_node()` | `arrange.rs` | 2 | Position glyphs and children at exact coordinates |

**Output:**

```rust
LayoutResult {
    pages: Vec<Page>   // index, width, height, root: GeometryNode
}
```

**Why two passes?** Pass 1 answers "how much space is needed?" (bottom-up). Pass 2 answers "here is your space—position yourself" (top-down). No reflow; deterministic results.

## Render plan (visual composition)

`build_render_plan()` in `k2f_layout/src/render_plan.rs` transforms positioned geometry into ordered `PaintOp` commands.

### Style resolution

`resolve_box_decoration()` in `resolved_style.rs`:

```rust
// SemanticNode: role="warning", variant="critical"
let role_style = theme.roles.get("warning")?;
let variant = role_style.variants.get("critical")?;
let named = merge_theme_decoration(base, variant_overrides);
// Compile inlines fills/borders/corners into lock paint (no string refs left).
```

### Paint operations

Example sequence for a glass-effect card:

```rust
vec![
    PaintOp::BackdropBlur { node_id: "card_1", rect: ..., radius_pt: 20000, ... },
    PaintOp::DrawBox { node_id: "card_1", rect: ..., decoration: BoxDecoration { ... } },
    PaintOp::DrawText { node_id: "title_1", rect: ..., runs: [...] },
]
```

### Drawing order

1. Backdrop effects (blur what was painted before)
2. Backgrounds (solid, gradients)
3. Borders
4. Content (text, images on top)

Children draw on top of their parents. `append_ops_for_geometry()` recurses into children after parent background ops.

### Full render plan example

```json
{
  "compositing": {
    "color_space": "srgb",
    "alpha_mode": "premultiplied",
    "blend_mode": "source_over"
  },
  "pages": [
    {
      "index": 0,
      "ops": [
        {
          "DrawBox": {
            "node_id": "document_1::page_0::background",
            "rect": { "x": 0, "y": 0, "width": 595000, "height": 842000 },
            "decoration": {
              "background": { "Inline": { "type": "solid", "color": "#FAFAFA" } }
            }
          }
        },
        {
          "DrawBox": {
            "node_id": "card_1",
            "rect": { "x": 72000, "y": 72000, "width": 300000, "height": 150000 },
            "decoration": {
              "background": { "Inline": { "type": "solid", "color": "#FFFFFF" } },
              "shadow": { "Inline": { "layers": [] } },
              "corner_radius_pt": 12000
            }
          }
        },
        {
          "DrawText": {
            "node_id": "title_1",
            "rect": { "x": 86400, "y": 79200, "width": 252000, "height": 30000 },
            "runs": [
              {
                "glyph_range": [0, 13],
                "style": { "font_family": "default", "font_size": 24000, "color": "#000000", "bold": true }
              }
            ]
          }
        }
      ]
    }
  ]
}
```

### Theme primitives example

```json
{
  "primitives": {
    "surfaces": {
      "material_background": { "type": "solid", "color": "gray_50" },
      "card_background": { "type": "solid", "color": "white" }
    },
    "corners": { "small": 4000, "medium": 12000 },
    "borders": { "subtle": { "width_pt": 500, "color": "#1111111A" } },
    "shadows": {
      "elevation.1": {
        "layers": [
          { "offset_x_pt": 0, "offset_y_pt": 2000, "blur_radius_pt": 4000, "spread_radius_pt": 0, "color": "#0000001A" },
          { "offset_x_pt": 0, "offset_y_pt": 1000, "blur_radius_pt": 1000, "spread_radius_pt": 0, "color": "#00000014" }
        ]
      }
    },
    "blurs": { "background": { "radius_pt": 20000 } }
  }
}
```

## Lock file output

`LockFile` in `k2f_core/src/lib.rs`:

- Canonical JSON serialization
- `content_hash` — semantic tree
- `appearance_hash` — semantic + theme + fonts + page config + engine identity
- `engine_version`, `engine_commit_sha`

Inside a `.K2F` package the file is always `document.K2F.lock`. Golden tests use `golden.geometry.K2F.lock` in fixtures only.

## Manifest requirements

On-disk `manifest.json` follows `schema/manifest.schema.json`:

```json
{
  "title": "Quarterly Report",
  "canvas_mode": "paged",
  "page_config": {
    "width": 595000,
    "height": 842000,
    "margin": [72000, 72000, 72000, 72000]
  },
  "engine_version": "0.1.0"
}
```

`canvas_mode` is const `"paged"` in v0.1. Content lives in `content/root.json` (with optional referenced `content/**/*.json` includes), not in manifest. In-engine `Manifest` (compile time) also carries the expanded `root` and optional `running_blocks`.

## Pagination

The lock stores a page tree:

```rust
LockFile {
    geometry: LayoutResult { pages: Vec<Page> },
    render_plan: RenderPlan,
    ...
}
```

Respects `page_config` width/height/margin. Header/footer repetition uses `running_blocks`. Node-level `break_inside` / `keep_with_next` exist on the semantic tree.

Compile may print `LAYOUT_SLACK` on stderr when a large stretched box (≥40% of the page content box) is empty at the bottom — typically the `{fr:1}` grower, not the page shell. `PAGE_UNDERFILL` reports a page content box ≥25% empty below (skipped on the last page of a multi-page document). Diagnostics are not stored in the lock; compile and `pack_verify.py` exit 0. Last page of a flow document may be short.

## Modifier limits

Schema-enforced max 50 modifiers per node.

Closed `type` enum: `emphasis`, `link`, `underline`, `strikethrough`, `subscript`, `superscript`, `math`, `syntax_highlight`.

`intent` is a free string. Unknown `type` → engine error. No built-in `font-size` / `font-family` / `color` modifier type.

## Fixed-point layout

All geometry uses `i128` with 1/1000 pt resolution. Ensures bit-identical layout on x86_64, ARM, and WASM. Eliminates f64 rounding differences.

```python
# Floating point can vary across platforms:
width = 1.234 + 5.678  # might be 7.912000000000001

# Fixed-point is exact:
width_pt = Pt(1234) + Pt(5678)  # always Pt(7912)
```

## Layout strategies

**Stack** — items flow vertically or horizontally with gaps; alignment within parent width.

**Grid** — explicit tracks (`pt` / `fr` / `{auto:true}` content size); not CSS `auto-fit`.

**Overlay** — items can overlap; explicit z-order for glass effects and badges.

## Deterministic paint algorithms

Blur is not standardized across platform renderers. Chrome and Safari can differ at pixel edges even with the same numeric blur value.

**Rule:** the engine is the canonical paint master.

- Do not specify effects as browser-native strings like `blur(10px)` in the canonical path
- Specify effects by named primitives (`blurs.background`, `elevation.1`) resolved at compile time
- Canonical blur/shadow implementations live in [`engine/k2f_core/src/effects/`](../../engine/k2f_core/src/effects/) — parameter units (fixed-point pt), premultiplied alpha, edge handling, and rounding are defined there, not by browser CSS

Either compute shadows/blur deterministically during rendering, or bake results into the lock as deterministic paint ops so viewers are dumb executors.

## Layering and stacking contexts

Glass and transparency require unambiguous composition:

- Background → surfaces → content → overlays
- Deterministic blending rules for transparency
- Optional explicit z-index/overlay semantics

## Formal rendering test suite

Mandatory for every engine PR. Golden suite: `tests/runner` compares lock geometry and render plan; a small PNG appearance gate covers shadows, cards, and running footers. Published examples are not pixel-golden — the lock is the canonical artifact.

## How K2F fits the design philosophy

| Property | How K2F achieves it |
|----------|---------------------|
| Efficient | Rust + WASM, O(n) layout, fixed-point math |
| Flexible | Paged document layout in v0.1; slide / infinite canvas are roadmap |
| Simple | Input → Transform → Output; no reflow, no scripts |
| Deterministic | Fixed-point, strict grid, pagination, engine versioning |
| AI-friendly | Semantic nodes, modifiers, table references, role + variant |
| Elegant-by-design | Global primitives + variants + deterministic visual composition |

---

*Historical note: early internal docs used the codename "The Semantic-Geometry Bridge."*
