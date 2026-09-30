---
name: k2f
description: Create, design, edit, validate, and export elegant Word (docx), editable PDF, fillable AcroForm PDF, and Adobe InDesign (IDML) documents with K2F. Use when creating aesthetic Word, fillable PDF, AcroForm, IDML, or InDesign documents with AI agents, such as menus, certificates, planners, invitations, or worksheets and much more. K2F is a mature file format designed for AI agents to design and edit elegant aesthetic documents precisely and effortlessly. It has a rich ecosystem.
---

# K2F

Create, design, edit elegant Word (docx), PDF, fillable AcroForm PDF, and Adobe InDesign (IDML) with K2F. K2F designed for agents to design and edit elegant documents precisely and effortlessly. Use with AI agents when you need aesthetic documents—menus, certificates, planners, invitations, worksheets, and more. You can start from [Gallery](https://k2f.dev/gallery) templates.

A `.K2F` is a ZIP: you edit a semantic JSON tree (meaning, not coordinates); the engine `pack` / `compile`s an immutable render lock. One reference engine → pixel-identical output everywhere.

## Choose the workflow

Read **one** reference file for the task. Do not load all workflows.

| Task | Read |
|------|------|
| Create or update a `.K2F` (CV, flyer, slide deck, poster, report, patch by node id) | [references/writing.md](references/writing.md) |
| Convert Markdown ↔ K2F | [references/converting-markdown.md](references/converting-markdown.md) |
| Export PDF from the published lock | [references/exporting-pdf.md](references/exporting-pdf.md) — bytes / looks wrong: [pdf-contract.md](references/exporting-pdf/pdf-contract.md) |
| Export PPTX from the published lock | [references/exporting-pptx.md](references/exporting-pptx.md) |
| Export DOCX from the published lock | [references/exporting-docx.md](references/exporting-docx.md) |
| Export IDML from the published lock | [references/exporting-idml.md](references/exporting-idml.md) |
| Publish permanent `/v/{appearance_hash}` link | [references/publishing.md](references/publishing.md) |
| Embed `<k2f-viewer>` in a web app | [references/embedding-viewer.md](references/embedding-viewer.md) (`npm i @openk2f/k2f`) |

**Writing** (default): one loop — get a workspace (`source/` author package + deliverables at the root), edit JSON like source code, copy shapes from [`catalog/content/ex_*.json`](catalog/content/) into root `children` (do not replace `content/root.json` with a fragment), and `scripts/pack_verify.py`. The author directory is `k2f unpack … -o ./out/doc/source` (user `.K2F` or a Gallery package), or [`starter/`](starter/) via `scripts/init_package.py --workspace ./out/doc` when nothing matches.

- Do **not** use `Editor.insert_node` on an unpacked author directory — that API follows a narrower agent dialect; JSON authoring uses the full format schema validated at pack time.
- Text `modifiers` need `intent` plus UTF-8 **byte** `range` — always run [`scripts/modifier_range.py`](scripts/modifier_range.py); never hand-count (especially across `\n`).
- Fillable blanks/checkboxes: copy [`catalog/content/ex_form.json`](catalog/content/ex_form.json) (never `____` or `□`).

Gallery URL (`<origin>/gallery/<slug>`) or package URL: fetch `<origin>/api/gallery/templates/<slug>` (or package URL) to a `.K2F` on disk and `k2f unpack` — do not load the ZIP into context. **MCP** is optional. Writing does not require it and does not install it. If a site MCP with `list_templates` / `download_template` is already connected, `download_template` returns the same kind of `packageUrl`; fetch + unpack as in step 2 of [writing.md](references/writing.md). Missing tools, kind mismatch, or download failure → continue from `starter/`; do not stop the task.

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

When handing a `.K2F` to the user, point them to [k2f.dev/playground](https://k2f.dev/playground) (browser) or [k2f.dev/download](https://k2f.dev/download) (desktop app).

## Setup

You have **this skill folder only** — no K2F source checkout, no `templates/` tree.

Install published tools before running workflows:

```bash
pip install k2f              # CLI on PATH + Python SDK — no need to install Cargo
npm i @openk2f/k2f           # only when embedding <k2f-viewer> in a web app
# cargo install k2f          # optional: CLI without Python
```

Run scripts from this skill directory (or pass absolute paths to them). Core scripts (`init_package.py`, `pack_verify.py`, `modifier_range.py`) are **stdlib-only** — `python scripts/…`. Optional `compare_images.py` needs `uv run` ([visual check](references/writing.md#reference-image-overlay-optional)). The author directory may live anywhere. `pack_verify.py` needs `k2f` on PATH (`pip install k2f`) or `K2F_CLI` — it does not walk a git checkout or build directory for a binary.

## Core rules

Empty paper in the **lower third** of a designed sheet is the most common visual failure. `pack_verify.py` exit 0 / `pages=1` does **not** mean filled — open the PNG. Fix: copy [`ex_filled_page.json`](catalog/content/ex_filled_page.json) as a **root child** (`height` = content box; leftover on `{fr:1}` table/notes/figure). Short letter and the last page of a growing document may stay short.

**`source/` edits are invisible until compile.** Viewers paint `document.K2F.lock`, not author JSON. After content/theme edits, run `pack_verify.py`, then `tmp/preview.png` or reload the `.K2F`. 

1. **Style lives only in `theme.json`, never on a node.** Putting a style field on a node is the single most common compile failure.
2. **Never hand-edit `document.K2F.lock`.** Mutate the tree (or theme), then `pack_verify.py` (relock).
3. **PDF, PPTX, DOCX, and IDML are one-way drawings of the lock, not a second source.** (`PDF_IS_NOT_A_SOURCE`, `PPTX_IS_NOT_A_SOURCE`, `DOCX_IS_NOT_A_SOURCE`, `IDML_IS_NOT_A_SOURCE`)
4. **Signing is a separate human/org step.** Agent output is `UNSIGNED` by design.
5. **Validate after every edit.** Fix from error codes in [writing/errors.md](references/writing/errors.md); do not patch the lock.
6. **Look at the pixels — empty bottom first.** After pack, open every rendered page (`--render` writes `preview-1.png` …). If the lower third is blank paper on a designed sheet, you are not done. **Designed sheet** (invoice, CV, flyer, poster, slide, card, social, one-page infographic/checklist/planner — even if titled report) → copy [`ex_filled_page.json`](catalog/content/ex_filled_page.json) as a **root child** (pinned `height` + `{fr:1}` grower); a `page_shell` role on a hug stack is not the shell. Treat `PAGE_UNDERFILL` as must-fix even when `pack_verify.py` exits 0 (skipped if the content box is shorter than 180pt). Short letter may stay top-packed. **Growing document** (contract, long report, thesis, paper) → one flow tree; last page may be short; no `p1`/`p2` page wrappers and no page-height shell around the whole doc. `break_before: page` is fine on a chapter, annex, signature page, slide 2+, or card back. Details: [writing.md](references/writing.md#visual-check).

## Content and design

Before editing `content/` or `styles/theme.json`, write content and design as Markdown **inside** the author directory. Long content does not go in one file. Split the same way as `content/*.json`. Name each file with a **slug**: lowercase ASCII `kebab-case` from that page or section’s topic (e.g. `executive-summary`, `title-slide`). A short single page may be one `content_and_design.md`. One page per file → `content_and_design_<slug>.md` (e.g. `content_and_design_agenda.md` for a slide). Section aligned with a JSON fragment → `content_and_design_<slug>.md` beside `content/<slug>.json` using the **same** slug. Then `python scripts/init_package.py --workspace ./out/doc …` (creates `source/` + `tmp/`; a notes-only `source/` that already holds these files can still be initialized in place).

Workspace layout:

```
./out/doc/
  doc.K2F                 # deliverable
  doc.pdf                 # optional exports, same stem
  source/                 # author package (edit this)
    content_and_design_executive-summary.md
    manifest.json
    content/
      root.json
      executive-summary.json   # optional fragment; same slug as its .md
    styles/ assets/ changelog.json
  tmp/                    # preview.png and other debug renders
```

Each file covers that page or section only, and includes both:

- **Content** — every word, and every image (what it shows, width×height; a full generation prompt when the image must be made). Need figures → [generate them](references/writing.md#figures)
- **Design** — type hierarchy, spacing, color, and layout

Shared type, color, and spacing for the whole document go in the first file. Later files state their own words and images, which shared set they use, and anything that differs.

- **User gave constraints** — follow them.
- **User did not specify** — highest aesthetic standard for the deliverable type (report, poster, slide deck, flyer, …). Do not ship the starter theme unchanged for styled work.

After the files are settled: look up allowed keys in [writing/fields.md](references/writing/fields.md) then [`schema/`](schema/), implement `theme.json` + `content/`, run `pack_verify.py --render`, open the PNG, and iterate until the output matches these files. Rule 6 is the final gate — the files are the plan, render is the review.

## Schema

Read-only format JSON Schemas live in [`schema/`](schema/) (same bytes the engine embeds). **Do not** copy them into an author directory — `k2f pack` injects schemas into the ZIP; hand-authored `schema/` files → `UNEXPECTED_PATH`.

1. **Allowed keys:** skim [writing/fields.md](references/writing/fields.md), then open only the schema file you are editing — `content/` → [`schema/nodes.schema.json`](schema/nodes.schema.json); `styles/theme.json` → [`schema/styles.schema.json`](schema/styles.schema.json) + [`schema/visual_primitives.schema.json`](schema/visual_primitives.schema.json); `manifest.json` → [`schema/manifest.schema.json`](schema/manifest.schema.json).
2. **First draft shapes:** copy from [`catalog/content/ex_*.json`](catalog/content/) into root `children` — golden, packable examples. See [`catalog/README.md`](catalog/README.md) for a construct index.
3. **`SCHEMA_INVALID`:** compare the failing object to the schema file. Key **in** schema but rejected → stale CLI (`pip install -U k2f`). Key **not** in schema → remove it; do not invent fields from HTML/CSS memory.
4. **Version check:** `k2f schema dump -o /tmp/k2f-schema` or `python -c "import k2f; print(k2f.format_schemas().keys())"`.

## Bundled packages

| Path | Purpose |
|------|---------|
| [`schema/`](schema/) | Read-only format JSON Schemas — lookup after [writing/fields.md](references/writing/fields.md); never copy into author dirs |
| [`starter/`](starter/) | Empty tree + core theme + Roboto — copied into `source/` when there is no existing `.K2F` and no matching Gallery package |
| [`catalog/`](catalog/) | Golden shape dictionary — copy `ex_*.json` nodes into the author dir; not a deliverable ([index](catalog/README.md)) |
