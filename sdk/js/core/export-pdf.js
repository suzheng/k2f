import { call } from "./errors.js";
import { initWasm } from "./init.js";

/** PDF of the published lock. Does not recompile. Optional scale: 2, 3, or 4. */
export async function exportPdf(packageBytes, scale = 2) {
  const wasm = await initWasm();
  const viewer = call(() => new wasm.K2fViewer(packageBytes));
  try {
    if (typeof viewer.export_pdf_at === "function") {
      return call(() => viewer.export_pdf_at(scale));
    }
    return call(() => viewer.export_pdf());
  } finally {
    viewer.free();
  }
}
