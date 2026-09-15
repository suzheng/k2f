import { createK2f, exportPdf, exportPptx, exportDocx, exportIdml } from "../k2f.js";
import { invoicePackage } from "./helpers/invoice-package.mjs";
import { zipFirstEntry } from "./helpers/zip-first-entry.mjs";

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
const idml = await exportIdml(bytes);
if (idml[0] !== 0x50 || idml[1] !== 0x4b) {
  throw new Error("exportIdml must return a ZIP");
}
const viaViewerIdml = viewer.export_idml();
if (Buffer.from(idml).compare(Buffer.from(viaViewerIdml)) !== 0) {
  throw new Error("exportIdml(bytes) must match Viewer.export_idml()");
}
const first = zipFirstEntry(idml);
if (first.name !== "mimetype" || first.compression !== "stored") {
  throw new Error("IDML mimetype must be first stored entry");
}
const mimeBody = new TextDecoder().decode(first.data).trim();
if (mimeBody !== "application/vnd.adobe.indesign-idml-package") {
  throw new Error(`IDML mimetype body wrong: ${mimeBody}`);
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
try {
  new k2f.Viewer(idml);
  throw new Error("IDML must not open as a K2F source");
} catch (err) {
  const msg = String(err && err.message ? err.message : err);
  if (msg.includes("IDML must not open")) throw err;
  // IDML guard is in k2f_idml::export_bytes; unpack fails earlier with UNEXPECTED_PATH.
  if (!msg.includes("IDML_IS_NOT_A_SOURCE") && !msg.includes("UNEXPECTED_PATH")) {
    throw new Error(`expected IDML rejection, got ${msg}`);
  }
}
console.log(
  `ok smoke invoice package pages=${viewer.page_count()} pdf=${pdf.length} pptx=${pptx.length} docx=${docx.length} idml=${idml.length} banner=${viewer.banner()}`,
);
