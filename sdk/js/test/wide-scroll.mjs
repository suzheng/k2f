import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createK2f } from "../k2f.js";
import { fitZoom, snapToStep, stageInnerWidth } from "../viewer/zoom-fit.js";
import { invoicePackage } from "./helpers/invoice-package.mjs";
import { CHROME, runChromePage } from "./helpers/serve-chrome.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "../../..");

if (snapToStep(0.59) !== 0.5) throw new Error("snapToStep(0.59) should be 0.5");
if (snapToStep(1.1) !== 1) throw new Error("snapToStep(1.1) should be 1");
if (snapToStep(0.3) !== 0.5) throw new Error("snapToStep(0.3) should floor to 0.5");

const fakeStage = { clientWidth: 400 };
if (stageInnerWidth(fakeStage) !== 352) {
  throw new Error(`stageInnerWidth 400px container should be 352, got ${stageInnerWidth(fakeStage)}`);
}

const k2f = await createK2f();
const viewer = new k2f.Viewer(invoicePackage(k2f));
const fz = fitZoom(viewer, 352);
if (fz !== 0.5) throw new Error(`fitZoom A4 in 352px should be 0.5, got ${fz}`);
viewer.free();

if (!existsSync(CHROME)) {
  console.log("ok wide-scroll unit checks (Chrome not found, skip browser)");
  process.exit(0);
}

const invoiceBytes = Buffer.from(invoicePackage(k2f));

const result = await runChromePage(root, "/sdk/js/test/wide-scroll.html", {
  timeoutMs: 45000,
  extra(req, res) {
    const rel = decodeURIComponent((req.url || "/").split("?")[0]);
    if (rel === "/examples/invoice.K2F") {
      res.writeHead(200, { "content-type": "application/zip" });
      res.end(invoiceBytes);
      return true;
    }
    return false;
  },
});
const body = JSON.parse(result);
if (body.error) throw new Error(body.error);
if (body.initialZoom !== "50%") {
  throw new Error(`initial fit zoom should be 50%, got ${body.initialZoom}`);
}
if (!body.fitLeftOk || !body.fitRightOk) {
  throw new Error(`page should fit at open: ${JSON.stringify(body)}`);
}
if (!body.needsScroll) throw new Error("zoomed page should need horizontal scroll");
if (!body.leftOk || !body.rightOk) {
  throw new Error(`horizontal scroll should expose both edges: ${JSON.stringify(body)}`);
}
if (!body.fullscreenVisible) throw new Error("fullscreen button should be visible");
console.log(`ok wide-scroll ${JSON.stringify(body)}`);
