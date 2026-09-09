# Codebase overview

Map of the K2F repository for contributors: crates, compile vs execute, and where to find code by responsibility.

**Status:** Shipped in v0.1  
**Contract:** [k2f-v0.1.md](../spec/k2f-v0.1.md) · **Roadmap:** [status.md](../guide/status.md)

## Skill audience (non-negotiable)

**The agent skill under [`skills/k2f/`](../../skills/k2f/SKILL.md) is for external developers.** They copy that folder into Cursor (or another agent) and **never see this repository**: no crates, no `templates/`, no repo-root `schema/`, no `engine/`, no `target/debug/k2f`.

Everything the skill says, and every path it links to, must work in that world. Allowed references: files **inside the skill folder** (including read-only `skills/k2f/schema/*.schema.json`), plus **published** packages (`pip install k2f` puts the CLI on PATH, `npm i @openk2f/k2f`, optional `cargo install k2f`). Forbidden: repo-relative paths, checkout assumptions, “rebuild the engine,” MCP that is not on a registry, or “open `k2f/templates/` on disk.”

This architecture document is for **contributors**. Do not treat it as something the skill may cite. When you change engine behavior, update the skill’s **starter/catalog/scripts/schema** so an agent with only that folder can still succeed.

## What is K2F?

Instead of positioning content with coordinates:

```json
{"text": "WARNING!", "x": 100, "y": 50, "color": "red", "bold": true}
```

K2F authors describe meaning:

```json
{"content": "WARNING!", "role": "warning"}
```

The reference engine computes position, color, and appearance—consistently on every device. Opening a locked `.K2F` file does **not** re-layout body text; viewers execute `document.K2F.lock` only.

Design rationale: [design.md](design.md). Format fields: [k2f-v0.1.md](../spec/k2f-v0.1.md).

## Project structure

K2F is a Cargo workspace. Layout is one stage; packaging, lock execution, PDF export, and agent APIs are separate crates.

```
engine/
├── k2f_core/          # Semantic tree, LockFile, hashes, paint-plan types
├── k2f_text/          # Fonts, rustybuzz shaping, bidi, fallback
├── k2f_layout/        # Compile State A → document.K2F.lock
├── k2f_paint/         # Open package, rasterize lock, text layer, hit-test
├── k2f_package/       # ZIP pack/unpack, schema validation, signatures
├── k2f_pdf/           # Draw lock to PDF (not a second layout engine)
├── k2f_sdk/           # Agent Editor, Markdown
├── k2f_mcp/           # MCP stdio server (tool contract: mcp_tools.json)
├── k2f_py/            # Python bindings (`import k2f`; published on PyPI as `k2f`)
└── k2f_wasm/          # WASM: compile, viewer, sdk
cli/k2f_cli/           # `k2f` CLI (pack, compile, verify, export-pdf, markdown, …)
desktop/k2f_reader/    # Native lock executor (in development)
sdk/js/                # JS SDK + `<k2f-viewer>` web component
tests/runner/          # Golden suite runner
```

| Crate / surface | Role |
|---|---|
| `k2f_core` | Semantic tree, `LockFile`, hashes, paint plan types |
| `k2f_text` | Fonts, shaping (`rustybuzz`), bidi, fallback |
| `k2f_layout` | State A → lock (`compile.rs`) |
| `k2f_paint` | Open package, rasterize lock, text layer, hit-test |
| `k2f_package` | ZIP, schema validation, Ed25519 signatures |
| `k2f_pdf` | Lock → PDF drawing |
| `k2f_sdk` | Agent `Editor`, Markdown |
| `k2f_mcp` | MCP stdio adapter; same tool names as SDK |
| `k2f_py` | Python bindings (`import k2f`) |
| `k2f_wasm` | `compile` / `viewer` / `sdk` WASM bindings |
| `cli/k2f_cli` (`k2f` on crates.io) | CLI |
| `desktop/k2f_reader` | Native lock executor (source crate; not an installer) |
| `sdk/js/` | `<k2f-viewer>` + JS SDK |

## Compile vs execute

**Compile** (once, when content or theme changes):

```
Semantic tree + theme + fonts → Measure → Arrange → Render plan → document.K2F.lock
```

**Execute** (every open in a viewer):

```
.K2F ZIP → inspect lock → k2f_paint rasterize / k2f_pdf draw  (no recompile)
```

Entry points:

1. **Compile** — `LayoutEngine::compile_chunk()` in `k2f_layout` (also CLI `k2f compile` and SDK save)
2. **Open/paint** — `OpenedDocument::open()` in `k2f_paint` (web viewer, desktop reader, `k2f render`)

Algorithm details: [layout-engine.md](layout-engine.md).

## Engine properties

These describe how the reference engine behaves. They complement—but are not identical to—the immutable design principles in [design.md](design.md).

- **Meaning first** — authors describe what things are (headings, warnings, data), not how they look
- **Deterministic** — same input produces the same output on every platform
- **Linear-time layout** — O(n) measure + arrange; no reflow dirty bits
- **Adaptable themes** — different document types via theme and role/variant, not inline styles
- **One-way pipeline** — viewers are dumb lock executors; relock happens in SDK/CLI when content changes

## Module breakdown

### k2f_core — data model and validation

Defines semantic nodes, geometry nodes, lock file structures, and validation rules.

Key files:

