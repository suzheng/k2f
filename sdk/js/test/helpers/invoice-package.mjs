import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { invoiceDocument } from "../../../../examples/web-embed/invoice-doc.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "../../../..");

/** Compile a real invoice package through the WASM Document API. */
export function invoicePackage(k2f) {
  const data = JSON.parse(
    readFileSync(join(root, "examples/invoice/assets/data/invoice_data.json"), "utf8"),
  );
  return invoiceDocument(k2f, data);
}
