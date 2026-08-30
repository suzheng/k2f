import { createK2f } from "../k2f.js";
import { invoicePackage } from "./helpers/invoice-package.mjs";
import "../viewer.js";

const k2f = await createK2f();
const bytes = invoicePackage(k2f);
const viewer = new k2f.Viewer(bytes);
const png = viewer.render_page(0, k2f.Viewer.official_scale());
if (png[0] !== 0x89 || png[1] !== 0x50) {
  throw new Error("render_page must return a PNG of the lock");
}
console.log(
  `ok viewer api pages=${viewer.page_count()} png=${png.length} scale=${k2f.Viewer.official_scale()}`,
);
