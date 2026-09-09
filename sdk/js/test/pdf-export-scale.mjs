import assert from "node:assert/strict";
import {
  DEFAULT_PDF_EXPORT_SCALE,
  normalizePdfExportScale,
  pdfExportScaleLabel,
} from "../viewer/pdf-export-scale.js";

assert.equal(DEFAULT_PDF_EXPORT_SCALE, 4);
assert.equal(normalizePdfExportScale(4), 4);
assert.equal(normalizePdfExportScale(2), 2);
assert.equal(normalizePdfExportScale(99), 4);
assert.match(pdfExportScaleLabel(4), /Maximum/);

console.log("ok pdf-export-scale");
