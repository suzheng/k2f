import { call } from "./errors.js";
import { initWasm } from "./init.js";

/** Adobe InDesign .idml of the published lock. Does not recompile. */
export async function exportIdml(packageBytes) {
  const wasm = await initWasm();
  const viewer = call(() => new wasm.K2fViewer(packageBytes));
  try {
    return call(() => viewer.export_idml());
  } finally {
    viewer.free();
  }
}
