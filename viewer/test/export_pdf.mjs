import { createK2f, exportPdf } from "../../sdk/js/k2f.js";
import { invoicePackage } from "../../sdk/js/test/helpers/invoice-package.mjs";

const k2f = await createK2f();
const bytes = invoicePackage(k2f);
const viewer = new k2f.Viewer(bytes);
if (viewer.page_count() < 2) {
  throw new Error(`invoice must paginate, got ${viewer.page_count()}`);
}
const pdf = Buffer.from(await exportPdf(bytes));
if (pdf.subarray(0, 5).toString() !== "%PDF-") {
  throw new Error("exportPdf must return a PDF");
}
try {
  new k2f.Viewer(pdf);
  throw new Error("PDF must not open as a K2F source");
} catch (err) {
  const msg = String(err && err.message ? err.message : err);
  if (!msg.includes("PDF_IS_NOT_A_SOURCE")) {
    throw new Error(`expected PDF_IS_NOT_A_SOURCE, got ${msg}`);
  }
}
console.log(`ok invoice export_pdf bytes=${pdf.length} pages=${viewer.page_count()}`);
