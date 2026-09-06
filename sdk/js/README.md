# k2f (JavaScript / WASM)

Embed `.K2F` documents in the browser, build documents with the agent SDK, edit by stable node id, and export PDF from the published lock.

**License:** Apache-2.0

## Install

```bash
npm install @openk2f/k2f
```

Requires Node.js 18+ for tooling; the viewer runs in modern browsers over HTTP (WASM does not load from `file://`).

## Two package entries

| Import | Use when |
|--------|----------|
| `@openk2f/k2f` | Editor, Markdown, signing — full SDK WASM |
| `@openk2f/k2f/viewer` | `<k2f-viewer>` web component and paint-only preview |

Both ship WASM under `@openk2f/k2f/wasm/` (full) and `@openk2f/k2f/wasm-viewer/` (viewer-only). Copy the `.wasm` file to a public URL if your bundler cannot resolve it next to the JS module.

## Editor API

Open an official template and insert semantic nodes:

```javascript
import { initWasm, createK2f, exportPdf } from "@openk2f/k2f";

await initWasm("/k2f_wasm_bg.wasm");
const k2f = await createK2f();
const ed = k2f.Editor.openTemplate("invoice");

ed.insertNode("root", 0, {
  id: "invoice.title",
  role: "h1",
  content: { type: "text", value: "Invoice #1042" },
  keep_with_next: true,
});
ed.insertNode("root", 1, {
  id: "invoice.body",
  role: "body",
  content: { type: "text", value: "Payment due in 30 days." },
});

const bytes = ed.save();
const pdf = await exportPdf(bytes);
```

Edit an existing package by stable dotted id:

```javascript
const editor = k2f.Editor.open(bytes);

console.log(editor.outline());
console.log(editor.diff());

editor.replaceText("invoice.body", "Updated copy.");
const updated = editor.save();
```

## Markdown and signing

```javascript
import { initWasm, createK2f } from "@openk2f/k2f";

await initWasm("/k2f_wasm_bg.wasm");
const k2f = await createK2f();

const fromMd = k2f.markdownToK2f("# Hello\n\nBody text.", {
  title: "From Markdown",
  template: "report",
});
const md = k2f.k2fToMarkdown(fromMd);

const key = k2f.generateSigningKey();
const signed = k2f.sign(fromMd, key.secret_hex, "agent@example.com");
```

## Web component

```html
<k2f-viewer src="/files/report.K2F" editable></k2f-viewer>
<script type="module">
  import "@openk2f/k2f/viewer";
</script>
```

| Attribute | Effect |
|-----------|--------|
| `src` | URL of a `.K2F` package |
| `editable` | Enable Edit toolbar + surgical popover |
| `no-banner` | Hide integrity chrome (`banner: "off"`) |

Custom events: `k2f-open`, `k2f-page-change`, `k2f-select`.

Programmatic mount (editable — requires full SDK WASM):

```javascript
import { initWasm, mountK2fViewer } from "@openk2f/k2f/viewer";

await initWasm("/k2f_wasm_bg.wasm");
const handle = await mountK2fViewer(host, bytes, {
  editable: true,
});
handle.export("pdf");
handle.destroy();
```

Preview-only mount (no surgical edit):

```javascript
import { initViewerWasm, mountK2fViewer } from "@openk2f/k2f/viewer";

await initViewerWasm("/k2f_viewer_bg.wasm");
const handle = await mountK2fViewer(host, bytes, { editable: false });
handle.destroy();
```

## WASM deployment (Vite / Next.js / Webpack)

1. Run `initWasm("/k2f_wasm_bg.wasm")` before edit/compile flows, or `initViewerWasm("/k2f_viewer_bg.wasm")` for paint-only preview.
2. Copy from `node_modules/@openk2f/k2f/wasm/k2f_wasm_bg.wasm` (or `wasm-viewer/`) into `public/`.
3. For Vite, you can also import the WASM URL and pass it to `initWasm`.

Example (Vite):

```javascript
import wasmUrl from "@openk2f/k2f/wasm/k2f_wasm_bg.wasm?url";
import { initWasm, createK2f } from "@openk2f/k2f";

await initWasm(wasmUrl);
```

## Public contracts (`k2f/public/*`)

Stable export paths for spec, guides, schemas, examples, and MCP tool JSON. See [`public/README.md`](public/README.md).

In a git checkout these are symlinks to the repo source; the npm tarball contains real files.

## TypeScript

Types ship in `types.d.ts` for both `@openk2f/k2f` and `@openk2f/k2f/viewer` exports.

## More documentation

- [Repository README](https://github.com/suzheng/k2f#readme)
- [Web viewer guide](https://github.com/suzheng/k2f/blob/main/docs/guide/web-viewer.md)
- [Format specification v0.1](https://github.com/suzheng/k2f/blob/main/docs/spec/k2f-v0.1.md)
