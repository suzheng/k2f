#!/usr/bin/env bash
# Emit contracts and assert required banner / verify ids.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
node scripts/emit-js-contracts.mjs

node --input-type=module <<'EOF'
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const dir = "sdk/js/public/contracts";
const banners = JSON.parse(readFileSync(join(dir, "banners.json"), "utf8"));
const codes = JSON.parse(readFileSync(join(dir, "verify-codes.json"), "utf8"));

function need(arr, item, label) {
  if (!arr.includes(item)) throw new Error(`${label} missing ${item}: ${arr}`);
}

if (!Array.isArray(banners) || banners.length === 0) throw new Error("empty banners");
if (!Array.isArray(codes) || codes.length === 0) throw new Error("empty verify-codes");
if (existsSync(join(dir, "templates.json"))) {
  throw new Error("templates.json must not be emitted (SDK does not bundle templates)");
}

need(banners, "UNSIGNED", "banners");
need(banners, "BROKEN_INTEGRITY", "banners");
need(codes, "UNSIGNED", "verify-codes");
need(codes, "PDF_IS_NOT_A_SOURCE", "verify-codes");

console.log("check-js-contracts ok");
EOF
