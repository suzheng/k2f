import { initWasm, createK2f, exportPdf, exportPptx, exportDocx, exportIdml } from "./k2f.js";
import { initViewerWasm, createViewer } from "./core/init-viewer.js";
import { mountK2fViewer } from "./viewer/mount.js";
import { defineK2fViewer } from "./viewer/element.js";

export { initWasm, createK2f, exportPdf, exportPptx, exportDocx, exportIdml, initViewerWasm, createViewer, mountK2fViewer };
export { K2fViewerElement } from "./viewer/element.js";

defineK2fViewer();
