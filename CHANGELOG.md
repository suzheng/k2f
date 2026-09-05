# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added

- `k2f export-pptx <package> -o <out.pptx>` — draw the published lock into PowerPoint. Not a second layout engine; slight text reflow is expected. PPTX is not a K2F source (`PPTX_IS_NOT_A_SOURCE`).
- `k2f export-docx <package> -o <out.docx>` — draw the published lock into Word. Not a second layout engine; slight text reflow is expected. DOCX is not a K2F source (`DOCX_IS_NOT_A_SOURCE`).
- Desktop reader: HUD format **DOCX** and `k2f-reader --export-docx out.docx file.K2F` (same lock bytes as GUI Export; conflicts with `--export-pdf` / `--export-pptx`).
- `k2f compile` prints `LAYOUT_SLACK` when a page-height shell is empty at the bottom (warning, exit 0, lock unchanged). Intentional whitespace is allowed; compile/verify/`pack_verify.py` do not fail on this line.
- Grid tracks `{ "auto": true }` — content-sized from measured cells; leftover space goes to `fr`. Table `column_widths` stay `{pt}` / `{fr}` only.

### Changed

- Agent skill: poster/slide page shell is a pinned-height **grid** with `{auto:true}` header/footer and a `{fr:1}` grower ([`ex_poster_shell.json`](skills/k2f/catalog/content/ex_poster_shell.json)); do not stack content inside a fixed-height shell

## [0.1.2] - 2026-08-30

### Added

- **`pip install k2f` ships the CLI** — `k2f` console script on PATH (pack, compile, verify, render, export-pdf, markdown); agents no longer need `cargo install k2f`

### Changed

- Agent skill install guidance: **prefer `pip install k2f`**; `cargo install k2f` is optional for Rust-only environments

## [0.1.1] - 2026-08-30

### Added

- Agent skill **`looks/`** — three example design skins (`quiet-light`, `vivid-blocks`, `night-wash`): complete `theme.json`, composition guides, and shared extension roles (`display`, `kicker`, `caption`, `metric`, `shell`). Agents rewrite a package theme from these examples when the user did not specify a style.
- `Editor.set_running_header` / `set_running_footer` for manifest running blocks

### Changed

- Unified six agent skills into one `skills/k2f/` skill with workflow references (writing, converting-markdown, exporting-pdf, publishing, embedding-viewer)
- Agent skill: merged **generating** and **editing** into a single **writing** workflow (JSON + `k2f unpack` / `pack_verify.py`); optional Python `Editor` moved to `references/writing/sdk.md`
- Public creation path is `Editor.open_template` / `open_dir` / `open_bytes` plus `insert_node` JSON

### Removed

- Public `Composer` / `Document` builder (`add_heading`, `add_body`, `add_warning`, `add_table`)

## [0.1.0] - 2026-08-28

### Added

- K2F format specification v0.1 (ZIP package, semantic tree, theme, compiled lock)
- Reference engine: layout, paint, PDF export, package validation, Ed25519 signing
- CLI (`k2f`) — pack, compile, verify, sign, render, export-pdf, markdown
- Python SDK (`pip install k2f`) — Document, Editor, Markdown bridge
- JavaScript / WASM SDK (`npm i @openk2f/k2f`) — Document, Editor, `<k2f-viewer>` web component
- MCP stdio server (`k2f_mcp`) with tool catalog in `mcp_tools.json`
- Six agent skills under `skills/`
- Public documentation under `docs/` and integrity policy in `SECURITY.md`

[0.1.2]: https://github.com/suzheng/k2f/releases/tag/v0.1.2
[0.1.1]: https://github.com/suzheng/k2f/releases/tag/v0.1.1
[0.1.0]: https://github.com/suzheng/k2f/releases/tag/v0.1.0
