import { call } from "./errors.js";
import { initWasm } from "./init.js";

/** Word .docx of the published lock. Does not recompile. */
export async function exportDocx(packageBytes) {
  const wasm = await initWasm();
  const viewer = call(() => new wasm.K2fViewer(packageBytes));
  try {
    return call(() => viewer.export_docx());
  } finally {
    viewer.free();
  }
}
