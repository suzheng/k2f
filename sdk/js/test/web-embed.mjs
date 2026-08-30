import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { CHROME, runChromePage } from "./helpers/serve-chrome.mjs";

if (!existsSync(CHROME)) {
  console.log("skip web-embed (Chrome not found)");
  process.exit(0);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "../../..");
const result = await runChromePage(root, "/sdk/js/test/web-embed.html");
const body = JSON.parse(result);
if (body.error) throw new Error(body.error);
if (!(body.pages > 1)) throw new Error(`pages ${body.pages}`);
if (body.pageLabel !== `2 / ${body.pages}`) throw new Error(`pageLabel ${body.pageLabel}`);
if (body.pdf !== "%PDF-") throw new Error(`pdf ${body.pdf}`);
if (!body.png) throw new Error("missing lock PNG");
console.log(`ok web-embed ${JSON.stringify(body)}`);
