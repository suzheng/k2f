import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createK2f } from "../k2f.js";
import { invoicePackage } from "./helpers/invoice-package.mjs";
import { CHROME, runChromePage } from "./helpers/serve-chrome.mjs";

if (!existsSync(CHROME)) {
  console.log("skip stack-scroll (Chrome not found)");
  process.exit(0);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "../../..");
const k2f = await createK2f();
const invoiceBytes = Buffer.from(invoicePackage(k2f));

const result = await runChromePage(root, "/sdk/js/test/stack-scroll.html", {
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
if (!(body.pages > 1)) throw new Error(`pages ${body.pages}`);
if (!(body.wraps > 1)) throw new Error(`wraps ${body.wraps}`);
if (!(body.pngs > 1)) throw new Error(`pngs ${body.pngs}`);
if (!(body.scrollHeight > body.firstH)) {
  throw new Error(`scrollHeight ${body.scrollHeight} firstH ${body.firstH}`);
}
if (body.afterScroll !== `2 / ${body.pages}`) {
  throw new Error(`afterScroll ${body.afterScroll}`);
}
if (body.afterGo !== `2 / ${body.pages}`) {
  throw new Error(`afterGo ${body.afterGo}`);
}
console.log(`ok stack-scroll ${JSON.stringify(body)}`);
