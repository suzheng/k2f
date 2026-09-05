# Exporting K2F to PPTX

## Overview

PPTX is a **drawing of the published lock**, not a second layout engine. `.K2F` stays the source. Slight text reflow in PowerPoint is expected and allowed.

Exact CLI names: [exporting-pptx/surfaces.md](exporting-pptx/surfaces.md). Capability table: crate README (`k2f_pptx`).

## Prerequisites

```bash
pip install k2f    # CLI on PATH
```

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
| Treat PPTX as a second source | One-way dump of the lock |
| Expect glyph-identical slides | Slight reflow is legal |
| Stamp a full-slide PNG | Native text/tables/pics must stay editable |

## See also

- [exporting-pdf.md](exporting-pdf.md) — PDF of the same lock
- [exporting-docx.md](exporting-docx.md) — Word of the same lock
- [writing.md](writing.md) — create or update K2F first
