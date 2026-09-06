---
name: k2f
description: Work with K2F documents (.K2F), including creating, reading, editing, validating, converting Markdown, exporting PDF, PPTX, or Word, publishing permanent links, and embedding the viewer. Use when the task involves a K2F document, asks to produce a deterministic semantically editable document, create a CV/flyer/presentation/poster/report/book, patch by stable node id, convert Markdown↔K2F, export-pdf/Download PDF (not jsPDF/html2pdf), export-pptx/Download PowerPoint, export-docx/Download Word, publish /v/{appearance_hash}, embed k2f-viewer in Next/Vite/React, or when UNLOCKED/PDF_IS_NOT_A_SOURCE/PPTX_IS_NOT_A_SOURCE/DOCX_IS_NOT_A_SOURCE appears.
---

# K2F

## Environment

You have **this skill folder only** — no K2F source checkout, no `templates/` tree.

Install published tools before running workflows:

```bash
pip install k2f              # CLI on PATH + Python SDK — no need to install Cargo
npm i @openk2f/k2f           # only when embedding <k2f-viewer> in a web app
# cargo install k2f          # optional: CLI without Python
```

**Writing:** one loop — get an unpacked author directory, edit JSON like source code, copy shapes from [`catalog/content/ex_*.json`](catalog/content/), and `scripts/pack_verify.py`. The working directory comes from `k2f unpack` (user `.K2F` or a Gallery package), or [`starter/`](starter/) via `scripts/init_package.py` when nothing matches. Do **not** use `Editor.insert_node` on an unpacked author directory — that API follows a narrower agent dialect; JSON authoring uses the full format schema validated at pack time.

**MCP** is optional. Writing does not require it and does not install it. If a site MCP with `list_templates` / `download_template` is already connected, use it in step 2 of [writing.md](references/writing.md). Missing tools, kind mismatch, or download failure → continue from `starter/`; do not stop the task.

Run scripts from this skill directory (or pass absolute paths to them).

## What K2F is

PDF is unreadable and uneditable for AI. HTML/CSS renders differently on every device. K2F fixes both: agents write plain JSON *meaning*, and one reference engine turns that meaning into pixel-identical output everywhere.

Instead of positioning content with coordinates:

    {"text": "WARNING!", "x": 100, "y": 50, "color": "red", "bold": true}

K2F authors describe meaning:

    {"id": "doc.warning", "role": "warning", "content": {"type": "text", "value": "WARNING!"}}

Every `.K2F` file is a ZIP holding two states:

- **State A — semantic tree** (`content/root.json`, mutable, the file you edit). Every node has a stable dotted `id`, a `role`, an optional `variant`, and optional `modifiers` (inline emphasis/link/math on a byte range). No coordinates, no colors, no font sizes — those live only in `styles/theme.json`, keyed by role/variant.
- **State C — render lock** (`document.K2F.lock`, immutable, generated). Exact geometry, paint operations, and integrity hashes. Every viewer and every PDF export paints *this*, not the tree.

The engine — not the agent — turns A into C by running `pack` / `compile`. Other package paths: `manifest.json` (page size, margins), `assets/fonts|images|data/*`, `changelog.json` (edit history), optional `signatures/v1.json`.

## Core rules

