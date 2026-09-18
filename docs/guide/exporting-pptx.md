# Export to PowerPoint

PPTX is a **drawing of the published lock**, not a second layout engine. `.K2F` stays the source. Slight text reflow in PowerPoint is expected and allowed.

## From Markdown

```bash
pip install k2f
# From the skill folder:
python scripts/init_package.py --workspace ./out/doc --title "From Markdown" --page a4

k2f markdown slides.md -o ./out/doc/doc.K2F --template ./out/doc/source
k2f export-pptx ./out/doc/doc.K2F -o ./out/doc/doc.pptx
```

Need pixel-identical pages → [PDF](exporting-pdf.md).

## CLI

```bash
k2f export-pptx file.K2F -o out.pptx
k2f export-pptx ./out/doc/doc.K2F -o ./out/doc/doc.pptx
```

No `--scale`. No `--trust-pack`. Draws the published lock; does not compile. Unlocked input fails with `UNLOCKED`.

Native text, tables, and pictures stay editable. Do not stamp a full-slide PNG.

`form_field` nodes export as lock boxes and text — not PowerPoint form controls. Fillable widgets are PDF-only ([Fillable PDF](exporting-pdf.md#fillable-pdf)).

## SDK

| Surface | Call | Notes |
| --- | --- | --- |
| CLI `k2f export-pptx` | `-o` | Existing lock. No recompile. |
| Python `Editor` | `ed.export_pptx_bytes()` | No module-level `k2f.export_pptx`. Save first if you patched the tree. |
| JS `@openk2f/k2f` | `exportPptx(bytes)` after `initWasm`; `ed.exportPptx()` | Editor draws the in-memory package — `save()` for the latest lock. |

Dropping a `.pptx` into K2F fails with `PPTX_IS_NOT_A_SOURCE`.

## Common mistakes

| Mistake | Reality |
| --- | --- |
| Markdown → a PPTX library | MD → K2F → `export-pptx` |
| Stamp a full-slide PNG | Native text/tables/pics must stay editable |
| Expect glyph-identical slides | Slight reflow is legal |
| Edit the PPTX and expect K2F to sync | One-way export |

## See also

- [Export](exporting.md) — all formats
- [PDF](exporting-pdf.md) · [Word](exporting-docx.md) · [InDesign](exporting-idml.md)
- [Markdown conversion](markdown.md)
