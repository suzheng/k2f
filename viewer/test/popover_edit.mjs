import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createK2f } from "../../sdk/js/k2f.js";
import { invoicePackage } from "../../sdk/js/test/helpers/invoice-package.mjs";
import { CHROME, runChromePage } from "../../sdk/js/test/helpers/serve-chrome.mjs";

if (!existsSync(CHROME)) {
  console.log("skip popover_edit (Chrome not found)");
  process.exit(0);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const k2f = await createK2f();
const key = k2f.generateSigningKey();
const signed = Buffer.from(
  k2f.sign(invoicePackage(k2f), key.secret_hex, "tester", BigInt(1_704_067_200)),
);

const result = await runChromePage(root, "/viewer/test/popover_edit.html", {
  extra(req, res) {
    const rel = decodeURIComponent((req.url || "/").split("?")[0]);
    if (rel === "/examples/invoice.K2F") {
      res.writeHead(200, { "content-type": "application/zip" });
      res.end(signed);
      return true;
    }
    return false;
  },
});

const body = JSON.parse(result);
if (body.error) throw new Error(body.error);
if (!body.popover) throw new Error("popover missing after click");
if (!body.viewModeNoPopover) throw new Error("view mode must not open popover");
if (body.bannerBefore !== "SIGNED") {
  throw new Error(`expected SIGNED before save, got ${body.bannerBefore}`);
}
if (body.bannerAfter !== "UNSIGNED") {
  throw new Error(`expected UNSIGNED after save, got ${body.bannerAfter} ${body.statusAfter}`);
}
if (!body.textAfter || !body.textAfter.includes("110.00")) {
  throw new Error(`relock text ${body.textAfter}`);
}
console.log(`ok popover_edit ${JSON.stringify(body)}`);
