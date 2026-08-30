import { createK2f } from "../k2f.js";
import { invoicePackage } from "./helpers/invoice-package.mjs";

const k2f = await createK2f();
const bytes = invoicePackage(k2f);
const viewer = new k2f.Viewer(bytes);
if (typeof viewer.text_layer !== "function") {
  throw new Error("Viewer.text_layer must exist");
}
const spans = JSON.parse(viewer.text_layer(0));
if (!Array.isArray(spans) || !spans.length) {
  throw new Error("text_layer(0) must return spans");
}
for (const s of spans) {
  if (!s.text || !s.node_id) throw new Error(`empty span ${JSON.stringify(s)}`);
  if (!(s.width_pt > 0) || !(s.height_pt > 0)) {
    throw new Error(`span box ${JSON.stringify(s)}`);
  }
}
console.log(`ok text_layer spans=${spans.length} sample=${JSON.stringify(spans[0].node_id)}`);
