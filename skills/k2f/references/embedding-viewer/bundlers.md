# Bundlers and WASM placement

## Default (npm `@openk2f/k2f`)

1. `npm i @openk2f/k2f` so `node_modules/@openk2f/k2f/wasm/k2f_wasm_bg.wasm` exists.
2. Copy it to a URL the browser can fetch:

```bash
node scripts/copy-wasm.mjs --dest ./public
```

(Run from the `skills/k2f/` directory, or pass the script path. `--dest` is the app’s static folder.)

Typical result: `public/k2f_wasm_bg.wasm`.

3. Before mount:

```js
import { initWasm, mountK2fViewer } from "@openk2f/k2f/viewer";

await initWasm("/k2f_wasm_bg.wasm");
const handle = await mountK2fViewer(host, bytes, { editable: false });
```

The published npm package **does not** include a viewer-only binary. External apps always use `initWasm` (full sdk WASM). `runtime: "viewer"` needs a separately built viewer-only file named `k2f_viewer_bg.wasm` next to the sdk file.

## Dual WASM (optional)

If `copy-wasm.mjs` also finds a viewer-only binary, it writes `k2f_viewer_bg.wasm`:

| Public URL | Init | When |
|------------|------|------|
| `/k2f_wasm_bg.wasm` | `initWasm` | Edit, compile, Editor; npm default |
| `/k2f_viewer_bg.wasm` | `initViewerWasm` | Paint/preview only |

Keep init + `runtime` in **one** helper so you never mix binaries (sdk init then viewer mount, or the reverse).

## Rules for bundlers

- Call `initWasm` / `initViewerWasm` with an explicit public URL when the bundler cannot resolve `.wasm` next to the JS chunk.
- Mount only on the client — do not SSR the canvas.
- Do not pass `Uint8Array` as a React Server Component prop; pass `src` or load bytes on the client.
- Lazy-load: `import("@openk2f/k2f/viewer")` to keep the main bundle small.
- WASM will not load from `file://` — serve over HTTP.
- Import mount APIs from `@openk2f/k2f/viewer` only (avoids a second `initWasm` via bare `@openk2f/k2f`).

## Runtime selection (`mountK2fViewer` options)

| Condition | WASM used |
|-----------|-----------|
| `editable: true` | sdk (Editor + Viewer) |
| `runtime: "sdk"` | sdk |
| else (default preview) | viewer-only if `initViewerWasm` was given a file; else sdk |
