# Embed K2F in a website

The `@openk2f/k2f` npm package paints `.K2F` the same way PDF.js paints PDF: unpack, verify, rasterize the published lock. The browser is not the layout engine.

```bash
bash scripts/build-sdk-js.sh
```

Serve the **repo root** (not this folder) so the example can fetch `sdk/js/` and invoice data:

```bash
python3 -m http.server 8000
```

Open http://127.0.0.1:8000/examples/web-embed/

Expected: stacked invoice pages you can scroll (Word-style), integrity banner, Previous/Next jump a page, Export PDF downloads `k2f-lock.pdf` from the lock (no re-layout).

This demo opens `examples/invoice.K2F` when that file exists and the current engine accepts it. Otherwise it compiles the same invoice from `examples/invoice/assets/data/invoice_data.json` through `createK2f()` (packed `.K2F` files are gitignored and may fail `SCHEMA_INVALID` after a schema change).

## Static HTML

```html
<k2f-viewer src="/files/report.K2F" editable></k2f-viewer>
<script type="module">
  import "@openk2f/k2f/viewer";
</script>
```

Or pass bytes:

```js
import { createK2f } from "@openk2f/k2f";
import { mountK2fViewer } from "@openk2f/k2f/viewer";

const k2f = await createK2f();
const bytes = /* Uint8Array of a .K2F package */;
await mountK2fViewer(document.getElementById("k2f-root"), bytes, { editable: true });
```

## Vite / React

```javascript
import { mountK2fViewer, initWasm } from "@openk2f/k2f/viewer";

await initWasm();
const bytes = await fetch("/invoice.K2F").then((r) => r.arrayBuffer());
const handle = await mountK2fViewer(document.getElementById("k2f-root"), new Uint8Array(bytes), {
  banner: true,
  editable: true,
});
// handle.destroy() on unmount
```

If the bundler does not emit the `.wasm` next to the JS, copy `node_modules/@openk2f/k2f/wasm/k2f_wasm_bg.wasm` into `public/` and call `initWasm("/k2f_wasm_bg.wasm")` first.

## Next.js App Router

1. Copy `k2f_wasm_bg.wasm` to `public/`.
2. `await initWasm("/k2f_wasm_bg.wasm")` before mount (or set `assetPrefix` if the app is not at `/`).
3. Mount only on the client — do not SSR the viewer.
4. Load `.K2F` with `fetch` or pass bytes from a server route.

## Rules

- **Never** use `fillText` or CSS flow for document body text.
- Opening a locked file paints the lock. It does not recompile.
- Page width comes from `page_config`, not the viewport.
- WASM will not load from `file://`. Use HTTP.