- `lib.rs` — main data types and business rules
- `visual_primitives.rs` — named surfaces, shadows, borders, blurs
- `paint_plan.rs`, `paint_types.rs` — drawing instruction types
- `theme_vocab.rs`, `role_variant_validation.rs` — theme cross-checks
- `effects/` — blur and shadow algorithms
- `layout_hints.rs` — stack/grid layout hint types

Core structures (*illustrative; Rust is the source of truth*):

```python
class SemanticNode:
    id = "warning_1"
    role = "warning"
    variant = "critical"          # optional
    content = "DANGER!"
    modifiers = [...]             # max 50
    layout = "vertical_stack"     # if container

class GeometryNode:
    id = "warning_1"
    x, y, width, height           # fixed-point 1/1000 pt
    glyphs = [...]
    children = [...]
```

### k2f_text — deterministic text

Font loading, shaping (`rustybuzz`), bidi, fallback, and coverage. Ensures identical glyph metrics across OSes.

Key files: `lib.rs`, `shape.rs`, `font.rs`, `bidi.rs`, `fallback.rs`, `coverage.rs`

### k2f_layout — compile pipeline

Compiles the semantic tree into `document.K2F.lock`.

Key files:

- `lib.rs` — `LayoutEngine::compile_chunk()` / `layout()`
- `compile.rs` — parse, validate, hash, serialize lock
- `measure.rs` — pass 1 (sizing)
- `arrange.rs` — pass 2 (positioning)
- `pagination.rs` — page breaks
- `theme/` — theme JSON loading and primitive resolution
- `text_layout/` — wrapping, alignment
- `grid.rs`, `alignment.rs` — grid/table and alignment
- `render_plan.rs` — ordered paint ops

v0.1 is **paged document** only (`canvas_mode: "paged"`).

### k2f_paint — lock executor

`OpenedDocument::open`, `render_page`, text-layer spans, hit-test. Used by `<k2f-viewer>` and `desktop/k2f_reader`. Does not call `compile_chunk`.

### k2f_package — container

ZIP pack/unpack, embedded schema validation, integrity inspect, Ed25519 sign/verify.

Canonical paths: `content/root.json`, optional referenced `content/**/*.json`, `document.K2F.lock`.

Content includes are resolved in `k2f_package` (`engine/k2f_package/src/includes/`): load/unpack expands stubs into one `SemanticNode` tree; pack collapses mapped node ids back to on-disk fragments. `k2f_layout` and viewers never see include stubs.

### k2f_pdf — PDF bridge

Draws the lock into PDF. PDF is not a reversible source (`PDF_IS_NOT_A_SOURCE`).

### k2f_sdk / k2f_mcp / k2f_py / k2f_wasm — agent surfaces

- **k2f_sdk** — `Editor` (open package bytes or author dir, edit by node id + relock), Markdown
- **k2f_mcp** — stdio MCP server; tool contract at `engine/k2f_mcp/mcp_tools.json`
- **k2f_py** — PyO3 bindings; published on PyPI as `k2f` (CLI console script + `import k2f`)
- **k2f_wasm** — three feature groups: `compile`, `viewer`, `sdk`

## Dependencies

```
sdk/js, k2f, desktop/k2f_reader, k2f_mcp
    → k2f_paint, k2f_pdf, k2f_wasm (viewer)
k2f_sdk / k2f_py / k2f_mcp
    → k2f_layout, k2f_package, k2f_pdf
k2f_wasm (compile / sdk)
    → k2f_layout, k2f_sdk
k2f_package → k2f_core, k2f_layout
k2f_layout → k2f_core, k2f_text
k2f_paint → k2f_core
k2f_text → k2f_core
```

Viewers never depend on `k2f_layout` at open time.

## File organization by job

| Job | Location |
|-----|----------|
| Data models | `k2f_core/src/lib.rs`, `visual_primitives.rs`, `paint_*.rs` |
| Compile entry | `k2f_layout/src/compile.rs` |
| Measure / arrange | `k2f_layout/src/measure.rs`, `arrange.rs`, `pagination.rs` |
| Grid / tables | `k2f_layout/src/grid.rs` |
| Text shaping | `k2f_text/src/`, `k2f_layout/src/text_layout/` |
| Theme resolution | `k2f_layout/src/theme/`, `style.rs`, `resolved_style.rs` |
| Paint plan | `k2f_layout/src/render_plan.rs` |
| Effects | `k2f_core/src/effects/` |
| Lock execution | `k2f_paint/` |
| Tests | `*_tests.rs`, `tests/e2e_compile.rs`, `tests/runner/` |

## Testing

- **Unit tests** — individual components (text measure, stack height, etc.)
- **Integration tests** — full JSON → lock pipeline
- **Determinism tests** — identical output across repeated runs and platforms
- **Golden suite** — `tests/runner` compares lock **geometry / render plan / content hash** (not engine SHA). A small set of PNG files is the appearance gate for frozen paint-algorithm fixtures (shadows, cards, running footers), not published examples.

Local (fast):

```bash
cargo run -p k2f-test-runner -- run
```

CI uses `--dataset-runs 5` plus `--check-png` (committed visual-gate pages only).

## Related documents

- [design.md](design.md) — design principles and semantic model
- [layout-engine.md](layout-engine.md) — two-pass layout, paint plan, fixed-point math
- [editor.md](editor.md) — future human writing UI (design proposal)
