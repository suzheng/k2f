# Exporting K2F to IDML

IDML is a **drawing of the published lock**, not a second layout engine. `.K2F` stays the source. Slight text reflow in InDesign is expected and allowed. Fonts are listed by family name only — they are **not embedded**.

## When to Use

- Download / attach / email an InDesign package from an existing `.K2F`
- Hand the lock to someone who only opens `.idml`

**When NOT to use**

- Create a document as IDML only → [writing.md](writing.md) then export
- Edit an IDML and expect K2F to sync → one-way export
- Need pixel-identical pages → [exporting-pdf.md](exporting-pdf.md)

## Default path

```bash
k2f export-idml file.K2F -o out.idml
k2f export-idml ./out/doc/doc.K2F -o ./out/doc/doc.idml   # workspace: deliverable next to .K2F, not in source/
```

No `--scale`. No `--trust-pack`. Draws the published lock; does not compile.

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
| Expect fonts to travel with the package | Faces are not embedded; missing fonts substitute |

## See also

- [exporting-pdf.md](exporting-pdf.md) — PDF of the same lock
- [exporting-pptx.md](exporting-pptx.md) — PowerPoint of the same lock
- [exporting-docx.md](exporting-docx.md) — Word of the same lock
- [writing.md](writing.md) — create or update K2F first
