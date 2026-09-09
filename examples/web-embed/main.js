import { createK2f } from "../../sdk/js/k2f.js";
import "../../sdk/js/viewer.js";

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
  throw new Error(
    "web-embed requires examples/published/invoice.K2F (SDK does not bundle templates)",
  );
}
await el.open(bytes);
