# K2F design goals and principles

Why K2F exists, what it optimizes for, and the design rules that must not be compromised.

**Status:** Shipped in v0.1 (paged documents); slide and infinite-canvas modes are roadmap only  
**Contract:** [k2f-v0.1.md](../spec/k2f-v0.1.md) · **Roadmap:** [status.md](../guide/status.md)

## Motivation

PDF is difficult for AI systems to read and nearly impossible to edit reliably. Presentation formats (PPT) and whiteboard exports share the same problem: meaning is trapped in pixels or proprietary layout.

K2F is a new document format designed for the AI era with these goals:

1. **AI-readable and AI-writable content** — text, tables, and structure are expressed as a logical JSON tree, not opaque drawing instructions.
2. **AI-updatable styling** — appearance is controlled through roles, variants, and named primitives rather than one-off inline styles.
3. **Deterministic cross-device rendering** — the same package, theme, fonts, and pinned engine version produce the same canonical output on every device and tool, comparable to PDF consistency.
4. **Optional broader scope** — the format may eventually replace some PPT and image-export use cases (slides, infinite canvases). These are design targets, not shipped in v0.1.
5. **Minimal schema** — the core specification stays deliberately small and stable.

Core philosophy: **Semantic Intent. Deterministic Geometry.**

## Non-negotiable principles

These five principles define K2F's competitive advantage. They must not be compromised as the format evolves.

1. **Semantic-first, AI-native source** — K2F represents meaning, structure, and intent. AI edits semantic content, never pixels or arbitrary presentation instructions.

2. **Deterministic canonical rendering** — The same document, assets, theme, and pinned engine specification must produce the same canonical output everywhere.

3. **Strict intent–presentation separation** — Semantic content may reference controlled roles and variants, but arbitrary visual styling must never leak into the semantic layer.

4. **Minimal and stable core** — The core specification remains deliberately small and stable. New capabilities should prefer themes, primitives, or extensions; core changes require strong justification and backward compatibility whenever possible.

5. **Self-contained canonical representation** — Everything required for canonical interpretation and rendering must be explicitly contained or version-bound. External, platform-dependent behavior must never affect canonical output.

## Conceptual model

K2F separates **what** a document means from **where** it appears on the page.

```
State A (semantic tree)  →  Reference engine  →  State C (render lock)
     content/root.json         compile               document.K2F.lock
```

### State A — semantic tree

- JSON describing semantic content (headings, tables, text, containers).
- Mutable; contains no physical coordinates.
- Uses hierarchical dotted IDs and semantic roles.
- Lives at `content/root.json` inside the `.K2F` ZIP. Long documents may add referenced `content/**/*.json` fragments via explicit `{ "include": "content/...." }` stubs in container `children` (resolved at pack time into one tree). See [k2f-v0.1.md](../spec/k2f-v0.1.md) for the full container layout.

### Reference engine (not a third on-disk state)

The reference engine is a family of Rust crates. It is **not** a file inside the package.

| Phase | Crate | Function |
|-------|-------|----------|
| Layout | `k2f_layout` | `f(content, theme) → geometry` — deterministic, immutable per compile |
| Paint plan | `k2f_layout` (`render_plan.rs`) | `g(geometry, theme.primitives) → render_plan` — emitted into the lock |
| Execute | `k2f_paint` | Rasterizes the lock; viewers do not re-layout |
| Package | `k2f_package` | ZIP pack/unpack, schema validation, signatures |

Bindings include native CLI, desktop reader, Python (`k2f_py`), and WASM (`k2f_wasm`). WASM is one binding, not the only runtime.

### State C — render lock

- Package path: `document.K2F.lock` (golden fixtures may use `golden.geometry.K2F.lock`).
- Contains exact coordinates, glyph positions, a deterministic render plan (fills, strokes, gradients, shadows, blur) in explicit draw order, plus `content_hash` and `appearance_hash`.
- Immutable. Opening a locked `.K2F` file paints this lock; it does not compile.

## v0.1 scope vs roadmap

