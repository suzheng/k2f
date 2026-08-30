import { createK2f, exportPdf } from "../k2f.js";
import { invoicePackage } from "./helpers/invoice-package.mjs";

const k2f = await createK2f();
const blob = invoicePackage(k2f);
if (blob[0] !== 0x50 || blob[1] !== 0x4b) {
  throw new Error("save must return a ZIP package");
}

const ed = k2f.Editor.openTemplate("invoice");
ed.insertNode("root", 0, {
  id: "invoice.note",
  role: "warning",
  content: { type: "text", value: "Net 30." },
  break_inside: "avoid",
});
ed.replaceText("invoice.note", "Net 14.");
if (ed.getNode("invoice.note").content.value !== "Net 14.") {
  throw new Error("replaceText must update the semantic node");
}
ed.setRunningFooter("Page {{page_current}} of {{page_total}}");

const pdf = await exportPdf(blob);
if (Buffer.from(pdf.subarray(0, 5)).toString() !== "%PDF-") {
  throw new Error("exportPdf must return a PDF");
}
const viewer = new k2f.Viewer(blob);
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
