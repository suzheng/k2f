import { createK2f, exportPdf, exportPptx, exportDocx } from "../k2f.js";
import { packAuthorDir } from "./helpers/invoice-package.mjs";

const k2f = await createK2f();
const blob = packAuthorDir("templates/invoice");
if (blob[0] !== 0x50 || blob[1] !== 0x4b) {
  throw new Error("invoice fixture must be a ZIP package");
}

const ed = k2f.Editor.open(blob);
ed.insertNode("root", 0, {
  id: "invoice.sdk_note",
  role: "warning",
  content: { type: "text", value: "Net 30." },
  break_inside: "avoid",
});
ed.replaceText("invoice.sdk_note", "Net 14.");
if (ed.getNode("invoice.sdk_note").content.value !== "Net 14.") {
  throw new Error("replaceText must update the semantic node");
}
ed.setRunningFooter("Page {{page_current}} of {{page_total}}");

// save first so exportPptx draws a compiled package.
const saved = ed.save();
const fromEditor = ed.exportPptx();
if (fromEditor[0] !== 0x50 || fromEditor[1] !== 0x4b) {
  throw new Error("Editor.exportPptx must return a ZIP");
}
const viaAlias = ed.export_pptx();
if (viaAlias.length !== fromEditor.length) {
  throw new Error("export_pptx alias must match exportPptx");
}
const fromBytes = await exportPptx(saved);
if (fromBytes[0] !== 0x50) throw new Error("exportPptx(saved) must be a ZIP");
const afterSave = ed.exportPptx();
if (Buffer.from(afterSave).compare(Buffer.from(fromBytes)) !== 0) {
  throw new Error("Editor.exportPptx after save must match exportPptx(bytes)");
}

const fromEditorDocx = ed.exportDocx();
if (fromEditorDocx[0] !== 0x50 || fromEditorDocx[1] !== 0x4b) {
  throw new Error("Editor.exportDocx must return a ZIP");
}
const viaAliasDocx = ed.export_docx();
if (viaAliasDocx.length !== fromEditorDocx.length) {
  throw new Error("export_docx alias must match exportDocx");
}
const fromBytesDocx = await exportDocx(saved);
if (fromBytesDocx[0] !== 0x50) throw new Error("exportDocx(saved) must be a ZIP");
const afterSaveDocx = ed.exportDocx();
if (Buffer.from(afterSaveDocx).compare(Buffer.from(fromBytesDocx)) !== 0) {
  throw new Error("Editor.exportDocx after save must match exportDocx(bytes)");
}

const pdf = await exportPdf(saved);
if (Buffer.from(pdf.subarray(0, 5)).toString() !== "%PDF-") {
  throw new Error("exportPdf must return a PDF");
}
const viewer = new k2f.Viewer(saved);
if (viewer.page_count() < 1) throw new Error("compiled invoice has no pages");
try {
  new k2f.Viewer(new Uint8Array([0, 1, 2, 3]));
  throw new Error("garbage bytes must not open");
} catch (err) {
  if (err.message === "garbage bytes must not open") throw err;
  if (!err.message) throw new Error("Viewer must throw Error with message");
  if (!err.code) throw new Error("Viewer must throw Error with code");
}
console.log(`ok editor facade zip=${blob.length} pdf=${pdf.length} pages=${viewer.page_count()}`);
