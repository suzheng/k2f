import { createK2f } from "../../sdk/js/k2f.js";
import { invoicePackage } from "../../sdk/js/test/helpers/invoice-package.mjs";

// Compile through the SDK. Packed examples/contract.K2F may fail SCHEMA_INVALID
// if its theme predates the current styles schema.

const k2f = await createK2f();
const bytes = invoicePackage(k2f);
const viewer = new k2f.Viewer(bytes);
const banner = viewer.banner();
const code = viewer.status_code();
if (banner !== "UNSIGNED" && !(banner === "BROKEN_INTEGRITY" && code === "ENGINE_MISMATCH")) {
  throw new Error(`expected UNSIGNED (or ENGINE_MISMATCH in release wasm), got ${banner} ${code}`);
}
if (viewer.page_count() < 1) {
  throw new Error("expected at least one lock page");
}
const png = viewer.render_page(0, k2f.Viewer.official_scale());
if (png[0] !== 0x89 || png[1] !== 0x50) {
  throw new Error("render_page must return a PNG of the lock");
}
console.log(`ok compiled invoice ${banner} ${code} pages=${viewer.page_count()} png=${png.length}`);
