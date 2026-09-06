import assert from "node:assert/strict";
import { bannerIsPinned, formatBytes } from "../viewer/chrome-behavior.js";
import { exportFormatLabel, EXPORT_FORMATS, normalizeExportFormat } from "../viewer/export-format.js";
import { normalizeTheme, preferredTheme } from "../viewer/theme.js";

assert.equal(formatBytes(0), "0 B");
assert.equal(formatBytes(512), "512 B");
assert.equal(formatBytes(2048), "2.0 KB");
assert.equal(formatBytes(2 * 1024 * 1024), "2.0 MB");

assert.equal(exportFormatLabel("pdf"), "Export as PDF");
assert.equal(exportFormatLabel("pptx"), "Export as PowerPoint");
assert.equal(normalizeExportFormat("nope"), "k2f");
assert.equal(EXPORT_FORMATS.length, 7);
assert.ok(EXPORT_FORMATS.every((f) => f.label.startsWith("Export as ")));

assert.equal(normalizeTheme("dark"), "dark");
assert.equal(normalizeTheme("light"), "light");
assert.equal(normalizeTheme("nope"), "light");
assert.ok(preferredTheme() === "light" || preferredTheme() === "dark");

function banner(classes, hidden = false) {
  return {
    hidden,
    classList: {
      contains(name) {
        return classes.includes(name);
      },
    },
  };
}

assert.equal(bannerIsPinned(banner(["banner-signed"])), false);
assert.equal(bannerIsPinned(banner(["banner-broken"])), true);
assert.equal(bannerIsPinned(banner(["banner-signed-broken"])), true);
assert.equal(bannerIsPinned(banner(["banner-broken"], true)), false);

console.log("ok chrome-ui");
