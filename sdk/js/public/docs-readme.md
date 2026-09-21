# Introduction

K2F is a document format for **pixel-identical, semantically editable** documents. You author **meaning** as JSON; one reference engine compiles it, and every viewer paints the same output.

## Package model

A `.K2F` file is a ZIP. You edit two layers; `k2f pack` / `compile` writes the third:

| Layer | Path | What it is |
| --- | --- | --- |
| **Content** | `content/root.json` | Semantic tree: stable `id`, `role`, and `content`. No coordinates, colors, or font sizes. |
| **Theme** | `styles/theme.json` | Appearance keyed by role (and optional variant). Fonts live here and under `assets/fonts/`. |
| **Lock** | `document.K2F.lock` | Compiled geometry and paint operations. Viewers and exports paint **this**. Do not hand-edit. |

`manifest.json` sets page size and margins; `assets/` holds fonts and images. Change content or theme, then relock.

## Quick install

```bash
pip install k2f
```

Python 3.9+. Optional: `npm i @openk2f/k2f` for the [web viewer](guide/web-viewer.md); `cargo install k2f` for a CLI without Python. For the full author loop (skill folder, `init_package.py`, `pack_verify.py`), see [Getting started](guide/getting-started.md).

## Get started

- [Getting started](guide/getting-started.md) — agent skill or CLI, first `.K2F`
- [Format spec](spec/k2f-v0.3.md) — normative contract (container, processing, integrity)

## Authoring

1. Copy golden nodes from the [catalog](reference/catalog.md) into `content/root.json` `children` (keep `id: "root"`; do not replace the file with a catalog fragment).
2. Look up allowed keys in [Allowed keys](reference/keys.md), then the matching file under [`schema/`](../skills/k2f/schema/).

Topic guides:

- [Text](authoring/text.md) — paragraphs, headings, lists, inline marks
- [Images](authoring/images.md) — embedded bitmaps and SVG
- [Tables](authoring/tables.md) — inline tables
- [Layout](authoring/layout.md) — stack and grid
- [Theme and fonts](authoring/theme.md) — roles, variants, embedded fonts

## Reference

- [Catalog](reference/catalog.md) — packable `ex_*.json` examples
- [Allowed keys](reference/keys.md) — node, theme, and manifest fields
- [Format spec](spec/k2f-v0.3.md)

## Export

Markdown or JSON → packed `.K2F` → PDF, PowerPoint, Word, or InDesign. The lock is the source; each file is a drawing of it.

- [Export](guide/exporting.md) — pipeline and format comparison
- [PDF](guide/exporting-pdf.md) — fillable AcroForm by default
- [PowerPoint](guide/exporting-pptx.md)
- [Word](guide/exporting-docx.md)
- [InDesign](guide/exporting-idml.md)

## Guides

- [Markdown conversion](guide/markdown.md) — Markdown ↔ K2F
- [Web viewer](guide/web-viewer.md) — embed `<k2f-viewer>` (serve packages over HTTP, not `file://`)

Permanent publish links (`/v/{appearance_hash}`) are documented in the [K2F Skill](https://github.com/suzheng/k2f-skills/blob/main/skills/core/k2f/SKILL.md) (`references/publishing.md`).

## Agents and tools

- [K2F Skill](https://github.com/suzheng/k2f-skills/blob/main/skills/core/k2f/SKILL.md) — `npx skills add suzheng/k2f-skills --skill k2f`; workflows for create, convert, export, and embed · [k2f.dev/skills/k2f](https://k2f.dev/skills/k2f)
- [Playground](https://k2f.dev/playground) — compile and preview in the browser
- [Verify](https://k2f.dev/verify) — check a `.K2F` file for integrity banners

## Project

- [Status and roadmap](guide/status.md)
- [Desktop reader (in development)](../desktop/k2f_reader/README.md) — native lock executor (same rules as the web viewer)
- [Contributing](../CONTRIBUTING.md) · [Security](../SECURITY.md) · [Compatibility](../COMPATIBILITY.md)

Contributor internals (engine design, crate map): [`architecture/`](architecture/README.md). Not part of the authoring path.
