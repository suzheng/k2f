#!/usr/bin/env node
/**
 * Copy K2F WASM binaries into a public directory for bundlers.
 *
 * Usage (from skills/k2f/ directory):
 *   node scripts/copy-wasm.mjs [--dest ./public] [--from /path/to/k2f_wasm_bg.wasm]
 *
 * Resolves sdk WASM from (first hit):
 *   --from
 *   <cwd>/node_modules/@openk2f/k2f/wasm/k2f_wasm_bg.wasm
 *   walk up from <cwd> looking for node_modules/@openk2f/k2f/wasm/k2f_wasm_bg.wasm
 *
 * If a sibling wasm-viewer/k2f_wasm_bg.wasm exists next to the sdk file’s
 * parent (…/wasm-viewer/ vs …/wasm/), also copies k2f_viewer_bg.wasm.
 */
import {
  closeSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  openSync,
  readSync,
  statSync,
} from "node:fs";
import { dirname, join, parse, resolve } from "node:path";

const WASM_MAGIC = Buffer.from([0x00, 0x61, 0x73, 0x6d]);

function parseArgs(argv) {
  let dest = join(process.cwd(), "public");
  let from = null;
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--dest" && argv[i + 1]) {
      dest = resolve(argv[++i]);
    } else if (argv[i] === "--from" && argv[i + 1]) {
      from = resolve(argv[++i]);
    }
  }
  return { dest, from };
}

function hasWasmMagic(path) {
  const fd = openSync(path, "r");
  const buf = Buffer.alloc(4);
  try {
    readSync(fd, buf, 0, 4, 0);
  } finally {
    closeSync(fd);
  }
  return buf.equals(WASM_MAGIC);
}

function walkUp(startDir, rel) {
  let dir = resolve(startDir);
  const { root } = parse(dir);
  while (true) {
    const candidate = join(dir, rel);
    if (existsSync(candidate)) return candidate;
    if (dir === root) return null;
    dir = dirname(dir);
  }
}

function firstExisting(candidates) {
  for (const p of candidates) {
    if (p && existsSync(p)) return p;
  }
  return null;
}

function copyOne(src, dst, label) {
  if (!existsSync(src)) {
    console.error(`missing ${label}: ${src}`);
    return false;
  }
  if (!hasWasmMagic(src)) {
    console.error(`invalid WASM magic in ${src} (${label})`);
    return false;
  }
  mkdirSync(dirname(dst), { recursive: true });
  copyFileSync(src, dst);
  console.log(`copied ${label} -> ${dst} (${statSync(dst).size} bytes)`);
  return true;
}

const { dest, from } = parseArgs(process.argv.slice(2));

const sdkSrc = firstExisting([
  from,
  join(process.cwd(), "node_modules/@openk2f/k2f/wasm/k2f_wasm_bg.wasm"),
  walkUp(process.cwd(), "node_modules/@openk2f/k2f/wasm/k2f_wasm_bg.wasm"),
  walkUp(process.cwd(), "sdk/js/wasm/k2f_wasm_bg.wasm"),
]);

if (!sdkSrc) {
  console.error(
    "sdk WASM not found. Install the @openk2f/k2f package so this exists:\n" +
      "  node_modules/@openk2f/k2f/wasm/k2f_wasm_bg.wasm\n" +
      "Or pass --from /absolute/path/to/k2f_wasm_bg.wasm",
  );
  process.exit(1);
}

const sdkDst = join(dest, "k2f_wasm_bg.wasm");
if (!copyOne(sdkSrc, sdkDst, "sdk")) process.exit(1);

const wasmDir = dirname(sdkSrc);
const viewerSrc = join(dirname(wasmDir), "wasm-viewer/k2f_wasm_bg.wasm");
let viewerCopied = false;
if (existsSync(viewerSrc)) {
  const viewerDst = join(dest, "k2f_viewer_bg.wasm");
  if (!copyOne(viewerSrc, viewerDst, "viewer-only")) process.exit(1);
  viewerCopied = true;
} else {
  console.warn("viewer-only WASM skipped (optional; npm @openk2f/k2f does not ship it).");
}

console.log("");
console.log("Init before mount:");
console.log(`  await initWasm("/k2f_wasm_bg.wasm");`);
if (viewerCopied) {
  console.log(`  await initViewerWasm("/k2f_viewer_bg.wasm");  // preview-only runtime`);
} else {
  console.log("  (no viewer-only binary: use initWasm for all mounts)");
}
