# Exporting K2F to DOCX

DOCX is a **drawing of the published lock**, not a flowing “Save as Word” layout engine. `.K2F` stays the source. Slight text reflow in Word is expected and allowed.

## When to Use

- Download / attach / email a Word file from an existing `.K2F`
- Hand the lock to someone who only opens `.docx`

**When NOT to use**

- Create a document as DOCX only → [writing.md](writing.md) then export
- Edit a DOCX and expect K2F to sync → one-way export
- Need pixel-identical pages → [exporting-pdf.md](exporting-pdf.md)

## Default path

```bash
k2f export-docx file.K2F -o out.docx
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
| `DOCX_IS_NOT_A_SOURCE` | User dropped a Word file — open `.K2F` only |
| `UNKNOWN_PAINT_OP` | Engine/lock mismatch — do not skip ops |

## Common mistakes

| Mistake | Reality |
|---------|---------|
| Restyle with Word Heading 1 | Absolute lock coordinates would reflow |
| Stamp a full-page PNG | Native text/tables/pics must stay editable |
| Expect glyph-identical pages | Slight reflow is legal |

## See also

- [exporting-pdf.md](exporting-pdf.md) — PDF of the same lock
- [exporting-pptx.md](exporting-pptx.md) — PowerPoint of the same lock
- [writing.md](writing.md) — create or update K2F first
