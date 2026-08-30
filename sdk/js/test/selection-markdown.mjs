import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createK2f } from "../k2f.js";
import { createViewer } from "../viewer.js";
import { invoicePackage } from "./helpers/invoice-package.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "../../..");

const k2f = await createK2f();
const invoiceBytes = invoicePackage(k2f);
const invoiceViewer = new k2f.Viewer(invoiceBytes);
if (typeof invoiceViewer.selection_markdown !== "function") {
  throw new Error("Viewer.selection_markdown must exist (rebuild sdk wasm)");
}
const header = "STATEMENT #2025-001";
const invoiceMd = invoiceViewer.selection_markdown(
  JSON.stringify([
    {
      node_id: "invoice.header",
      char_start: 0,
      char_end: [...header].length,
    },
  ]),
);
if (!invoiceMd.trimStart().startsWith("# ")) {
  throw new Error(`invoice header must be ATX h1, got:\n${invoiceMd}`);
}
if (!invoiceMd.includes("STATEMENT")) {
  throw new Error(`invoice markdown missing STATEMENT:\n${invoiceMd}`);
}
if (invoiceMd.includes("<!--")) {
  throw new Error(`clipboard must omit k2f hints:\n${invoiceMd}`);
}

const contract = readFileSync(join(root, "examples/published/contract.K2F"));
const viewerOnly = await createViewer();
const contractViewer = new viewerOnly.Viewer(contract);
if (typeof contractViewer.selection_markdown !== "function") {
  throw new Error("viewer-only selection_markdown must exist (rebuild viewer wasm)");
}
const title = "独立顾问协议";
const contractMd = contractViewer.selection_markdown(
  JSON.stringify([
    {
      node_id: "contract.title",
      char_start: 0,
      char_end: [...title].length,
    },
  ]),
);
if (!contractMd.trim()) {
  throw new Error("contract.title markdown empty");
}
if (!contractMd.includes(title)) {
  throw new Error(`contract markdown missing title:\n${contractMd}`);
}
if (contractMd.includes("<!--")) {
  throw new Error(`clipboard must omit k2f hints:\n${contractMd}`);
}

console.log(
  `ok selection_markdown invoice=${invoiceMd.trim().slice(0, 40)} contract=${contractMd.trim().slice(0, 20)}`,
);
