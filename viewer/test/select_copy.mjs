import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createK2f } from "../../sdk/js/k2f.js";
import { invoicePackage } from "../../sdk/js/test/helpers/invoice-package.mjs";
import { CHROME, runChromePage } from "../../sdk/js/test/helpers/serve-chrome.mjs";

if (!existsSync(CHROME)) {
  console.log("skip select_copy (Chrome not found)");
  process.exit(0);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const k2f = await createK2f();
const invoiceBytes = Buffer.from(invoicePackage(k2f));

const result = await runChromePage(root, "/viewer/test/select_copy.html", {
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
if (!(body.spanCount > 0)) throw new Error(`spanCount ${body.spanCount}`);
if (!(body.brCount >= body.spanCount)) throw new Error(`brCount ${body.brCount}`);
if (!body.plain || body.plain.startsWith("{") || body.plain.includes('"id"')) {
  throw new Error(`plain text must be human, got ${JSON.stringify(body.plain)}`);
}
if (!body.nodes || !body.nodes.length || !body.nodes[0].node_id) {
  throw new Error(`k2f nodes ${JSON.stringify(body.nodes)}`);
}
if (!body.copyPlain || body.copyPlain.startsWith("{") || body.copyPlain.includes('"id"')) {
  throw new Error(`copy event plain must be human, got ${JSON.stringify(body.copyPlain)}`);
}
const visible = body.plain.trim();
if (visible && !body.copyPlain.includes(visible.split(/\s+/)[0])) {
  throw new Error(
    `copyPlain must include selected text, plain=${JSON.stringify(body.plain)} copy=${JSON.stringify(body.copyPlain)}`,
  );
}
if (body.firstNodeId === "invoice.header" && !body.copyPlain.trimStart().startsWith("# ")) {
  throw new Error(`invoice header copy must be Markdown h1, got ${JSON.stringify(body.copyPlain)}`);
}
if (!body.copyNodes || !body.copyNodes.includes(body.nodes[0].node_id)) {
  throw new Error(`copy event nodes ${body.copyNodes}`);
}
if (!body.multiPlain || !body.multiPlain.includes("\n")) {
  throw new Error(`multi-span copy must keep line breaks, got ${JSON.stringify(body.multiPlain)}`);
}
console.log(
  `ok select_copy ${JSON.stringify({ spanCount: body.spanCount, copyPlain: body.copyPlain.slice(0, 40) })}`,
);
