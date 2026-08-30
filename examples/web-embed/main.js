import { createK2f } from "../../sdk/js/k2f.js";
import "../../sdk/js/viewer.js";
import { invoiceDocument } from "./invoice-doc.js";

const k2f = await createK2f();
const el = document.querySelector("k2f-viewer");
const packed = await fetch(new URL("../../examples/published/invoice.K2F", import.meta.url));
let bytes;
if (packed.ok) {
  const raw = new Uint8Array(await packed.arrayBuffer());
  try {
    const viewer = new k2f.Viewer(raw);
    viewer.free();
    bytes = raw;
  } catch {
    bytes = null;
  }
}
if (!bytes) {
  const data = await fetch(
    new URL("../invoice/assets/data/invoice_data.json", import.meta.url),
  ).then((r) => r.json());
  bytes = invoiceDocument(k2f, data);
}
await el.open(bytes);
