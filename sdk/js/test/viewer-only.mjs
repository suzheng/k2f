import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createViewer } from "../viewer.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "../../..");
const contract = readFileSync(join(root, "examples/published/contract.K2F"));

const k2f = await createViewer();
if (k2f.Editor) {
  throw new Error("viewer-only wasm must not expose Editor");
}
const viewer = new k2f.Viewer(contract);
if (typeof viewer.selection_markdown !== "function") {
  throw new Error("viewer-only wasm must export selection_markdown");
}
const png = viewer.render_page(0, k2f.Viewer.official_scale());
if (png[0] !== 0x89 || png[1] !== 0x50) {
  throw new Error("render_page must return PNG bytes");
}
const pptx = viewer.export_pptx();
if (pptx[0] !== 0x50 || pptx[1] !== 0x4b) {
  throw new Error("viewer-only export_pptx must return a ZIP");
}
const docx = viewer.export_docx();
if (docx[0] !== 0x50 || docx[1] !== 0x4b) {
  throw new Error("viewer-only export_docx must return a ZIP");
}
console.log(`ok viewer-only pages=${viewer.page_count()} png=${png.length} pptx=${pptx.length} docx=${docx.length}`);
