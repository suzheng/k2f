/** Thin JS/TS facade over the WASM Document builder. Geometry stays in the engine. */

export { wrapWasm } from "./core/wrap.js";
export { initWasm, createK2f } from "./core/init.js";
export { exportPdf } from "./core/export-pdf.js";
export { markdownToK2f, k2fToMarkdown } from "./core/markdown.js";
