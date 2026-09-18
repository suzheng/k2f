# Architecture

Design rationale, repository map, and engine internals for K2F contributors.

## Reading order

1. **[design.md](design.md)** — why K2F exists, non-negotiable principles, semantic vs presentation separation, v0.1 scope vs roadmap
2. **[codebase.md](codebase.md)** — crate map, compile vs execute, dependencies, where to find code
3. **[layout-engine.md](layout-engine.md)** — two-pass layout, paint plan, fixed-point math, modifier resolution
4. **[editor.md](editor.md)** — future human writing UI *(design proposal; not shipped)*

## Quick links

| Document | Audience | Status |
|----------|----------|--------|
| [design.md](design.md) | Anyone evaluating or extending the format | Shipped principles; some roadmap items |
| [codebase.md](codebase.md) | Contributors navigating the repo | Shipped |
| [layout-engine.md](layout-engine.md) | Engine contributors | Shipped (paged mode) |
| [editor.md](editor.md) | Future UI work | Design proposal only |

Format contract (paths, fields, validation): [k2f-v0.1.md](../spec/k2f-v0.1.md). When this series disagrees with the spec or `schema/*.json`, the spec wins.

Shipped vs roadmap: [status.md](../guide/status.md)
