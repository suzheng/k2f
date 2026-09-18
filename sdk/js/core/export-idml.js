import { call } from "./errors.js";
import { initWasm } from "./init.js";

/** InDesign package zip (IDML + Document Fonts). Does not recompile. */
export async function exportIdml(packageBytes) {
  const wasm = await initWasm();
  const viewer = call(() => new wasm.K2fViewer(packageBytes));
  try {
    return call(() => viewer.export_idml());
  } finally {
    viewer.free();
  }
}

/** Lone .idml with no Document Fonts. */
export async function exportIdmlOnly(packageBytes) {
  const wasm = await initWasm();
  const viewer = call(() => new wasm.K2fViewer(packageBytes));
  try {
    return call(() => viewer.export_idml_only());
  } finally {
    viewer.free();
  }
}
