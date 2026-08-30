#!/usr/bin/env node
/**
 * Emit sdk/js/public/contracts/*.json from Rust sources (single source of truth).
 * Called by build-sdk-js.sh and check-js-contracts.sh.
 */
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const outDir = join(root, "sdk/js/public/contracts");

function parseBannerStrings(src) {
  return [...src.matchAll(/Banner::\w+ => "([A-Z_]+)"/g)].map((m) => m[1]);
}

function parseVerifyCodes(src) {
  return [...src.matchAll(/pub const CODE_[A-Z_]+: &str = "([A-Z_]+)"/g)].map((m) => m[1]);
}

function parseTemplateNames(src) {
  const m = src.match(/pub const OFFICIAL_IDS:\s*&\[&str\]\s*=\s*&\[([^\]]+)\]/);
  if (!m) throw new Error("OFFICIAL_IDS missing in templates/mod.rs");
  return [...m[1].matchAll(/"([a-z_]+)"/g)].map((x) => x[1]);
}

function writeJson(name, value) {
  writeFileSync(join(outDir, name), `${JSON.stringify(value, null, 2)}\n`);
}

mkdirSync(outDir, { recursive: true });

const banners = parseBannerStrings(
  readFileSync(join(root, "engine/k2f_paint/src/banner.rs"), "utf8"),
);
const verifyCodes = parseVerifyCodes(
  readFileSync(join(root, "engine/k2f_package/src/error.rs"), "utf8"),
);
const templates = parseTemplateNames(
  readFileSync(join(root, "engine/k2f_sdk/src/templates/mod.rs"), "utf8"),
);

if (banners.length === 0) throw new Error("no banners emitted");
if (verifyCodes.length === 0) throw new Error("no verify codes emitted");
if (templates.length === 0) throw new Error("no templates emitted");

writeJson("banners.json", banners);
writeJson("verify-codes.json", verifyCodes);
writeJson("templates.json", templates);

console.log(
  `contracts -> ${outDir} (banners=${banners.length} codes=${verifyCodes.length} templates=${templates.length})`,
);
