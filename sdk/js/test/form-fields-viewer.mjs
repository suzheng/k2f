import assert from "node:assert/strict";
import { createK2f } from "../k2f.js";
import { buildFormLayer, fieldsOnPage } from "../viewer/form-layer.js";
import { compileAuthorDir, invoicePackage } from "./helpers/invoice-package.mjs";
import { installMiniDom } from "./helpers/mini-dom.mjs";

installMiniDom();

const k2f = await createK2f();
const invoice = new k2f.Viewer(invoicePackage(k2f));
assert.equal(typeof invoice.form_fields, "function", "rebuild sdk wasm for form_fields()");
assert.deepEqual(JSON.parse(invoice.form_fields()), []);
invoice.free();

const bytes = compileAuthorDir(k2f, "examples/contract");
const viewer = new k2f.Viewer(bytes);
const fields = JSON.parse(viewer.form_fields());
assert.ok(fields.length >= 4, `contract fields ${fields.map((f) => f.id)}`);
for (const need of [
  "contract.signatures.client_name",
  "contract.signatures.client_date",
  "contract.signatures.contractor_name",
  "contract.signatures.contractor_date",
]) {
  assert.ok(fields.some((f) => f.id === need), `missing ${need} in ${fields.map((f) => f.id)}`);
}
for (const f of fields) {
  assert.equal(typeof f.page, "number");
  assert.ok(f.width > 0 && f.height > 0, `${f.id} empty box`);
  assert.equal(f.kind, "text");
}

const pages = new Map();
for (const f of fields) {
  if (!pages.has(f.page)) pages.set(f.page, document.createElement("div"));
  pages.get(f.page).dataset.page = String(f.page);
}
let controls = 0;
for (const [page, wrap] of pages) {
  const layer = buildFormLayer(wrap, fieldsOnPage(fields, page), 1, { values: new Map() });
  controls += layer.querySelectorAll("[data-field-id]").length;
}
assert.equal(controls, fields.length);
viewer.free();

console.log(`ok form-fields-viewer contract fields=${fields.length} pages=${pages.size}`);
