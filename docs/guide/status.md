# Project status

## Shipped in v0.1

- **Package format** — ZIP container, semantic tree, theme, embedded fonts, compiled lock
- **Verify & integrity** — hash chain, banners (`SIGNED`, `UNSIGNED`, `BROKEN_INTEGRITY`, …)
- **Web viewer** — paints lock only; no `fillText` body text
- **Registry packages** — `pip install k2f` (PyPI), `npm i @openk2f/k2f` (npm), `cargo install k2f` (crates.io); v0.1.0 on all three (npm uses scoped name `@openk2f/k2f` because unscoped `k2f` is blocked).
- **Agent layers** — skills (repo instructions) → SDK (implementation) → MCP stdio adapter ([`engine/k2f_mcp`](../../engine/k2f_mcp/README.md))
- **Markdown bridge** — `markdown_to_k2f` / `k2f_to_markdown` (display `$$...$$` nodes; inline `$...$` modifiers)
- **Native math** — display `$$` nodes and inline `$` modifiers compiled to glyphs + fraction rules (TeX subset including stretchy `\left\right` and `matrix`/`align`/`cases`; unknown commands fail).
- **Agent skills** — one skill under [`skills/k2f/`](../../skills/k2f/SKILL.md) with workflow references
- **MCP stdio server** — local `k2f_mcp`; tool contract [`mcp_tools.json`](../../engine/k2f_mcp/mcp_tools.json); run notes [`README`](../../engine/k2f_mcp/README.md). **Remote HTTP MCP not shipped**

## In development

- **[`desktop/k2f_reader`](../../desktop/k2f_reader/README.md)** — native lock executor (same rules as the web viewer: opening does not recompile; pixels and PDF come from `document.K2F.lock`). Packaged builds are on the website [`/download`](https://k2f.dev/download). Build from source with `cargo run -p k2f_reader -- path/to/file.K2F`. CI runs `cargo test -p k2f_reader`.

## Roadmap (not shipped)

| Item | Notes |
|------|-------|
| Remote HTTP MCP | OAuth, multi-tenant; stdio is shipped |
| Slide / infinite canvas | Product surface TBD |
| Desktop file association / installers | OS registration and packaged installers |

Internal implementation tracks live outside this public repository. They are not part of the public contract.

## Versioning

Format spec [v0.1](../spec/k2f-v0.1.md) tracks engine crate `0.1.0`. See [COMPATIBILITY.md](../../COMPATIBILITY.md) for engine matching policy.
