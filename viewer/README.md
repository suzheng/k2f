# K2F viewer (lock executor)

This page is a thin host for the `@openk2f/k2f/viewer` web component. It unpacks a `.K2F`, verifies hashes, and paints `document.K2F.lock` through WASM. Click a box to select a node. Save replaces that node's text, recompiles, drops any signature, and downloads a new `.K2F`.

## Build

Requires `wasm-bindgen-cli` 0.2.x (same minor as the `wasm-bindgen` crate) or `wasm-pack`.

From the repo root:

```bash
scripts/build-viewer.sh
```

Serve the **repo root** so the host can import `sdk/js`:

```bash
python3 -m http.server 8000
```

Open http://127.0.0.1:8000/viewer/ and drop a current `.K2F` (or compile one with `createK2f()` / `k2f_cli`). Packed `examples/*.K2F` files are gitignored and may fail `SCHEMA_INVALID` if their theme predates the current schema. **Export PDF** downloads a drawing of the published lock. A PDF is not a K2F source; dropping one shows `PDF_IS_NOT_A_SOURCE`.

Banner states: **SIGNED**, **UNSIGNED**, **SIGNED-BUT-BROKEN**, **BROKEN INTEGRITY**, **UNLOCKED**. Identity is the public-key fingerprint. Known issuers are a viewer config map (`KNOWN_ISSUERS`), not part of the file format.

## Zoom

UI zoom is CSS scale of a raster produced at **2×** (`OFFICIAL_PNG_SCALE`). Pixel comparisons must use that raster, not a screenshot of the window. Page width comes from `page_config`, not from the viewport. There is no reading-mode reflow.

## Native reader

A native lock executor lives in [`desktop/k2f_reader`](../desktop/k2f_reader/README.md). Same contract: opening does not recompile; pixels and PDF come from `document.K2F.lock`. Packaged builds: website `/download`. Build from source with `cargo run -p k2f_reader`.
