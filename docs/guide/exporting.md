# Export

Turn a packed `.K2F` into PDF, PowerPoint, Word, or InDesign. Markdown and JSON are inputs to K2F; the exports are **drawings of the published lock**, not a second layout engine. `.K2F` stays the source.

```bash
pip install k2f
# From the skill folder (starter + scripts):
python scripts/init_package.py --workspace ./out/doc --title "From Markdown" --page a4

k2f markdown notes.md -o ./out/doc/doc.K2F --template ./out/doc/source
k2f export-pdf  ./out/doc/doc.K2F -o ./out/doc/doc.pdf
k2f export-pptx ./out/doc/doc.K2F -o ./out/doc/doc.pptx
k2f export-docx ./out/doc/doc.K2F -o ./out/doc/doc.docx
k2f export-idml ./out/doc/doc.K2F -o ./out/doc/doc
```

`--template` is an unpacked author directory ([Markdown conversion](markdown.md)). JSON authoring uses the same packed file: [Getting started](getting-started.md) (`pack_verify.py`), then the same `k2f export-*` commands.

## What you get

| Format | Guide | Pixel-identical | Fillable | Native text |
| --- | --- | --- | --- | --- |
| PDF | [PDF](exporting-pdf.md) | Yes | AcroForm widgets by default | Selectable; not a Word file |
| PowerPoint | [PowerPoint](exporting-pptx.md) | Slight reflow allowed | No | Editable shapes |
| Word | [Word](exporting-docx.md) | Slight reflow allowed | No content controls | Editable text boxes |
| InDesign | [InDesign](exporting-idml.md) | Slight reflow allowed | No | Editable; package includes `Document Fonts/` |

Do not stamp a full-page PNG into PPTX, DOCX, or IDML. Do not treat those files as a second source (`PPTX_IS_NOT_A_SOURCE`, `DOCX_IS_NOT_A_SOURCE`, `IDML_IS_NOT_A_SOURCE`, `PDF_IS_NOT_A_SOURCE`).

Markdown import does **not** create fillable fields. For a fillable PDF, add `form_field` nodes after import — [PDF](exporting-pdf.md#fillable-pdf).

Unlocked packages fail with `UNLOCKED`. Compile or `pack_verify.py` first.

## See also

- [Markdown conversion](markdown.md) — Markdown ↔ K2F
- [Getting started](getting-started.md) — author shell and pack loop
- [Web viewer](web-viewer.md) — in-browser export
- [K2F Skill](../../skills/k2f/SKILL.md) — agent workflows
