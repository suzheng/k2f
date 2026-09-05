import { createK2f, exportPdf, exportPptx, exportDocx } from "../k2f.js";
import { invoicePackage } from "./helpers/invoice-package.mjs";

const k2f = await createK2f();
const bytes = invoicePackage(k2f);
const viewer = new k2f.Viewer(bytes);
if (!(viewer.page_count() > 0)) {
  throw new Error("pages");
}
if (viewer.page_count() < 2) {
  throw new Error(`invoice must paginate, got ${viewer.page_count()}`);
}
const pdf = await exportPdf(bytes);
if (!(pdf.length > 100)) {
  throw new Error("pdf bytes");
}
if (Buffer.from(pdf.subarray(0, 5)).toString() !== "%PDF-") {
  throw new Error("exportPdf must return a PDF of the lock");
}
const pptx = await exportPptx(bytes);
if (pptx[0] !== 0x50 || pptx[1] !== 0x4b) {
  throw new Error("exportPptx must return a ZIP");
}
const docx = await exportDocx(bytes);
if (docx[0] !== 0x50 || docx[1] !== 0x4b) {
  throw new Error("exportDocx must return a ZIP");
}
try {
  new k2f.Viewer(pdf);
  throw new Error("PDF must not open as a K2F source");
} catch (err) {
  const msg = String(err && err.message ? err.message : err);
  if (msg.includes("PDF must not open")) throw err;
  if (!msg.includes("PDF_IS_NOT_A_SOURCE")) {
    throw new Error(`expected PDF_IS_NOT_A_SOURCE, got ${msg}`);
  }
}
try {
  new k2f.Viewer(pptx);
  throw new Error("PPTX must not open as a K2F source");
} catch (err) {
  const msg = String(err && err.message ? err.message : err);
  if (msg.includes("PPTX must not open")) throw err;
  // PPTX guard is in k2f_pptx::export_bytes; unpack fails earlier with UNEXPECTED_PATH.
  if (!msg.includes("PPTX_IS_NOT_A_SOURCE") && !msg.includes("UNEXPECTED_PATH")) {
    throw new Error(`expected PPTX rejection, got ${msg}`);
  }
}
try {
  new k2f.Viewer(docx);
  throw new Error("DOCX must not open as a K2F source");
} catch (err) {
  const msg = String(err && err.message ? err.message : err);
  if (msg.includes("DOCX must not open")) throw err;
  // DOCX guard is in k2f_docx::export_bytes; unpack fails earlier with UNEXPECTED_PATH.
  if (!msg.includes("DOCX_IS_NOT_A_SOURCE") && !msg.includes("UNEXPECTED_PATH")) {
    throw new Error(`expected DOCX rejection, got ${msg}`);
  }
}
console.log(
  `ok smoke invoice package pages=${viewer.page_count()} pdf=${pdf.length} pptx=${pptx.length} docx=${docx.length} banner=${viewer.banner()}`,
);
