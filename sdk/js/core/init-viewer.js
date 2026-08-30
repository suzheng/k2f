import { wrapViewerWasm } from "./wrap-viewer.js";

let viewerReady = null;

export async function initViewerWasm(wasmSource) {
  if (!viewerReady) {
    viewerReady = loadViewerWasm(wasmSource).catch((err) => {
      viewerReady = null;
      throw err;
    });
  }
  return viewerReady;
}

async function loadViewerWasm(wasmSource) {
  const mod = await import("../wasm-viewer/k2f_wasm.js");
  await mod.default({ module_or_path: await resolveViewerWasm(wasmSource) });
  return wrapViewerWasm(mod);
}

async function resolveViewerWasm(wasmSource) {
  if (wasmSource != null) return wasmSource;
  const url = new URL("../wasm-viewer/k2f_wasm_bg.wasm", import.meta.url);
  if (
    url.protocol === "file:" &&
    typeof process !== "undefined" &&
    process.versions?.node
  ) {
    const { readFile } = await import(/* webpackIgnore: true */ "node:fs/promises");
    return await readFile(url);
  }
  return url;
}

export async function createViewer() {
  return initViewerWasm();
}
