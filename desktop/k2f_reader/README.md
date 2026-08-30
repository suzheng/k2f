# k2f-reader

Native lock executor for `.K2F` files. Same rules as the web viewer:

- Opening does not recompile
- Pixels come from `document.K2F.lock`
- PDF export draws the lock
- A PDF is not a K2F source (`PDF_IS_NOT_A_SOURCE`)

Links `k2f_paint` + `k2f_package` + `k2f_pdf` only. The official raster is `render_page` at `OFFICIAL_PNG_SCALE` (2×). Zoom is UI scale of that bitmap. Surgical edit is not in v0.

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
cargo run -p k2f_reader -- examples/published/invoice.K2F
```

Title: `K2F Reader — {banner} — {title}`. Under `BROKEN_INTEGRITY` the title also shows `status_code`. HUD: banner, `page / count`, zoom, export format (**K2F** / **PDF** / **MD** / **PNG** / **JPG**), **Export**. Pages stack vertically; the wheel scrolls them. Left/Right jump so the next sheet sits under the HUD.

| Key | Action |
| --- | --- |
| Wheel / trackpad | Scroll the stacked pages |
| Left / Right | Previous / next page |
| `+` / `-` | Zoom (0.5–3.0) |
| Ctrl/Cmd+C | Copy selected text (`text/plain`) |
| Ctrl/Cmd+Shift+S | Export in the selected format (native Save) |

Drag on the page (not the HUD) maps window px → document pt and copies intersecting text-layer spans.

## Headless

No window. Safe for CI. Commands below work from a clone (no `PATH` install):

```bash
cargo run -p k2f_reader -- --verify path/to/file.K2F
cargo run -p k2f_reader -- --export-pdf out.pdf examples/published/invoice.K2F
```

`--verify` prints the banner on stdout. Exit `0` if `UNSIGNED` or `SIGNED`; exit `1` if broken / unlocked (`status_code` on stderr); exit `2` on usage (no FILE). Unlocked files have no lock, so export fails. `--export-pdf` still writes the published lock when the banner is broken. Both flags together write the PDF then verify.

Do not gate CI on `--verify` of `examples/published/*.K2F`. A committed lock can already be `ENGINE_MISMATCH` after an engine bump. Use `cargo test -p k2f_reader` (SDK fixtures). Tests never open a window or a Save dialog.

## Not in v0

File association, code signing / notarization, Tauri / Electron / wgpu.
