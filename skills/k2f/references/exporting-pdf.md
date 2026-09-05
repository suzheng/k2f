# Exporting K2F to PDF

## Overview

PDF is a **drawing of the published lock**, not a second layout engine. `.K2F` stays the source.

Exact API names: [exporting-pdf/surfaces.md](exporting-pdf/surfaces.md). Output contract (extra page, caption, raster ops): [exporting-pdf/pdf-contract.md](exporting-pdf/pdf-contract.md).

## Prerequisites

```bash
pip install k2f    # CLI on PATH + Editor.export_pdf_bytes
```

## When to Use

- Download / attach / email PDF from an existing `.K2F`
- Web app "Download PDF" on a locked document

**When NOT to use**

- Create a document as PDF only → [writing.md](writing.md) then export
- Edit a PDF → edit K2F, re-export

## Default path

```bash
k2f export-pdf file.K2F -o out.pdf
k2f export-pdf file.K2F -o out.pdf --scale 4   # higher-res stamp pages only
python3 scripts/check-pdf.py out.pdf
```

`scripts/check-pdf.py` lives under this skill’s `scripts/`. Run it from the skill directory (or pass an absolute path).

## Which lock is drawn

| Input | What happens |
|-------|----------------|
| Compiled `.K2F` on disk / Viewer / JS `exportPdf(bytes)` | Draws the **existing** lock. No recompile. |
| Unsaved Editor (Python) | Draws the **old** lock. [writing/sdk.md](writing/sdk.md) `save()` first, or re-run `pack_verify.py` on an unpacked dir. |
| In-memory `Document` | `export_pdf_bytes()` / `exportPdf()` compiles then exports. |

JS `Editor` has **no** export. `save()` then `exportPdf(bytes)`.

## Other surfaces (one-liners)

| Surface | Call | Notes |
|---------|------|-------|
| Python | `doc.export_pdf(path)` / `doc.export_pdf_bytes()`; `ed.export_pdf_bytes()` | No module-level `k2f.export_pdf` |
| JS | `exportPdf(bytes)`; `doc.exportPdf()`; `handle.exportPdf()` | Editor: save then `exportPdf(bytes)` |
| Viewer UI | `<k2f-viewer>` Export PDF button / `handle.exportPdf()` | [embedding-viewer.md](embedding-viewer.md) |

## Validation loop

1. Ensure a compiled lock exists (`save` / `compile` if UNLOCKED).
2. Export via CLI or SDK.
3. Run `scripts/check-pdf.py` on the output.
4. If check fails: use the table below. **Never** fall back to html2pdf, jsPDF, browser print, or React-PDF.

## Failure protocol

| Situation | Action |
|-----------|--------|
| `UNLOCKED` | `compile` or `Document.save()` / Editor `save()` |
| `PDF_IS_NOT_A_SOURCE` | User dropped a PDF — open `.K2F` only |
| `INVALID_ARGUMENT` (scale) | Use `--scale 2`, `3`, or `4` only |
| `RASTER_PAINT_OP` | Stamp detector bug — unsupported op leaked to vector path |
| `UNKNOWN_PAINT_OP` | Engine/lock mismatch — do not skip ops |
| No embedded font | Package must include fonts (starter/catalog embed Roboto) |

## Common mistakes

| Mistake | Reality |
|---------|---------|
| Markdown / HTML → PDF library | MD → K2F → export PDF |
| Edit the PDF and expect K2F to sync | One-way export |
| Extra last page is a bug | Integrity verification page — see [pdf-contract.md](exporting-pdf/pdf-contract.md) |
| Export Editor before `save` | Stale lock in the PDF |

## See also

- [writing.md](writing.md) — create or update K2F first
- [exporting-pptx.md](exporting-pptx.md) — PowerPoint of the same lock
- [exporting-docx.md](exporting-docx.md) — Word of the same lock
- [embedding-viewer.md](embedding-viewer.md) — in-browser export button