1. **Style lives only in `theme.json`, never on a node.** Putting a style field on a node is the single most common compile failure.
2. **Never hand-edit `document.K2F.lock`.** Mutate the tree (or theme), then relock.
3. **PDF, PPTX, and DOCX are one-way drawings of the lock, not a second source.** (`PDF_IS_NOT_A_SOURCE`, `PPTX_IS_NOT_A_SOURCE`, `DOCX_IS_NOT_A_SOURCE`)
4. **Signing is a separate human/org step.** Agent output is `UNSIGNED` by design.
5. **Validate after every edit.** Fix from error codes in [writing/errors.md](references/writing/errors.md); do not patch the lock.
6. **Look at the pixels.** `verify` only checks hashes. After pack, render a PNG and open the image. Poster/slide empty bottom: copy [`catalog/content/ex_poster_shell.json`](catalog/content/ex_poster_shell.json); compile may print `LAYOUT_SLACK` → [writing/errors.md](references/writing/errors.md). Type too large/small: role styles in `theme.json`. Commands: [writing.md](references/writing.md#visual-check).

## Design first

Before editing `content/` or `styles/theme.json`, write a **design specification** as Markdown (e.g. `design.md` next to the author directory).

- **User gave design constraints** — the spec follows their requirements.
- **User did not specify** — design to the highest aesthetic standard for the deliverable type (report, poster, slide deck, flyer, …). Do not ship the starter theme unchanged for styled work.

The spec must be concrete enough to implement. Cover what applies:

- Typography — fonts, role hierarchy, size intent per canvas
- Spacing — line rhythm, paragraph gaps, stack/grid gaps
- Color and surfaces — page background, accent, cards and section treatment
- Layout — margins, columns, hero zones, headers/footers, figure placement
- Non-text elements — images, tables, decorative assets and their proportions

After the spec is settled: look up allowed keys in [writing/fields.md](references/writing/fields.md) then [`schema/`](schema/), implement `theme.json` + `content/`, run `pack_verify.py --render`, open the PNG, and iterate until the output matches the spec. Rule 6 is the final gate — the spec is the plan, render is the review.

## Schema

Read-only format JSON Schemas live in [`schema/`](schema/) (same bytes the engine embeds). **Do not** copy them into an author directory — `k2f pack` injects schemas into the ZIP; hand-authored `schema/` files → `UNEXPECTED_PATH`.

1. **Allowed keys:** skim [writing/fields.md](references/writing/fields.md), then open only the schema file you are editing — `content/` → [`schema/nodes.schema.json`](schema/nodes.schema.json); `styles/theme.json` → [`schema/styles.schema.json`](schema/styles.schema.json) + [`schema/visual_primitives.schema.json`](schema/visual_primitives.schema.json); `manifest.json` → [`schema/manifest.schema.json`](schema/manifest.schema.json).
2. **First draft shapes:** copy from [`catalog/content/ex_*.json`](catalog/content/) — golden, packable examples. See [`catalog/README.md`](catalog/README.md) for a construct index.
3. **`SCHEMA_INVALID`:** compare the failing object to the schema file. Key **in** schema but rejected → stale CLI (`pip install -U k2f`). Key **not** in schema → remove it; do not invent fields from HTML/CSS memory.
4. **Version check:** `k2f schema dump -o /tmp/k2f-schema` or `python -c "import k2f; print(k2f.format_schemas().keys())"`.

## Bundled packages

| Path | Purpose |
|------|---------|
| [`schema/`](schema/) | Read-only format JSON Schemas — lookup after [writing/fields.md](references/writing/fields.md); never copy into author dirs |
| [`starter/`](starter/) | Empty tree + core theme + Roboto — the working directory when there is no existing `.K2F` and no matching Gallery package |
| [`catalog/`](catalog/) | Golden shape dictionary — copy `ex_*.json` nodes into the author dir; not a deliverable ([index](catalog/README.md)) |

## Choose the workflow

Read **one** reference file for the task. Do not load all workflows.

| Task | Read |
|------|------|
| Create or update a `.K2F` (CV, flyer, slide deck, poster, report, patch by node id) | [references/writing.md](references/writing.md) |
| Convert Markdown ↔ K2F | [references/converting-markdown.md](references/converting-markdown.md) |
| Export PDF from the published lock | [references/exporting-pdf.md](references/exporting-pdf.md) — bytes / looks wrong: [pdf-contract.md](references/exporting-pdf/pdf-contract.md) |
| Export PPTX from the published lock | [references/exporting-pptx.md](references/exporting-pptx.md) |
| Export DOCX from the published lock | [references/exporting-docx.md](references/exporting-docx.md) |
| Publish permanent `/v/{appearance_hash}` link | [references/publishing.md](references/publishing.md) |
| Embed `<k2f-viewer>` in a web app | [references/embedding-viewer.md](references/embedding-viewer.md) |
