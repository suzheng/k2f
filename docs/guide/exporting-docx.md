# Export to Word

DOCX is a **drawing of the published lock**, not a flowing “Save as Word” layout engine. `.K2F` stays the source. Slight text reflow in Word is expected and allowed.

## From Markdown

```bash
pip install k2f
# From the skill folder:
python scripts/init_package.py --workspace ./out/doc --title "From Markdown" --page a4

k2f markdown notes.md -o ./out/doc/doc.K2F --template ./out/doc/source
k2f export-docx ./out/doc/doc.K2F -o ./out/doc/doc.docx
```

Need pixel-identical pages → [PDF](exporting-pdf.md).

## CLI

```bash
k2f export-docx file.K2F -o out.docx
k2f export-docx ./out/doc/doc.K2F -o ./out/doc/doc.docx
```

No `--scale`. No `--trust-pack`. Draws the published lock; does not compile. Unlocked input fails with `UNLOCKED`.

Native text, tables, and pictures stay editable. Do not stamp a full-page PNG. Do not restyle with Word Heading 1 — absolute lock coordinates would reflow.

`form_field` nodes export as lock boxes and text — not Word content controls. Fillable widgets are PDF-only ([Fillable PDF](exporting-pdf.md#fillable-pdf)).

## SDK

| Surface | Call | Notes |
| --- | --- | --- |
| CLI `k2f export-docx` | `-o` | Existing lock. No recompile. |
| Python `Editor` | `ed.export_docx_bytes()` | No module-level `k2f.export_docx`. Save first if you patched the tree. |
| JS `@openk2f/k2f` | `exportDocx(bytes)` after `initWasm`; `ed.exportDocx()` | Editor draws the in-memory package — `save()` for the latest lock. |

Dropping a Word file into K2F fails with `DOCX_IS_NOT_A_SOURCE`.

## Common mistakes

| Mistake | Reality |
| --- | --- |
| Markdown → a DOCX library | MD → K2F → `export-docx` |
| Restyle with Word Heading 1 | Absolute lock coordinates would reflow |
| Stamp a full-page PNG | Native text/tables/pics must stay editable |
| Expect glyph-identical pages | Slight reflow is legal |
| Edit the DOCX and expect K2F to sync | One-way export |

## See also

- [Export](exporting.md) — all formats
- [PDF](exporting-pdf.md) · [PowerPoint](exporting-pptx.md) · [InDesign](exporting-idml.md)
- [Markdown conversion](markdown.md)
