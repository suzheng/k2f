---
name: k2f
description: Work with K2F documents (.K2F), including creating, reading, editing, validating, converting Markdown, exporting PDF, publishing permanent links, and embedding the viewer. Use when the task involves a K2F document, asks to produce a deterministic semantically editable document, create a CV/flyer/presentation/poster/report/book, patch by stable node id, convert Markdown↔K2F, export-pdf/Download PDF (not jsPDF/html2pdf), publish /v/{appearance_hash}, embed k2f-viewer in Next/Vite/React, or when UNLOCKED/PDF_IS_NOT_A_SOURCE appears.
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

**Writing:** start from [`starter/`](starter/) via `scripts/init_package.py`, or `k2f unpack` an existing `.K2F`, then edit JSON like source code, copy shapes from [`catalog/content/ex_*.json`](catalog/content/), and `scripts/pack_verify.py`. Do **not** use `Editor.insert_node` on an unpacked author directory — that API follows a narrower agent dialect; JSON authoring uses the full format schema validated at pack time.

**MCP** is not published on PyPI/npm/crates.io. This skill does not depend on MCP.

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
3. **PDF is a one-way rendering of the lock, not a second source.** (`PDF_IS_NOT_A_SOURCE`)
4. **Signing is a separate human/org step.** Agent output is `UNSIGNED` by design.
5. **Validate after every edit.** Fix from error codes in [writing/errors.md](references/writing/errors.md); do not patch the lock.
6. **Look at the pixels.** `verify` only checks hashes. After pack, render a PNG and open the image. Poster/slide empty bottom: copy [`catalog/content/ex_poster_shell.json`](catalog/content/ex_poster_shell.json) (grid + `{fr:1}` row), not a vertical stack. Type too large/small: role styles in `theme.json`. Commands: [writing.md](references/writing.md#visual-check).

## Schema

Read-only format JSON Schemas live in [`schema/`](schema/) (same bytes the engine embeds). **Do not** copy them into an author directory — `k2f pack` injects schemas into the ZIP; hand-authored `schema/` files → `UNEXPECTED_PATH`.

1. **Allowed keys:** open the relevant schema file in this skill folder — `content/` → [`schema/nodes.schema.json`](schema/nodes.schema.json); `styles/theme.json` → [`schema/styles.schema.json`](schema/styles.schema.json) + [`schema/visual_primitives.schema.json`](schema/visual_primitives.schema.json); `manifest.json` → [`schema/manifest.schema.json`](schema/manifest.schema.json). Read only what you are editing.
2. **First draft shapes:** copy from [`catalog/content/ex_*.json`](catalog/content/) — golden, packable examples. See [`catalog/README.md`](catalog/README.md) for a construct index.
3. **`SCHEMA_INVALID`:** compare the failing object to the schema file. Key **in** schema but rejected → stale CLI (`pip install -U k2f`). Key **not** in schema → remove it; do not invent fields from HTML/CSS memory.
4. **Version check:** `k2f schema dump -o /tmp/k2f-schema` or `python -c "import k2f; print(k2f.format_schemas().keys())"`.

## Bundled packages

| Path | Purpose |
|------|---------|
| [`schema/`](schema/) | Read-only format JSON Schemas (lookup only — never copy into author dirs) |
| [`starter/`](starter/) | Empty tree + core theme + Roboto — copied by `init_package.py` |
| [`catalog/`](catalog/) | Golden shape reference — `ex_*.json` per layout/content type; must compile as a whole ([index](catalog/README.md)) |
| [`looks/`](looks/) | Example design skins — `theme.json` + composition guide; use only when the user did not name a style ([index](looks/README.md)) |

## Choose the workflow

Read **one** reference file for the task. Do not load all workflows.

| Task | Read |
|------|------|
| Create or update a `.K2F` (CV, flyer, slide deck, poster, report, patch by node id) | [references/writing.md](references/writing.md) |
| Convert Markdown ↔ K2F | [references/converting-markdown.md](references/converting-markdown.md) |
| Export PDF from the published lock | [references/exporting-pdf.md](references/exporting-pdf.md) |
| Publish permanent `/v/{appearance_hash}` link | [references/publishing.md](references/publishing.md) |
| Embed `<k2f-viewer>` in a web app | [references/embedding-viewer.md](references/embedding-viewer.md) |
