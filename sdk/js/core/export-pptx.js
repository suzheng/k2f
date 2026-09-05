import { call } from "./errors.js";
import { initWasm } from "./init.js";

/** PowerPoint .pptx of the published lock. Does not recompile. */
export async function exportPptx(packageBytes) {
  const wasm = await initWasm();
  const viewer = call(() => new wasm.K2fViewer(packageBytes));
  try {
    return call(() => viewer.export_pptx());
  } finally {
    viewer.free();
  }
}
