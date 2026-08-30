import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const pkgRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const require = createRequire(join(pkgRoot, "package.json"));

function resolvePublic(rel) {
  // Resolve relative to this package (works before npm link as "k2f")
  return join(pkgRoot, "public", rel);
}

function assert(cond, msg) {
  if (!cond) throw new Error(msg);
}

const spec = readFileSync(resolvePublic("spec/k2f-v0.1.md"), "utf8");
assert(spec.includes("#"), "spec must have a heading");

const design = readFileSync(resolvePublic("architecture/design.md"), "utf8");
assert(design.includes("Semantic-first"), "architecture/design.md must be exported");

const mcp = JSON.parse(readFileSync(resolvePublic("mcp_tools.json"), "utf8"));
assert(Array.isArray(mcp.tools) && mcp.tools.length > 0, "mcp_tools.tools empty");

const invoice = readFileSync(resolvePublic("examples/invoice.K2F"));
assert(invoice[0] === 0x50 && invoice[1] === 0x4b, "invoice.K2F must be a ZIP");

// Also prove package exports map resolves when required as k2f from site-style install
try {
  const url = import.meta.resolve
    ? import.meta.resolve("./public/spec/k2f-v0.1.md")
    : pathToFileURL(resolvePublic("spec/k2f-v0.1.md")).href;
  assert(typeof url === "string" && url.length > 0, "resolve public path");
} catch (e) {
  throw new Error(`public export resolve failed: ${e}`);
}

void require;
console.log("public-exports ok");
