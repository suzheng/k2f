import { wrapWasm } from "./wrap.js";

let wasmReady = null;

export async function initWasm(wasmSource) {
  if (!wasmReady) {
    wasmReady = loadWasm(wasmSource).catch((err) => {
      wasmReady = null;
      throw err;
    });
  }
  return wasmReady;
}

async function loadWasm(wasmSource) {
  const mod = await import("../wasm/k2f_wasm.js");
  await mod.default({ module_or_path: await resolveWasm(wasmSource) });
  return mod;
}

async function resolveWasm(wasmSource) {
  if (wasmSource != null) return wasmSource;
  const url = new URL("../wasm/k2f_wasm_bg.wasm", import.meta.url);
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

export async function createK2f() {
  const wasm = await initWasm();
  return wrapWasm(wasm);
}
