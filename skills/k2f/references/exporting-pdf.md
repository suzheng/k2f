# Exporting K2F to PDF

PDF is a **drawing of the published lock**, not a second layout engine. `.K2F` stays the source.

If the PDF looks wrong or you are checking bytes: [exporting-pdf/pdf-contract.md](exporting-pdf/pdf-contract.md).

## When to Use

- Download / attach / email PDF from an existing `.K2F`
- Web app "Download PDF" on a locked document

**When NOT to use**

- Create a document as PDF only → [writing.md](writing.md) then export
- Edit a PDF → edit K2F, re-export

## Default path

```bash
k2f export-pdf file.K2F -o out.pdf
k2f export-pdf ./out/doc/doc.K2F -o ./out/doc/doc.pdf   # workspace: deliverable next to .K2F, not in source/
k2f export-pdf file.K2F -o out.pdf --scale 4   # higher-res stamp pages only
```

`--scale` is `2` (default), `3`, or `4` — stamp pages only. Default export has the **same page count as the lock** (no integrity page, no source caption). `--trust-pack` adds those; then run `python3 scripts/check-pdf.py out.pdf` (script lives under this skill’s `scripts/`).

**Never** fall back to html2pdf, jsPDF, browser print, or React-PDF.

## Which lock is drawn

| Input | What happens |
|-------|----------------|
| Compiled `.K2F` on disk / Viewer / JS `exportPdf(bytes)` | Draws the **existing** lock. No recompile. |
| Unsaved Editor (Python) | Draws the **old** lock. [writing/sdk.md](writing/sdk.md) `save()` first, or re-run `pack_verify.py` on an unpacked dir. |
| In-memory `Document` | `export_pdf_bytes()` / `exportPdf()` compiles then exports. |

JS `Editor` has **no** export. `save()` then `exportPdf(bytes)`.

## Other surfaces

| Surface | Call | Notes |
|---------|------|-------|
| Python | `doc.export_pdf(path)` / `doc.export_pdf_bytes()`; `ed.export_pdf_bytes()` | No module-level `k2f.export_pdf`. Editor: **old** lock until `save_bytes()` |
| JS | `exportPdf(bytes)`; `doc.exportPdf()`; `handle.exportPdf()` | Editor: save then `exportPdf(bytes)` |
| Viewer UI | `<k2f-viewer>` Export PDF button / `handle.exportPdf()` | [embedding-viewer.md](embedding-viewer.md) |

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
| Extra last page on a default export | Bug, or you passed `--trust-pack` — see [pdf-contract.md](exporting-pdf/pdf-contract.md) |
| Export Editor before `save` | Stale lock in the PDF |

## See also

- [writing.md](writing.md) — create or update K2F first
- [exporting-pptx.md](exporting-pptx.md) — PowerPoint of the same lock
- [exporting-docx.md](exporting-docx.md) — Word of the same lock
- [embedding-viewer.md](embedding-viewer.md) — in-browser export button
