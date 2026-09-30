# Project status

K2F is on the **0.3.x** release line. The [format spec](../spec/k2f-v0.3.md) is the on-disk contract. This page summarizes what is stable, what is preview-quality, and what is still planned.

> **Desktop reader: packaged builds** — download installers on [k2f.dev/download](https://k2f.dev/download). macOS file association and OS code signing / notarization remain on the [roadmap](#roadmap-not-shipped).  
> **MCP: stdio shipped** — run the local [`k2f_mcp`](../../engine/k2f_mcp/README.md) adapter from a repo checkout. [Remote HTTP MCP](#roadmap-not-shipped) is not shipped.

## At a glance

| Area | Status |
|------|--------|
| Format spec [v0.3](../spec/k2f-v0.3.md) | Shipped |
| CLI & SDK on PyPI, npm, crates.io (**0.3.x**) | Shipped |
| Web viewer, Playground, permanent publish URLs | Shipped |
| Export (PDF, PowerPoint, Word, InDesign IDML) | Shipped |
| [K2F agent skill](https://k2f.dev/skills/k2f) & MCP stdio | Shipped |
| Desktop reader installers | Preview (unsigned) |
| Remote HTTP MCP | Planned |

## Shipped in 0.3.x

### Format & integrity

- **Package model** — ZIP container, semantic tree, theme, embedded fonts, compiled lock
- **Verify** — hash chain and viewer banners (`SIGNED`, `UNSIGNED`, `BROKEN_INTEGRITY`, …). See [Integrity](https://k2f.dev/integrity).
- **Registries** — `pip install k2f` (PyPI), `npm i @openk2f/k2f` (npm), `cargo install k2f` (crates.io). npm uses scoped `@openk2f/k2f` because unscoped `k2f` is taken.

### Viewers & export

- **Web viewer** — paints the lock only (no DOM `fillText` for body copy). Guide: [Web viewer](web-viewer.md).
- **Markdown bridge** — `markdown_to_k2f` / `k2f_to_markdown` (display `$$...$$` nodes; inline `$...$` modifiers).
- **Native math** — display `$$` and inline `$` compiled to glyphs (TeX subset including stretchy `\left`/`\right` and `matrix` / `align` / `cases`; unknown commands fail at compile time).
- **Fillable fields** — `content.type: "form_field"`; viewers overlay inputs; Save uses `replace_text` + relock; default PDF export writes AcroForm (`--flatten` paints lock glyphs). DOCX/PPTX export lock `DrawBox` / `DrawText` (no Word content controls).
- **Office export** — PDF, PPTX, DOCX, and IDML draw from the published lock ([export overview](exporting.md)).

### Agents & automation

- **Agent skill** — workflows, catalog, and scripts in [k2f](https://github.com/suzheng/k2f) ([`skills/core/k2f/SKILL.md`](https://github.com/suzheng/k2f/blob/main/skills/core/k2f/SKILL.md)); install with `npx skills add suzheng/k2f --skill k2f`. On-site mirror: [/skills/core/k2f](https://k2f.dev/skills/k2f).
- **Layers** — skill (instructions) → SDK (implementation) → MCP stdio ([`engine/k2f_mcp`](../../engine/k2f_mcp/README.md); contracts in [`mcp_tools.json`](../../engine/k2f_mcp/mcp_tools.json), browsable at [k2f.dev/roadmap/mcp](https://k2f.dev/roadmap/mcp)).

## Preview

- **[Desktop reader](../../desktop/k2f_reader/README.md)** — native lock executor (same rules as the web viewer: opening does not recompile; pixels and exports come from `document.K2F.lock`). Packaged builds on [k2f.dev/download](https://k2f.dev/download). From source: `cargo run -p k2f_reader -- path/to/file.K2F`. Surgical edit, form Save, and lock exports match the web stack. Details: [k2f.dev/roadmap/desktop](https://k2f.dev/roadmap/desktop).

## Roadmap (not shipped)

| Item | Notes |
|------|-------|
| Remote HTTP MCP | OAuth, multi-tenant; stdio is shipped |
| Slide / infinite canvas | Product surface TBD |
| Desktop code signing / notarization | Installers exist; OS trust workflows remain roadmap |

Tracks that live only outside this public repository are not part of the public contract.

## Versioning & compatibility

Format spec [v0.3](../spec/k2f-v0.3.md) matches SDK **0.3.x**. Patch releases keep that spec unless schemas or the lock format break. See [COMPATIBILITY.md](../../COMPATIBILITY.md) for `engine_version` matching on locks. Release notes: [CHANGELOG](https://k2f.dev/changelog).
