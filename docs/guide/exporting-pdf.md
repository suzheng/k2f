# Export to PDF

PDF is a **drawing of the published lock**, not a second layout engine. `.K2F` stays the source.

Default export writes **AcroForm widgets** for `form_field` nodes (fillable PDF). `--flatten` paints lock glyphs instead and omits widgets.

## From Markdown

```bash
pip install k2f
# From the skill folder:
python scripts/init_package.py --workspace ./out/doc --title "From Markdown" --page a4

k2f markdown notes.md -o ./out/doc/doc.K2F --template ./out/doc/source
k2f export-pdf ./out/doc/doc.K2F -o ./out/doc/doc.pdf
```

That PDF matches the lock pages. Underscores or `[____]` in Markdown are **not** fields — see [Fillable PDF](#fillable-pdf).

## CLI

```bash
k2f export-pdf file.K2F -o out.pdf
k2f export-pdf ./out/doc/doc.K2F -o ./out/doc/doc.pdf
k2f export-pdf file.K2F -o out.pdf --flatten          # glyphs, no AcroForm
k2f export-pdf file.K2F -o out.pdf --scale 4          # stamp pages only (default 4)
```

`--scale` is `2`, `3`, or `4` and applies only to stamp pages (blur, shadow, or gradient). Default export has the **same page count as the lock** — that is the customer PDF. `--trust-pack` adds captions and a verify page; do not use it for customer delivery. `--trust-pack` with fillable fields fails (`FILLABLE_EXCLUSIVE`).

Do not fall back to html2pdf, jsPDF, browser print, or React-PDF.

## Fillable PDF

Markdown → K2F does **not** parse `form_field` comments or treat underscore runs as blanks. After import (or when authoring JSON):

1. Unpack if you only have the packed file: `k2f unpack ./out/doc/doc.K2F -o ./out/doc/source` (skip `--include-lock`).
2. Copy nodes from [`ex_form.json`](../../skills/k2f/catalog/content/ex_form.json) into `content/root.json` `children`. Role and `content.type` must both be `form_field`. Never draw `____` or `□` in body text.
3. Pack: `python scripts/pack_verify.py ./out/doc/source -o ./out/doc/doc.K2F`
4. Export: `k2f export-pdf ./out/doc/doc.K2F -o ./out/doc/doc.pdf`

Default PDF: AcroForm widgets (field `DrawText` skipped, `DrawBox` kept). `--flatten`: paint glyphs, no widgets. Field shape: [Allowed keys](../reference/keys.md). Viewers overlay native inputs on the same boxes ([Web viewer](web-viewer.md)).

Word and PowerPoint export those boxes as lock drawings — not Word content controls or PPT form widgets.

## SDK

| Surface | Call | Notes |
| --- | --- | --- |
| CLI `k2f export-pdf` | `-o`, optional `--flatten`, `--scale`, `--trust-pack` | Draws the existing lock. No recompile. |
| Python `Editor` | `ed.export_pdf_bytes()` | No module-level `k2f.export_pdf`. Save first if you patched the tree. |
| JS `@openk2f/k2f` | `exportPdf(bytes, scale?, flatten?)` after `initWasm` | Editor has **no** PDF export — `save()` then `exportPdf(bytes)`. |
| Viewer | Export PDF / `handle.exportPdf()` | [Web viewer](web-viewer.md) |

Unlocked input fails with `UNLOCKED`. Byte-level stamp vs vector contract: Skill [pdf-contract.md](../../skills/k2f/references/exporting-pdf/pdf-contract.md).

## Common mistakes

| Mistake | Reality |
| --- | --- |
| Markdown / HTML → a PDF library | MD → K2F → `export-pdf` |
| `____` in Markdown as a fill-in blank | Copy `ex_form.json`, pack, then export |
| Edit the PDF and expect K2F to sync | One-way export (`PDF_IS_NOT_A_SOURCE`) |
| `--trust-pack` for a customer or fillable PDF | Extra verify page; fillable documents error `FILLABLE_EXCLUSIVE` |
| Extra last page on a default export | Bug, or you passed `--trust-pack` |

## See also

- [Export](exporting.md) — all formats
- [PowerPoint](exporting-pptx.md) · [Word](exporting-docx.md) · [InDesign](exporting-idml.md)
- [Markdown conversion](markdown.md)
- [Catalog](../reference/catalog.md) — `ex_form.json`