| Capability | v0.1 | Roadmap |
|------------|------|---------|
| `canvas_mode: "paged"` | Shipped | — |
| Slide mode | — | Design target |
| Infinite canvas (`canvas_mode: "infinite"`) | — | Design target; tile streaming like map viewers |
| Per-chapter `content/*.json` include chunking | Shipped | Explicit `{ "include": "content/...." }` stubs in `content/root.json`; no `index.json` |
| Page sheet fill | Shipped | Root role `box_decoration.background` paints the full page (including margins). Not a `page_config` field. |
| `canvas_style` in manifest | — | Design target |

See [status.md](../guide/status.md) for the current shipped surface.

## Semantic layer

### Stable anchors (hierarchical IDs)

Format: `parent_id.child_type_semantic_name`

Example: `financials.q3_results.table_revenue`

Stable IDs let agents update only the intended nodes.

### Roles and variants

Content nodes express intent, not exact style.

```json
{ "role": "warning", "variant": "critical", "content": "Do not sign this." }
```

- **Role** — semantic meaning and structure (e.g. `warning`, `card`, `h1`).
- **Variant** — visual skin without role explosion (e.g. `role: "card", variant: "glass"`).

Role names, modifier types, and theme rules are defined in [k2f-v0.1.md](../spec/k2f-v0.1.md).

### Data handling

Small tables inline in JSON; large tables in `assets/data/`. Nodes reference external datasets so agents can swap sources without breaking structure.

### Semantic modifiers

One-off formatting (bold a word, add a link) must not bloat `theme.json`. Text nodes use a strict `modifiers` array:

- `type` is a closed enum: `emphasis`, `link`, `underline`, `strikethrough`, `subscript`, `superscript`, `math`, `syntax_highlight`.
- No built-in `font-size`, `font-family`, or `color` modifier types; unknown types fail compile.
- `range` is UTF-8 **byte** offsets `[start, end)` on character boundaries (not character indices).
- `intent` is a free string (e.g. `"critical"` for emphasis, URL for links).

```json
{
  "text": "Do not sign this.",
  "modifiers": [
    { "range": [0, 6], "type": "emphasis", "intent": "critical" }
  ]
}
```

Nodes have no `font_variant`. The engine maps intents to consistent visual styling.

### Modifier Absolute Rule

Modifiers can conflict with role base styles (e.g. role says gray text, emphasis says bold + black). Resolution is deterministic:

1. Group modifiers by `type`.
2. Sort within each type by range start position.
3. Apply in strict type precedence order.
4. Merge non-conflicting properties when ranges overlap.
5. Raise deterministic warnings or errors for true conflicts.

Modifiers are post-processing overlays that **always win** over role base styles for the affected byte range. Because arbitrary inline styles are forbidden and only semantic intents are allowed, the engine controls clashes—not the author.

Implementation details: [layout-engine.md](layout-engine.md).

### Agent authoring dialect vs on-disk schema

SDK agents use a narrower role set from `engine/k2f_sdk/profiles/agent_v0.schema.json`:

`document`, `section`, `h1`–`h4`, `body`, `warning`, `card`, `table`, `table_header_cell`, `table_row_cell`, `list_item`, `code`, `quote`, `rule`, `math`, `running_header`, `running_footer`, `signature_block`.

That profile is an SDK authoring dialect. It is **not** part of the on-disk format schema set embedded in packages.

Variants must be defined for each role in `styles/theme.json`. Optional `styles/tokens.json` is a data file, not a separate JSON Schema.

## Presentation layer

### Global styling only

Inline styles are forbidden. All styles live in `styles/theme.json`. Nodes reference appearance via role (+ optional variant) and modifier intents.

Layout uses the engine's own stack/grid constraints—not CSS Flexbox, floats, or reflow logic.

### Layout hints

Declarative hints are allowed; the engine calculates exact positions.

```json
"layout": { "type": "stack", "direction": "vertical", "gap": 8000 }
```

Alignment is expressed only as deterministic constraints within a parent box. Manual `x`/`y` coordinates are not permitted in State A.

### Visual primitives

