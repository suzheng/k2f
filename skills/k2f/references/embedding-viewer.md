# Embedding K2F Viewer

## Overview

Paint the published lock as engine PNG (official 2×) plus an optional `text_layer` for select/copy. Use `@openk2f/k2f/viewer` (`mountK2fViewer`, `<k2f-viewer>`). **Opening ≠ compiling.**

## Decision

| Context | Path |
|---------|------|
| Any website / Next / Vite / React | `import "@openk2f/k2f/viewer"` or `mountK2fViewer` |
| Surgical edit popover / `Editor.save()` | `editable: true` → sdk WASM + **Edit** toolbar (view mode by default) |
| Preview only (gallery, published `/v/{hash}`) | `runtime: "viewer"` + `initViewerWasm` **if** you have a viewer-only binary; otherwise `initWasm` |
| npm `@openk2f/k2f` package | Ships **sdk WASM only** — default to `initWasm` |

Import embed APIs only from `@openk2f/k2f/viewer` (not bare `@openk2f/k2f`) so `initWasm` is not duplicated.

## Workflow

1. Choose runtime (table above).
2. Place WASM under a public URL. From the `skills/k2f/` directory:

```bash
node scripts/copy-wasm.mjs --dest ./public
```

3. Client-only: `await initWasm("/k2f_wasm_bg.wasm")` (or `await initViewerWasm("/k2f_viewer_bg.wasm")` when the script copied that file), then mount. Bundler `import.meta.url` often 404s without this step.
4. Listen for `k2f-open` → check `detail.banner` / `detail.statusCode`. Fail → Failure protocol below.

Bundler details: [embedding-viewer/bundlers.md](embedding-viewer/bundlers.md). Handle / events / attributes: [embedding-viewer/api.md](embedding-viewer/api.md).

## Minimal embed

```html
<k2f-viewer src="/files/report.K2F"></k2f-viewer>
<script type="module">
  import "@openk2f/k2f/viewer";
</script>
```

Serve over HTTP (not `file://`). `<k2f-viewer>` observes only `src`; `editable` / `no-banner` are read at mount — toggling them later does not hot-swap.

## Rules

1. **No `fillText`** / CSS flow for document body text.
2. Selection uses engine `text_layer`; copy MIME: `text/plain` + `application/x-k2f-nodes+json`. Default `text/plain` is **Markdown** from the semantic tree (`selection_markdown`); **More → Copy as Plain Text** restores span text. Setting is stored as `localStorage.k2f.copyFormat`. Toolbar **Copy all as Markdown** copies the full document (`document_markdown`).
3. UI zoom is continuous CSS size (`page_pt × zoom`). Official paint is **2×** (baseline / contract). The screen may re-paint visible pages at a quantized display scale (`zoom × devicePixelRatio` buckets) after a short debounce — that is not a second layout engine.
4. `signed_by` on the file is an unbound claim — not a trusted issuer.

## Failure protocol

| Situation | Action |
|-----------|--------|
| WASM 404 | Fix public path; re-run `scripts/copy-wasm.mjs`; call matching `initWasm` / `initViewerWasm` |
| Wrong binary (sdk vs viewer-only) | Match runtime: editable/sdk → full wasm; preview-only → viewer wasm if present |
| Blank / `UNLOCKED` | No lock — `compile` or `Document.save()` / `Editor.save()` first |
| `BROKEN_INTEGRITY` / `SIGNED_BUT_BROKEN` | Show banner; paint **old** lock; do not silently recompile |
| CORS on `.K2F` fetch | Same-origin or CORS on file host |
| SSR canvas / `Uint8Array` as RSC prop | Client-only mount; pass `src` from server |

## Common mistakes

| Mistake | Reality |
|---------|---------|
| PDF.js / iframe PDF for `.K2F` | Use K2F viewer |
| HTML/CSS preview of semantic JSON | Breaks cross-device lock guarantee |
| Manual DOM text at lock coords | SDK text layer already does this |
| Hide banner on broken file | Misleading for signers |
| Register `<k2f-viewer>` via bare `@openk2f/k2f` | Import `@openk2f/k2f/viewer` |
| Expect npm to ship viewer-only WASM | Use `initWasm`; viewer-only is optional extra |

## See also

- [writing/sdk.md](writing/sdk.md) — what `editable` / popover save calls
- [publishing.md](publishing.md) — permanent `/v/{appearance_hash}` (not Playground `/p/{uuid}`)
- [exporting-pdf.md](exporting-pdf.md) — lock PDF download
- [writing.md](writing.md) — produce files to embed
