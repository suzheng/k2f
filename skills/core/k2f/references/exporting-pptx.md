# Exporting K2F to PPTX

PPTX is a **drawing of the published lock**, not a second layout engine. `.K2F` stays the source. Slight text reflow in PowerPoint is expected and allowed.

## When to Use

- Download / attach / email a PowerPoint from an existing `.K2F`
- Hand the lock to someone who only opens `.pptx`

**When NOT to use**

- Create a document as PPTX only → [writing.md](writing.md) then export
- Edit a PPTX and expect K2F to sync → one-way export
- Need pixel-identical pages → [exporting-pdf.md](exporting-pdf.md)

## Default path

```bash
k2f export-pptx file.K2F -o out.pptx
k2f export-pptx ./out/doc/doc.K2F -o ./out/doc/doc.pptx   # workspace: deliverable next to .K2F, not in source/
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
| `PPTX_IS_NOT_A_SOURCE` | User dropped a PPTX — open `.K2F` only |
| `UNKNOWN_PAINT_OP` | Engine/lock mismatch — do not skip ops |

## Common mistakes

| Mistake | Reality |
|---------|---------|
| Stamp a full-slide PNG | Native text/tables/pics must stay editable |
| Expect glyph-identical slides | Slight reflow is legal |

## See also

- [exporting-pdf.md](exporting-pdf.md) — PDF of the same lock
- [exporting-docx.md](exporting-docx.md) — Word of the same lock
- [writing.md](writing.md) — create or update K2F first
