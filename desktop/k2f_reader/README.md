# k2f-reader

Native lock executor for `.K2F` files. Same rules as the web viewer:

- Opening does not recompile
- Pixels come from `document.K2F.lock`
- PDF export draws the lock
- PPTX / DOCX export also draws the lock (not a second layout engine)
- A PDF is not a K2F source (`PDF_IS_NOT_A_SOURCE`)

Packaged builds: [k2f.dev/download](https://k2f.dev/download).

Links `k2f_paint` + `k2f_package` plus the PDF / PPTX / DOCX / Markdown exporters. The official raster is `render_page` at `OFFICIAL_PNG_SCALE` (2×) — baseline, export, and golden comparisons. UI zoom is continuous (buttons still use steps); the screen may re-paint visible pages at a quantized display scale (LOD) after a short debounce. Surgical edit is not in v0.

This crate is a workspace member but **not a default-member** (same pattern as `k2f_py`), so root `cargo test` does not pull GUI crates. CI runs `cargo test -p k2f_reader`.

## Build

From the repo root:

```bash
cargo build -p k2f_reader --release
```

Binary: `target/release/k2f-reader`.

On Debian/Ubuntu, compiling the window crate needs:

```bash
sudo apt-get install -y pkg-config libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libx11-dev libx11-xcb-dev libxcursor-dev libxrandr-dev libxi-dev
```

`cargo test -p k2f_reader` does not need a display or a Save dialog.

## Run

```bash
cargo run -p k2f_reader --release -- examples/published/invoice.K2F
```

Debug `cargo run -p k2f_reader --` also works; `--release` is snappier for first paint. Zoom updates layout immediately; display LOD re-paints after idle when a denser bucket is needed.

Title: document title for `UNSIGNED`; `K2F Reader — Signed — {title}` or `K2F Reader — Draft — {title}` when applicable; `BROKEN_INTEGRITY` / `SIGNED_BUT_BROKEN` keep the raw codes (with `status_code` under broken). Packaged app: clicking the icon opens an empty window (no Open dialog). **Open** on the toolbar, **File → Open** on macOS, or Ctrl/Cmd+O picks a `.K2F`. Double-clicking a `.K2F` (or passing it on the command line) loads that lock in the window. Toolbar with a document: Open, title, zoom `−` / `%` / `+`, copy format, **Export as** split button (last format) plus a caret menu of **Export as K2F** / **PDF** / PowerPoint / Word (**DOCX**) / Markdown / PNG / JPG — choosing a row exports immediately (same as the web viewer). Integrity chrome matches web `banner: "auto"`: quiet for `UNSIGNED`; compact Signed / Draft strips; plain-language warning for broken locks (not a full-width `BROKEN_INTEGRITY` ticker). Status bar: page, zoom, format. Pages stack vertically; the wheel scrolls them. Left/Right jump so the next sheet sits under the toolbar. Zoom scales the lock bitmap inside a stable window (default 1280×820, min 960×640).

| Key / gesture | Action |
| --- | --- |
| Wheel / trackpad scroll | Scroll the stacked pages |
| **macOS** trackpad pinch | Continuous zoom about the cursor (LOD re-paints after idle) |
| **Ctrl+wheel** (Windows / Linux / macOS) | Continuous zoom about the cursor |
| Left / Right | Previous / next page |
| `+` / `-` | Stepped zoom (0.1–3.0); scroll offset kept, then clamped |
| Drag on lock text | Select characters (I-beam cursor, blue highlight like the web viewer) |
| Ctrl/Cmd+O | Open a `.K2F` (native Open dialog) |
| Ctrl/Cmd+C | Copy the selection (`text/plain`) |
| Ctrl/Cmd+Shift+S | Export in the selected format (native Save) |

Pinch is a macOS/iOS winit event (`PinchGesture`); Windows and Linux use **Ctrl+wheel** for the same continuous zoom. Trackpad two-finger scroll never zooms unless Ctrl is held.

Drag across a line of lock text (not the toolbar or status bar) selects glyph runs the same way the web viewer does. The highlight stays after you release; releasing also copies. A click without a drag is a collapsed selection and copies nothing.

## Headless

No window. Safe for CI. Commands below work from a clone (no `PATH` install):

```bash
cargo run -p k2f_reader -- --verify path/to/file.K2F
cargo run -p k2f_reader -- --export-pdf out.pdf examples/published/invoice.K2F
cargo run -p k2f_reader -- --export-pptx out.pptx examples/published/invoice.K2F
cargo run -p k2f_reader -- --export-docx out.docx examples/published/invoice.K2F
```

`--verify` prints the banner on stdout. Exit `0` if `UNSIGNED` or `SIGNED`; exit `1` if broken / unlocked (`status_code` on stderr); exit `2` on usage (no FILE). Unlocked files have no lock, so export fails. `--export-pdf` / `--export-pptx` / `--export-docx` still write the published lock when the banner is broken. Export flags conflict with each other. `--verify` together with one export writes then verifies.

A self-consistent lock compiled by another engine stays `UNSIGNED` or `SIGNED`. `hash_code` may be `ENGINE_MISMATCH` (reader provenance, not a broken banner). Rewriting lock engine fields without updating `appearance_hash` is `APPEARANCE_CHANGED`. Do not treat `ENGINE_MISMATCH` as `BROKEN_INTEGRITY`. Use `cargo test -p k2f_reader` in CI. Tests never open a window or a Save dialog.

## Not in v0

Code signing / notarization, Tauri / Electron / wgpu.
