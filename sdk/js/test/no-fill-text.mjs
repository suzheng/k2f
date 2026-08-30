import { readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const skip = new Set(["wasm", "test", "pkg"]);

function walk(dir) {
  const out = [];
  for (const name of readdirSync(dir)) {
    if (skip.has(name)) continue;
    const p = join(dir, name);
    if (statSync(p).isDirectory()) out.push(...walk(p));
    else if (p.endsWith(".js") || p.endsWith(".mjs")) out.push(p);
  }
  return out;
}

const roots = [
  join(here, ".."),
  join(here, "../../../viewer"),
  join(here, "../../../examples/web-embed"),
];

for (const root of roots) {
  for (const file of walk(root)) {
    const src = readFileSync(file, "utf8");
    if (src.includes("fillText")) {
      throw new Error(`fillText is forbidden in the viewer path: ${file}`);
    }
  }
}

console.log("ok no fillText in sdk/js, viewer, examples/web-embed");