`palette` is the only color table. Theme `box_decoration` references named atoms (`background: "paper"`, `corner_radius: "small"`); compile inlines them into lock paint. Unknown names fail (`UNKNOWN_PRIMITIVE`). Inline fill/shadow/blur objects are invalid in theme JSON.

Official atoms include:

- Surfaces: `paper`, `warning_bg`, `glass_light` / `glass_dark`, …
- Corners: `none` / `small` / `medium` / `large` / `full`
- Borders: `subtle` / `contrast`
- Shadows: `elevation.1`–`3` (two-layer stacks)
- Blur: `background`

`card` variants `flat` / `raised` / `glass` exist in official themes; default document trees must not apply shadow or blur unless intended.

Basic shapes are decorated boxes via role + variant (no arbitrary vector paths in State A). Use SVG or images in `assets/` for complex graphics.

### Deterministic paint

Blur and shadow must be deterministic. Do not rely on browser-native filters for canonical output—compute or bake effects in the engine using spec-defined algorithms. Details: [layout-engine.md](layout-engine.md#deterministic-paint-algorithms).

### Layering and composition

When transparency is supported (e.g. glass variants), the engine must define stacking contexts—explicit draw order and deterministic layering—so composition is unambiguous.

## Technical determinism

The reference engine enforces consistency through:

- **Embedded fonts only** — no system fonts.
- **Strict text shaping** — `rustybuzz` (HarfBuzz algorithm, Rust port).
- **Math rendering** — IEEE-754.
- **Line breaking** — Unicode Line Breaking Algorithm (UAX #14).
- **Unicode normalization** — NFC for consistent hashing.
- **Deterministic paint** — shadows, blur, and gradients use specified algorithms, not platform renderers.

Engine version is pinned in the lock (`engine_version`, `engine_commit_sha`). See [COMPATIBILITY.md](../../COMPATIBILITY.md).

## Cryptographic binding

For documents that require integrity guarantees:

1. Hash the semantic tree → `content_hash`.
2. Compile `document.K2F.lock` (geometry + render plan).
3. Bind `appearance_hash` (semantic + theme + fonts + page config + engine identity).
4. Optionally sign via `signatures/v1.json` (Ed25519).

Viewers report `BROKEN_INTEGRITY` on content, appearance, or font failure. A reader whose engine identity differs from the lock reports `hash_code` `ENGINE_MISMATCH` and stays `UNSIGNED` or `SIGNED`.

## Operational constraints

### Self-describing schema

Embedded `/schema/` validates package content and enables agents to self-repair against format rules.

### Canonical JSON

Keys sorted alphabetically, 2-space indentation. Git diffs show semantic changes only.

### Prohibited features

- Scripts or executable code
- Interactive or reflow behaviors
- Base64 embedding inside JSON (binaries live in `/assets/`)
- Template variables (K2F stores final content, not logic)
- Arbitrary vector paths in semantic JSON
- Unbounded paint effects (no arbitrary CSS filter strings or platform-defined blur/shadow in the canonical path)

## Infinite canvas (design target — not in v0.1)

Modern flowcharts and whiteboard tools need unbounded space. The proposed model:

- `canvas_mode: "infinite"` in `manifest.json` (schema currently allows only `"paged"`).
- No page breaks; strict X/Y coordinates on an infinite plane.
- Viewer streams tiles (similar to map viewers).
- Could eventually replace some PPT, Miro, and Figma export workflows.

This remains roadmap, not shipped.

## Related documents

- [k2f-v0.1.md](../spec/k2f-v0.1.md) — format contract (paths, fields, validation). Spec "State B" means the compile phase (`k2f_layout`); it is not a third on-disk file in the package.
- [agent_v0.md](../instructions/agent_v0.md) — narrow SDK agent dialect (not the full format)
- [codebase.md](codebase.md) — repository and crate map
- [layout-engine.md](layout-engine.md) — compile pipeline, two-pass layout, paint plan
- [editor.md](editor.md) — future human writing UI (design proposal)

---

*Historical note: early internal docs used the codename "The Semantic-Geometry Bridge."*
