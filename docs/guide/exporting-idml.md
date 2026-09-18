# Export to InDesign

IDML is a **drawing of the published lock**, not a second layout engine. `.K2F` stays the source. Slight text reflow in InDesign is expected and allowed.

The IDML ZIP itself does not contain font files. Default export writes an InDesign **package folder**: `{stem}.idml` beside `Document Fonts/` so InDesign can open the file without a missing-font dialog. Byte APIs download a `.zip` of that folder. `--idml-only` writes a lone `.idml` (InDesign will missing-font unless the faces are already installed).

## From Markdown

```bash
pip install k2f
# From the skill folder:
python scripts/init_package.py --workspace ./out/doc --title "From Markdown" --page a4

k2f markdown notes.md -o ./out/doc/doc.K2F --template ./out/doc/source
k2f export-idml ./out/doc/doc.K2F -o ./out/doc/doc
```

Need pixel-identical pages → [PDF](exporting-pdf.md).

## CLI

```bash
k2f export-idml file.K2F -o ./out/doc/file          # folder: file/file.idml + file/Document Fonts/
k2f export-idml ./out/doc/doc.K2F -o ./out/doc/doc  # workspace: package next to .K2F, not in source/
k2f export-idml file.K2F -o ./out/doc/file.zip      # same tree, zipped
k2f export-idml file.K2F --idml-only -o file.idml   # lone IDML, no fonts
```

`-o foo.idml` without `--idml-only` writes directory `foo/` (strips the suffix). Unzip a `.zip` download, then open the `.idml` next to `Document Fonts`. No `--scale`. No `--trust-pack`. Draws the published lock; does not compile. Unlocked input fails with `UNLOCKED`.

Native text, tables, and pictures stay editable. Do not stamp a full-page PNG. Do not restyle with InDesign paragraph styles — absolute lock coordinates would reflow.

## SDK

| Surface | Call | Notes |
| --- | --- | --- |
| CLI `k2f export-idml` | `-o` directory, `.zip`, or `--idml-only` | Existing lock. No recompile. |
| Python `Editor` | `ed.export_idml_bytes()` (package zip) / `ed.export_idml_only_bytes()` | No module-level `k2f.export_idml`. Save first if you patched the tree. |
| JS `@openk2f/k2f` | `exportIdml(bytes)` (zip) / `exportIdmlOnly(bytes)`; Editor `exportIdml()` / `exportIdmlOnly()` | Editor draws the in-memory package — `save()` for the latest lock. |

Dropping an InDesign file into K2F fails with `IDML_IS_NOT_A_SOURCE`.

## Common mistakes

| Mistake | Reality |
| --- | --- |
| Markdown → an IDML library | MD → K2F → `export-idml` |
| Restyle with InDesign paragraph styles | Absolute lock coordinates would reflow |
| Stamp a full-page PNG | Native text/tables/pics must stay editable |
| Expect glyph-identical pages | Slight reflow is legal |
| Open a lone `.idml` without `Document Fonts` | Use the default package folder (or unzip the download) |
| Rename the package zip to `.idml` | The wrapper zip is not IDML; unzip first |

## See also

- [Export](exporting.md) — all formats
- [PDF](exporting-pdf.md) · [PowerPoint](exporting-pptx.md) · [Word](exporting-docx.md)
- [Markdown conversion](markdown.md)
- [Theme and fonts](../authoring/theme.md)
