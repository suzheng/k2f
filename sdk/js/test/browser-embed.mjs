import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createK2f } from "../k2f.js";
import { invoicePackage } from "./helpers/invoice-package.mjs";
import { CHROME, runChromePage } from "./helpers/serve-chrome.mjs";

if (!existsSync(CHROME)) {
  console.log("skip browser-embed (Chrome not found)");
  process.exit(0);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "../../..");
const k2f = await createK2f();
const invoiceBytes = Buffer.from(invoicePackage(k2f));

const result = await runChromePage(root, "/sdk/js/test/browser-embed.html", {
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
if (body.pageLabel !== `2 / ${body.pages}`) throw new Error(`pageLabel ${body.pageLabel}`);
if (body.pdf !== "%PDF-") throw new Error(`pdf ${body.pdf}`);
if (!body.png) throw new Error("missing lock PNG");
console.log(`ok browser-embed ${JSON.stringify(body)}`);
