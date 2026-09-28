# Changelog

Release history for the K2F reference engine, `k2f` CLI, Python and JavaScript SDKs, MCP server, desktop reader, and export tools.

- **Package versions** follow [Semantic Versioning](https://semver.org/). Tagged builds and artifacts: [GitHub Releases](https://github.com/suzheng/k2f/releases).
- **Format contract** is [Spec 0.3](docs/spec/k2f-v0.3.md). Patch releases usually keep that filename; see [Compatibility](COMPATIBILITY.md) for how `engine_version` on a compiled lock relates to the spec.
- **Install:** `pip install k2f` · `cargo install k2f` · `npm i @openk2f/k2f`

Entries follow [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

## [0.3.1] - 2026-09-19

### Fixed

- **`export-docx` / `export-pptx`:** Fidelity fixes when drawing the published lock into Word, LibreOffice Writer, PowerPoint, and LibreOffice Impress — lists (bullets, numbering, wrap indent), folded card shells and z-order, table cell padding and rules, font embedding (including CJK and bold-only faces), gradients and transparency, text wrap and alignment, form checkboxes, and line spacing. Office files remain one-way drawings of the lock (`DOCX_IS_NOT_A_SOURCE`, `PPTX_IS_NOT_A_SOURCE`).
- **`export-idml`:** Fidelity fixes for InDesign — native tables vs DrawBox fallbacks, typography (faux-bold, multi-line titles beside figures), linear gradients and translucent fills (process RGB / `WebIntent`), and related layout edge cases. IDML is not a K2F source (`IDML_IS_NOT_A_SOURCE`).

## [0.3.0] - 2026-09-18

### Added

- Format specification **[0.3](docs/spec/k2f-v0.3.md)** — current on-disk ZIP layout, schemas, and lock contract (aligned with the 0.3.x SDK line).
- Pack/compile/raster reject SVG live text with `SVG_TEXT` (not `IMAGE_SIZE`). Still fail-closed; put labels in a K2F text node or convert to `<path>`.
- List markers and code blocks use the same package-font coverage fallback as body text.
- Skill catalog: OFL `NotoSerif-Regular.ttf` for serif headings or body (`--add-font catalog/assets/fonts/NotoSerif-Regular.ttf`, then retarget role `font_family`). Starter stays Roboto-only; no cross-family fallback.
- `k2f export-idml <package> -o <out-dir>` — InDesign package (`{stem}.idml` + `Document Fonts/`). `-o file.zip` writes the same tree zipped; `--idml-only` writes a lone `.idml`. The IDML ZIP still does not contain TTF bytes. IDML is not a K2F source (`IDML_IS_NOT_A_SOURCE`).
- Role/variant `image_fit` (`contain` default, or `cover` center-crop). An image node's own `corner_radius` clips that bitmap. PPTX/DOCX emit `a:srcRect` + `roundRect`; IDML uses FillProportionally + frame corners.

### Changed

- Root `layout` of `grid` / `overlay` / `columns` / horizontal `stack` is a compile error (was a silent vertical-flow fallback for horizontal). Nest under a child. Catalog `ex_*.json` are children — do not replace `content/root.json`.

### Fixed

- `export-docx` / `export-pptx`: lock paint already applied theme `emphasis` (including `emphasis` → italic) is not remapped to bold. Overlaying every non-`italic` intent as bold made true-italic runs export as bold-italic. Empty-paint fallback still bolds default emphasis.
- `list_item` with a partial `list_style` (for example only `bullet_glyph`) compiles by merging starter defaults. Explicit `0` still wins; schema fields stay optional.

## [0.2.4] - 2026-09-11

### Added

- Desktop reader: continuous pinch / Ctrl+wheel zoom with display LOD, copy-all-as-text, and macOS menu checkmarks.
- Web viewer: display-scale LOD helpers aligned with the desktop reader.

### Fixed

- **`export-docx` / `export-pptx`:** Many fidelity fixes when drawing the published lock into Word, LibreOffice Writer, PowerPoint, and LibreOffice Impress — hero titles on banners, card folding and stacking, list bullets and numbering, table rows and partial-edge rules, dark-mode paper washes, font embedding and tracking, wrap and alignment, gradients and translucency, running headers with `{{page_*}}` fields, form checkboxes, and z-order against images. Exports remain one-way drawings of the lock.

## [0.2.3] - 2026-09-09

### Fixed

- Align published Rust (`k2f`, `k2f_sdk`), Python (`k2f`), and npm (`@openk2f/k2f`) with the template-bundling removal shipped on `main` after 0.2.2 (crates.io and PyPI 0.2.2 still exposed named template APIs).

## [0.2.2] - 2026-09-09

### Added

- `init_package.py` page presets `square` (1:1, 600pt), `portrait-45` (4:5), and `card` (US 3.5×2", 252×144pt), default margin 0. Same paged `ex_poster_shell` as slides; set `layout.height` to the page height. Duplex cards: two shells, back `break_before: page`, `--expect-pages 2`.
- `PAGE_UNDERFILL` compile diagnostic when a page content box is ≥25% empty at the bottom (not the last page of a multi-page flow). Warning only — `pack_verify.py` does not fail. Catalog [`ex_filled_page.json`](skills/k2f/catalog/content/ex_filled_page.json). `--render` also writes `preview-N.png` for extra pages.

### Changed

- `k2f pack`, `k2f compile`, and `Editor.save_bytes` coverage-subset large CJK faces in the package to GB2312 ∪ Big5 level 1 ∪ JIS X 0208 Han, plus all non-Han glyphs. Author directories keep the original face. Faces that would not drop any Han stay byte-identical. Missing Han still fails closed (`FONT_MISSING_GLYPH`).
- Agent skill: invoice/CV/flyer/poster are composed filled pages (`PAGE_UNDERFILL` = must-fix there). Contract/report/thesis is one flow tree — no `p1`/`p2` page containers; `break_before` on chapter/annex/signature is allowed. `LAYOUT_SLACK` stays warning-only.
- Theme roles other than `default` may omit `font_family` / `font_size` / `line_height_mult` / `color`; compile fills them from `default`. `default` still requires all four. Sheet fill remains the root role `box_decoration.background` (full page, including margins) — not `page_config.background`.
- `LAYOUT_SLACK` / `PAGE_UNDERFILL` skip miniature pages (content box shorter than 180pt) and unused gaps under 36pt, so card inset is not treated as a hollow grower. A4 / 16:9 growers are unchanged.
- `FONT_MISSING_GLYPH` names the uncovered code points and tells the author to `--add-font` a covering TTF/OTF (still fail-closed; no OS fallback). Package-embedded faces already fall back to each other.

### Fixed

- Grid `{auto:true}` tracks that would overflow the available axis now shrink in proportion instead of overlapping sibling columns/rows. Short auto content still hugs; leftover still goes to `fr`. Same rule on both axes.
- Root `layout` of `grid` / `overlay` / `columns` is a compile error (was a silent vertical-flow fallback). Nest under a child, e.g. `root.grid`.
- Linear gradient paint fails closed on unparseable stop colors (no silent one-stop / empty fill). `angle_degrees` is 0=right, 90=down — documented on the primitive schema and in the skill fills note (not CSS).
- SVG `<text>` rejection message tells agents to drop the tags and use a K2F text node (or `<path>`), matching the skill error table.
- Four-edge box borders follow `corner_radius` in PNG/viewer (and dashed/dotted PDF), matching fill/shadow. Partial-edge borders stay straight.
- **`export-docx` / `export-pptx`:** Office theme colors pinned to lock sRGB; Word avoids native `w:tbl` (Dark Mode inverts `w:shd`); table cells drawn as the same boxes + text as the rest of the page. **`export-docx`:** gradients, glass, and overlapping images use native DrawingML / `behindDoc` stacking so LibreOffice Writer does not hide labels behind full-page rasters.

### Removed

- Named official templates from every published SDK (`official_templates`, `copy_template`, `resolve_template`, `Editor.open_template` / `openTemplate`). WASM no longer `include_dir`s `k2f/templates/`. Create with `init_package.py` / unpack / Gallery, then `Editor.open_dir` or `Editor.open` / `open_bytes`. CLI `k2f markdown --template` is a required author directory. JS `markdownToK2f` takes `templateBytes`.

## [0.2.1] - 2026-09-07

### Fixed

- npm `@openk2f/k2f` 0.2.0 shipped the viewer-only WASM as the SDK binary (identical `wasm/` and `wasm-viewer/` files), so `markdown_to_k2f` and `Editor` were missing. SDK and viewer-only builds now use isolated cargo target dirs.

## [0.2.0] - 2026-09-07

### Added

- Agent skill **design-first workflow** — write a design specification Markdown before authoring K2F JSON; implement `theme.json` + content from the spec, then render and iterate until the output matches.
- `k2f export-pptx <package> -o <out.pptx>` — draw the published lock into PowerPoint. Not a second layout engine; slight text reflow is expected. PPTX is not a K2F source (`PPTX_IS_NOT_A_SOURCE`).
- `k2f export-docx <package> -o <out.docx>` — draw the published lock into Word. Not a second layout engine; slight text reflow is expected. DOCX is not a K2F source (`DOCX_IS_NOT_A_SOURCE`).
- Desktop reader: HUD format **DOCX** and `k2f-reader --export-docx out.docx file.K2F` (same lock bytes as GUI Export; conflicts with `--export-pdf` / `--export-pptx`).
- `k2f compile` prints `LAYOUT_SLACK` when a large stretched box is empty at the bottom — usually a `{fr:1}` grower packed with an auto-height stack, not the page shell (warning, exit 0, lock unchanged).
- Grid tracks `{ "auto": true }` — content-sized from measured cells; leftover space goes to `fr`. Table `column_widths` stay `{pt}` / `{fr}` only.
- Optional grid `rows`: omit → `{auto:true}` tracks `ceil(n_children / n_columns)`. Declared rows do not grow. `fr` rows still need a finite outer height.
- Catalog [`ex_glass.json`](skills/k2f/catalog/content/ex_glass.json) — `card` variant `glass` via named `box_decoration.blur` and a translucent surface (not a radial gradient).
- Line wrap: hyphen-minus and U+00AD are break opportunities (hyphen stays at the line end). A word still wider than the line is character-split. No hyphenation dictionary, no auto-shrink.

### Changed

- Font load/render parse only `.ttf`/`.otf`. License/sidecar files under `assets/fonts/` stay in the ZIP and `Package.fonts` (appearance hash unchanged) but are skipped at `Face::parse`. Invalid faces report `FONT_INVALID` with the path, not `UnknownMagic`. Auto-`default` and “at least one font” count faces only. PDF/DOCX/PPTX export apply the same face-path filter (do not embed or resolve license `.txt` as a font).
- SVG `<text>` / `<tspan>` / `<textPath>` / `<foreignObject>` detection strips XML comments and CDATA first, then matches real elements. Pack and compile reject them (`IMAGE_SIZE`); paint still fail-closed. Agent skill writing Rule 4 states the same at the writing-loop gate.
- Agent skill: flowing articles use **one** unpadded `columns` container with `column_span: all` (do not `break_before: page` on figures). Variant text fields stay under `text_overrides`. Compact tables: widen columns, lower role `font_size`, or insert U+00AD.
- Agent skill: poster/slide page shell is a pinned-height **grid** with `{auto:true}` header/footer and a `{fr:1}` grower ([`ex_poster_shell.json`](skills/k2f/catalog/content/ex_poster_shell.json)); fill the grower with nested `{fr:1}` rows ([`ex_poster_growers.json`](skills/k2f/catalog/content/ex_poster_growers.json)), not an auto-height stack. Zero-padding vertical stacks split by child when the page remainder is too small.
- **`export-docx` / `export-pptx`:** First major Office export pass — lock fonts and tracking, Dark Mode–safe theme colors and text-box underlays, card folding, wrap/alignment heuristics, hyperlinks, table row height and borders, running headers in the body for LibreOffice, and related DrawingML fixes. Slight host reflow remains expected; exports are not sources.

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
- Public creation path is `Editor.open_template` / `open_dir` / `open_bytes` plus `insert_node` JSON (superseded in 0.2.2 — use `init_package.py` / Gallery / unpack)

### Removed

- Public `Composer` / `Document` builder (`add_heading`, `add_body`, `add_warning`, `add_table`)

## [0.1.0] - 2026-08-28

### Added

- K2F format specification v0.1 (ZIP package, semantic tree, theme, compiled lock) — superseded by [Spec 0.3](docs/spec/k2f-v0.3.md)
- Reference engine: layout, paint, PDF export, package validation, Ed25519 signing
- CLI (`k2f`) — pack, compile, verify, sign, render, export-pdf, markdown
- Python SDK (`pip install k2f`) — Document, Editor, Markdown bridge
- JavaScript / WASM SDK (`npm i @openk2f/k2f`) — Document, Editor, `<k2f-viewer>` web component
- MCP stdio server (`k2f_mcp`) with tool catalog in `mcp_tools.json`
- Six agent skills under `skills/` (later unified; see 0.1.1)
- Public documentation under `docs/` and integrity policy in [SECURITY.md](SECURITY.md)

[Unreleased]: https://github.com/suzheng/k2f/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/suzheng/k2f/releases/tag/v0.3.1
[0.3.0]: https://github.com/suzheng/k2f/releases/tag/v0.3.0
[0.2.4]: https://github.com/suzheng/k2f/releases/tag/v0.2.4
[0.2.3]: https://github.com/suzheng/k2f/releases/tag/v0.2.3
[0.2.2]: https://github.com/suzheng/k2f/releases/tag/v0.2.2
[0.2.1]: https://github.com/suzheng/k2f/releases/tag/v0.2.1
[0.2.0]: https://github.com/suzheng/k2f/releases/tag/v0.2.0
[0.1.2]: https://github.com/suzheng/k2f/releases/tag/v0.1.2
[0.1.1]: https://github.com/suzheng/k2f/releases/tag/v0.1.1
[0.1.0]: https://github.com/suzheng/k2f/releases/tag/v0.1.0
