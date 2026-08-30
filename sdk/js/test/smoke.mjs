import { createK2f, exportPdf } from "../k2f.js";
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
console.log(
  `ok smoke invoice package pages=${viewer.page_count()} pdf=${pdf.length} banner=${viewer.banner()}`,
);
