# Exporting K2F to IDML

IDML is a **drawing of the published lock**, not a second layout engine. `.K2F` stays the source. Slight text reflow in InDesign is expected and allowed.

The IDML ZIP itself does not contain font files. Default export writes an InDesign **package folder**: `{stem}.idml` beside `Document Fonts/` so InDesign can open the file without a missing-font dialog. Byte APIs and the web viewer download a `.zip` of that folder. `--idml-only` writes a lone `.idml` (InDesign will missing-font unless the faces are already installed).

## When to Use

- Download / attach / email an InDesign package from an existing `.K2F`
- Hand the lock to someone who only opens `.idml`

**When NOT to use**

- Create a document as IDML only → [writing.md](writing.md) then export
- Edit an IDML and expect K2F to sync → one-way export
- Need pixel-identical pages → [exporting-pdf.md](exporting-pdf.md)

## Default path

```bash
k2f export-idml file.K2F -o ./out/doc/file          # folder: file/file.idml + file/Document Fonts/
k2f export-idml ./out/doc/doc.K2F -o ./out/doc/doc  # workspace: package next to .K2F, not in source/
k2f export-idml file.K2F -o ./out/doc/file.zip      # same tree, zipped
k2f export-idml file.K2F --idml-only -o file.idml   # lone IDML, no fonts
```

`-o foo.idml` without `--idml-only` writes directory `foo/` (strips the suffix). Unzip a `.zip` download, then open the `.idml` next to `Document Fonts`. No `--scale`. No `--trust-pack`. Draws the published lock; does not compile.

## Which lock is drawn

| Input | What happens |
|-------|----------------|
| Compiled `.K2F` on disk | Draws the **existing** lock. No recompile. |
| Unlocked `.K2F` | Fails with `UNLOCKED` |

## Failure protocol

| Situation | Action |
|-----------|--------|
| `UNLOCKED` | `compile` first |
| `IDML_IS_NOT_A_SOURCE` | User dropped an InDesign file — open `.K2F` only |
| `UNKNOWN_PAINT_OP` | Engine/lock mismatch — do not skip ops |

## Common mistakes

| Mistake | Reality |
|---------|---------|
| Restyle with InDesign paragraph styles | Absolute lock coordinates would reflow |
| Stamp a full-page PNG | Native text/tables/pics must stay editable |
| Expect glyph-identical pages | Slight reflow is legal |
| Open a lone `.idml` without `Document Fonts` | Use the default package folder (or unzip the download) |
| Rename the package zip to `.idml` | The wrapper zip is not IDML; unzip first |

## See also

- [exporting-pdf.md](exporting-pdf.md) — PDF of the same lock
- [exporting-pptx.md](exporting-pptx.md) — PowerPoint of the same lock
- [exporting-docx.md](exporting-docx.md) — Word of the same lock
- [writing.md](writing.md) — create or update K2F first
