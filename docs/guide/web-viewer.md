# Web viewer

Embed `.K2F` packages in the browser with `@openk2f/k2f/viewer` (`<k2f-viewer>`, `mountK2fViewer`). The component is a **lock executor**: it paints `document.K2F.lock` as engine PNG (official 2×) plus a transparent `text_layer` for select and copy. **Opening ≠ compiling** — viewers do not re-layout body text.

Use HTTP to serve packages and WASM (`file://` will not load WASM). For a runnable sample, see [examples/web-embed](https://github.com/suzheng/k2f/tree/main/examples/web-embed) in the repo.

## Install

```bash
npm install @openk2f/k2f
```

Node.js 18+ for tooling; the viewer runs in modern browsers. You only need this package when you embed the viewer — authoring and CLI workflows use `pip install k2f` ([Getting started](getting-started.md)).

## Choose an integration path

| Goal | What to use |
| --- | --- |
| Any site (Next, Vite, React, static HTML) | `import "@openk2f/k2f/viewer"` or `mountK2fViewer` |
| Surgical edit + **Edit** toolbar / `Editor.save()` | `editable: true` (or `editable` attribute) → full **sdk** WASM |
| Paint-only preview (gallery, published `/v/{hash}`) | Prefer **viewer-only** WASM when you have it; otherwise sdk WASM works for read-only mount |
| npm default | Ships **sdk** WASM under `@openk2f/k2f/wasm/`. Viewer-only WASM is under `@openk2f/k2f/wasm-viewer/` when the published tarball includes it — copy both with the skill script if present |

Import mount APIs only from `@openk2f/k2f/viewer`, not bare `@openk2f/k2f`, so `initWasm` is not initialized twice.

## Quick start

```html
<k2f-viewer src="/files/report.K2F"></k2f-viewer>
<script type="module">
  import "@openk2f/k2f/viewer";
</script>
```

Serve the page and the `.K2F` over HTTP. The element fetches `src` when connected; only `src` is observed — `editable` and `no-banner` are read at mount (toggle them by remounting or `element.open(bytes)`).

Programmatic mount:

```javascript
import { initWasm, mountK2fViewer } from "@openk2f/k2f/viewer";

await initWasm("/k2f_wasm_bg.wasm");
const bytes = await fetch("/files/report.K2F").then((r) => r.arrayBuffer());
const handle = await mountK2fViewer(document.getElementById("host"), new Uint8Array(bytes));
// handle.destroy() on teardown
```

Listen for `k2f-open` on the host (`bubbles`, `composed`): `detail.banner`, `detail.statusCode`, `detail.pageCount`, `detail.handle`.

## WASM setup

1. Copy WASM into a public URL. From [`skills/k2f/`](../../skills/k2f/):

```bash
node scripts/copy-wasm.mjs --dest ./public
```

Typical output: `public/k2f_wasm_bg.wasm`. If a viewer-only binary exists next to the sdk file in `node_modules`, the script also writes `k2f_viewer_bg.wasm`.

2. Before the first mount, call **`initWasm("/k2f_wasm_bg.wasm")`** (full sdk — edit, export, compile paths). For paint-only with a separate viewer binary: **`initViewerWasm("/k2f_viewer_bg.wasm")`**.

3. If the bundler cannot resolve `.wasm` beside the JS chunk, always pass an explicit URL (as above). Vite can use `import wasmUrl from "@openk2f/k2f/wasm/k2f_wasm_bg.wasm?url"` then `await initWasm(wasmUrl)`.

**Bundler rules:** mount only on the client (no SSR canvas); do not pass `Uint8Array` through React Server Components — pass `src` or fetch bytes on the client. Lazy-load `import("@openk2f/k2f/viewer")` to keep the main bundle small.

**Runtime selection** (`mountK2fViewer` options): `editable: true` or `runtime: "sdk"` → sdk WASM. Otherwise the mount uses viewer-only WASM when `initViewerWasm` was used; if not, sdk WASM is used. Keep init and `runtime` in one helper so you never mix binaries.

## `<k2f-viewer>` attributes

| Attribute | Effect |
| --- | --- |
| `src` | URL of a `.K2F` package (fetched on connect / when `src` changes). **Only observed attribute.** |
| `editable` | **Edit** toolbar + surgical popover (view mode by default). Read at mount. |
| `no-banner` | Hide integrity chrome (`banner: "off"`). Read at mount. |

## `mountK2fViewer` options

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `banner` | `boolean \| "auto" \| "full" \| "off"` | `"auto"` | Integrity chrome: tiered (`auto`), legacy verbose strip (`true` / `full`), hidden (`false` / `off`) |
| `editable` | `boolean` | `false` | Edit toolbar + popover; forces sdk WASM |
| `runtime` | `"sdk" \| "viewer"` | inferred | Explicit WASM flavor (see above) |
| `copyFormat` | `"markdown" \| "plain"` | from `localStorage` | Initial copy format (`k2f.copyFormat`) |
| `exportFormat` | see below | from `localStorage` | Initial **Export as** format (`k2f.exportFormat`) |

**View vs edit:** With `editable`, the toolbar shows **Edit** / **Done**. View mode: no popover on click (hyperlinks still open). Edit mode: click node → text-first popover → Save and relock.

**Form fill:** When the lock contains `form_field` nodes, the toolbar offers **Fill** / **Save** (native overlays on lock rectangles; Save calls `replace_text` and relocks). This is viewer chrome, not a flowing editor — see [Editor architecture](../architecture/editor.md).

## `ViewerHandle`

| Method / field | Purpose |
| --- | --- |
| `destroy()` | Tear down listeners and free WASM objects |
| `goPage(n)` | Scroll page `n` into view |
| `export(format?)` | Bytes from the lock — `k2f`, `pdf`, `pptx`, `docx`, `idml`, `markdown`, `png`, `jpg` (default from toolbar / `exportFormat`) |
| `selectId(id)` | Jump to a node (popover only in edit mode) |
| `open(bytes)` | Replace the open package |
| `editing` | `true` when the Edit toggle is active |
| `viewer` / `editor` | Underlying WASM objects (`editor` is `null` when not editable) |

Types: `ViewerMountOptions`, `ViewerHandle` on `@openk2f/k2f` and `@openk2f/k2f/viewer`.

## Events

| Event | Detail |
| --- | --- |
| `k2f-open` | `{ banner, statusCode, pageCount, handle }` |
| `k2f-page-change` | `{ page }` |
| `k2f-select` | Selection JSON for the clicked / selected node |

## Integrity banners

Derived from package integrity. Default `banner: "auto"` uses tiered visibility (same idea as the [format spec](../spec/k2f-v0.3.md#viewer-banners)):

| Banner | Meaning | Default UI |
| --- | --- | --- |
| `SIGNED` | Valid hash chain + valid signature | Compact green strip |
| `UNSIGNED` | Valid hashes, no signature | Hidden (quiet read) |
| `SIGNED_BUT_BROKEN` | Signature present but hash/crypto failed | Prominent warning |
| `BROKEN_INTEGRITY` | Content / appearance / font mismatch — paints **old** lock | Prominent warning |
| `UNLOCKED` | No `document.K2F.lock` | Compact draft strip |

Use `banner: "full"` for legacy verbose strips. Use `no-banner` / `banner: "off"` to hide chrome — still read `detail.banner` from `k2f-open`. `signed_by` on the file is an unbound claim; trusted issuer display names come from your app, not the package alone.

## Selection and copy

- Transparent spans from `viewer.text_layer(page)` sit over the PNG.
- Copy MIME types: `text/plain` and `application/x-k2f-nodes+json` when the selection intersects K2F text spans.
- Default `text/plain` is **Markdown** from the semantic tree (`selection_markdown`). **More → Copy as Markdown / Copy as Plain Text** (stored in `localStorage.k2f.copyFormat`). Toolbar **Copy all as Markdown** uses `document_markdown()`.

## Paint and zoom contract

1. **No `fillText` or CSS flow** for document body text — the lock is the only layout source ([Contributing](/contributing)).
2. UI zoom is continuous CSS size (`page_pt × zoom`). Official paint is **2×** (`Viewer.official_scale()`). The screen may re-paint visible pages at a quantized display scale after a short debounce — that is not a second layout engine.
3. Page size comes from lock `page_config`, not the viewport.

## When something goes wrong

| Situation | Action |
| --- | --- |
| WASM 404 | Fix public path; re-run `copy-wasm.mjs`; call matching `initWasm` / `initViewerWasm` |
| Wrong binary (sdk vs viewer-only) | Match init to mount: editable / `runtime: "sdk"` → sdk; preview-only → viewer wasm when present |
| Blank / `UNLOCKED` | No lock — `compile` or `Document.save()` / `Editor.save()` first |
| `BROKEN_INTEGRITY` / `SIGNED_BUT_BROKEN` | Show banner; paint **old** lock; do not silently recompile |
| CORS on `.K2F` fetch | Same-origin or CORS on the file host |
| SSR / RSC | Client-only mount; pass `src` from the server, not raw bytes |

## Common mistakes

| Mistake | Reality |
| --- | --- |
| PDF.js or an iframe for `.K2F` | Use the K2F viewer |
| HTML/CSS preview of semantic JSON | Breaks cross-device lock guarantee |
| Manual DOM text at lock coordinates | Use the built-in text layer |
| Hide banner on a broken file | Misleading for signers |
| Register `<k2f-viewer>` via bare `@openk2f/k2f` | Import `@openk2f/k2f/viewer` |
| Expect viewer-only WASM without copying it | Use `initWasm` for all mounts when only sdk wasm is present |

## See also

- [Getting started](getting-started.md) — produce a `.K2F` to embed
- [Markdown conversion](markdown.md) — quick content from Markdown
- [Integrity banners](/integrity) — UI tiers and verify codes
- [Publishing](../../skills/k2f/references/publishing.md) — permanent `/v/{appearance_hash}` viewer URLs
- [Export](exporting.md) — PDF, PowerPoint, Word, InDesign from the lock
- [Export PDF](exporting-pdf.md) — fillable AcroForm and flatten
- [Desktop reader](../../desktop/k2f_reader/README.md) — native lock executor (same paint rules)
- [K2F Skill](../../skills/k2f/SKILL.md) — agent workflows (`editable` / popover: [writing/sdk.md](../../skills/k2f/references/writing/sdk.md))
