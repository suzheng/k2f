# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added

- Agent skill **design-first workflow** — write a design specification Markdown before authoring K2F JSON; implement `theme.json` + content from the spec, then render and iterate until the output matches.
- `k2f export-pptx <package> -o <out.pptx>` — draw the published lock into PowerPoint. Not a second layout engine; slight text reflow is expected. PPTX is not a K2F source (`PPTX_IS_NOT_A_SOURCE`).
- `k2f export-docx <package> -o <out.docx>` — draw the published lock into Word. Not a second layout engine; slight text reflow is expected. DOCX is not a K2F source (`DOCX_IS_NOT_A_SOURCE`).
- Desktop reader: HUD format **DOCX** and `k2f-reader --export-docx out.docx file.K2F` (same lock bytes as GUI Export; conflicts with `--export-pdf` / `--export-pptx`).
- `k2f compile` prints `LAYOUT_SLACK` when a page-height shell is empty at the bottom (warning, exit 0, lock unchanged). Intentional whitespace is allowed; compile/verify/`pack_verify.py` do not fail on this line.
- Grid tracks `{ "auto": true }` — content-sized from measured cells; leftover space goes to `fr`. Table `column_widths` stay `{pt}` / `{fr}` only.

### Changed

- Agent skill: poster/slide page shell is a pinned-height **grid** with `{auto:true}` header/footer and a `{fr:1}` grower ([`ex_poster_shell.json`](skills/k2f/catalog/content/ex_poster_shell.json)); do not stack content inside a fixed-height shell
- `export-docx` / `export-pptx`: pin Office theme `dk1`/`lt1` to RGB black/white instead of `sysClr windowText`/`window`. Text that is lock-black / lock-white is stored as `000001` / `FFFFFE` so Word/PowerPoint Dark Mode cannot treat it as Automatic and invert it on a still-white page. Word export now includes `word/theme/theme1.xml`, explicit `w:background`, `w14:textFill`, RGB `wps:style` fontRef, and an opaque text-box underlay matching the shape behind the text so unfilled boxes are not remapped in Dark Mode.
- `export-docx` / `export-pptx`: resolve lock font keys (package path stem such as `Roboto-Regular`, plus `default`) to the embedded TTF family name instead of leaking the alias into `typeface` / `w:rFonts`. Character tracking is taken from lock glyph extra-advance (Word `w:spacing`, DrawingML `a:rPr spc`). Role `padding_pt.top` on a text node becomes text-box `tIns` from the first-line glyph `y_offset` (skipped when the box is vertically centered).
- `export-docx` / `export-pptx`: disable text-frame wrap when the lock is a single line that already fills the box, **or when the frame is only tall enough for one line**. Host bold/metrics that are wider than rustybuzz were wrapping the last word onto a clipped second line (title rows, padded pills). Multi-line lock boxes and tall frames with leftover width still wrap. One-line frames pin exact line spacing to the lock font (hosts otherwise use ~12pt pitch and hide 7–9pt text) and set DrawingML `horzOverflow`/`vertOverflow` to `overflow`. Symmetric glyph padding is treated as center (not left+`lIns`) so host-wider labels keep slack on both sides. PPTX copies lock glyph left/right gaps into `lIns`/`rIns` for true left/right frames. Word folds a same-node DrawBox into the text box (corner radius + fill) so LibreOffice cannot paint the empty pill on top of the label.
- `export-docx` / `export-pptx`: emit Office hyperlinks only when a `link` modifier intent is a URI. Theme keys such as `default` stay underline-from-lock and are not `hlinkClick`/`w:hyperlink` targets. Hosts that still wrap real URLs ignore run fill and paint theme `hlink` (blue); pin that slot, Word's Hyperlink style, and underline color to the lock run RGB.
- `export-docx`: LibreOffice Writer paints pictures above DrawingML shapes regardless of `behindDoc`, so a later opaque container hid stamps and address text. Large fill boxes now use `behindDoc=1`; 1 pt rules stay in front; images fully covered by a later opaque lock box are omitted (they are invisible in the lock). Landscape `w:pgSz` sets `w:orient="landscape"`.

### Removed

- Agent skill **`looks/`** — removed bundled design skins (`quiet-light`, `vivid-blocks`, `night-wash`) and `verify_looks.py`; styling is authored from the design spec instead

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
