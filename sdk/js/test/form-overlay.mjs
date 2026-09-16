import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createK2f } from "../k2f.js";
import { compileAuthorDir } from "./helpers/invoice-package.mjs";
import { CHROME, runChromePage } from "./helpers/serve-chrome.mjs";

if (!existsSync(CHROME)) {
  console.log("skip form-overlay (Chrome not found)");
  process.exit(0);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "../../..");
const k2f = await createK2f();
const contractBytes = Buffer.from(compileAuthorDir(k2f, "examples/contract"));

const result = await runChromePage(root, "/sdk/js/test/form-overlay.html", {
  timeoutMs: 60000,
  extra(req, res) {
    const rel = decodeURIComponent((req.url || "/").split("?")[0]);
    if (rel === "/examples/contract.K2F") {
      res.writeHead(200, { "content-type": "application/zip" });
      res.end(contractBytes);
      return true;
    }
    return false;
  },
});

const body = JSON.parse(result);
if (body.error) throw new Error(body.error);
if (body.browseInputs !== 0) throw new Error(`browse inputs ${body.browseInputs}`);
if (!(body.fieldCount >= 4)) throw new Error(`fieldCount ${body.fieldCount}`);
if (body.fillInputs !== body.fieldCount) {
  throw new Error(`fillInputs ${body.fillInputs} != ${body.fieldCount}`);
}
if (body.valueBefore !== "") throw new Error(`unsaved value leaked: ${body.valueBefore}`);
if (body.valueAfter !== "张三") throw new Error(`saved value ${body.valueAfter}`);
if (body.afterInputs !== 0) throw new Error(`inputs after save ${body.afterInputs}`);
if (body.readonlyInputs !== 0) throw new Error(`readonly inputs ${body.readonlyInputs}`);
console.log(`ok form-overlay ${JSON.stringify(body)}`);
