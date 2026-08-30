import { createK2f } from "../../sdk/js/k2f.js";
import { invoicePackage } from "../../sdk/js/test/helpers/invoice-package.mjs";

const k2f = await createK2f();
const bytes = invoicePackage(k2f);
const viewer = new k2f.Viewer(bytes);

const boxes = JSON.parse(viewer.boxes_for("invoice.total"));
if (!boxes.length) throw new Error("invoice.total must have a lock box");
const box = boxes[0];
const x = (box.x + box.width / 2) / 1000;
const y = (box.y + box.height / 2) / 1000;
const total = JSON.parse(viewer.hit_test(box.page, x, y));
if (total.ids[0] !== "invoice.total") {
  throw new Error(`expected invoice.total, got ${JSON.stringify(total)}`);
}
const sel = JSON.parse(viewer.hit_selection(box.page, x, y));
if (sel.id !== "invoice.total" || !sel.text.includes("7,047.00")) {
  throw new Error(`bad hit selection: ${JSON.stringify(sel)}`);
}

const clip = JSON.parse(viewer.clipboard("invoice.total"));
if (clip.id !== "invoice.total" || !clip.text.includes("7,047.00") || clip.x != null) {
  throw new Error(`clipboard must be semantic, got ${JSON.stringify(clip)}`);
}

const editor = k2f.Editor.open(bytes);
editor.replaceText("invoice.total", "Grand Total: $110.00");
const out = editor.save();
const again = new k2f.Viewer(out);
const sel2 = JSON.parse(again.selection("invoice.total"));
if (sel2.text !== "Grand Total: $110.00") throw new Error(sel2.text);
if (again.banner() !== "UNSIGNED" && again.status_code() !== "ENGINE_MISMATCH") {
  throw new Error(`expected UNSIGNED after relock, got ${again.banner()} ${again.status_code()}`);
}

console.log(`ok hit ${sel.id} relock ${again.banner()}`);
