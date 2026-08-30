import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createK2f } from "../k2f.js";
import { createViewer } from "../viewer.js";
import { invoicePackage } from "./helpers/invoice-package.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "../../..");

const k2f = await createK2f();
const bytes = invoicePackage(k2f);
const viewer = new k2f.Viewer(bytes);
const title = viewer.title();
if (!title) throw new Error("viewer.title() must return document title");
viewer.free();

const published = readFileSync(join(root, "examples/published/contract.K2F"));
const viewerOnly = await createViewer();
const contractViewer = new viewerOnly.Viewer(published);
const pageCount = contractViewer.page_count();
if (pageCount < 2) throw new Error("contract must be multi-page");

const scale = viewerOnly.Viewer.official_scale();
for (const [format, check] of [
  ["k2f", (b) => b[0] === 0x50 && b[1] === 0x4b],
  ["pdf", (b) => b[0] === 0x25 && b[1] === 0x50],
  ["markdown", (b) => {
    const t = new TextDecoder().decode(b).trim();
    if (!t) throw new Error("markdown empty");
    if (t.includes("<!--")) throw new Error("markdown must omit hints");
  }],
  ["png", (b) => b[0] === 0x50 && b[1] === 0x4b],
  ["jpg", (b) => b[0] === 0x50 && b[1] === 0x4b],
]) {
  const v = new viewerOnly.Viewer(published);
  let out;
  if (format === "k2f") {
    out = published;
  } else if (format === "pdf") {
    out = v.export_pdf();
  } else if (format === "markdown") {
    out = new TextEncoder().encode(v.document_markdown());
  } else if (format === "png") {
    out = v.export_pages_png_zip(scale);
  } else {
    out = v.export_pages_jpeg_zip(scale);
  }
  check(out);
  v.free();
}

const invoiceViewer = new k2f.Viewer(bytes);
const png = invoiceViewer.export_pages_png_zip(scale);
if (png[0] === 0x89) {
  console.log("ok invoice single-page png path");
} else if (png[0] === 0x50) {
  console.log(`ok invoice multi-page png zip pages=${invoiceViewer.page_count()}`);
} else {
  throw new Error("invoice png export magic");
}
invoiceViewer.free();

console.log(`ok export-formats title=${title} contractPages=${pageCount}`);
