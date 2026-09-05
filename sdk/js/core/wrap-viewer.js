import { agentError, call } from "./errors.js";

/** Viewer-only WASM facade — no Editor (no layout compile). */
export function wrapViewerWasm(wasm) {
  return {
    Viewer: class Viewer extends wasm.K2fViewer {
      constructor(bytes) {
        try {
          super(bytes);
        } catch (err) {
          throw agentError(err);
        }
      }
      render_page(page, scale) {
        return call(() => super.render_page(page, scale));
      }
      export_pdf() {
        return call(() => super.export_pdf());
      }
      export_pptx() {
        return call(() => super.export_pptx());
      }
      export_docx() {
        return call(() => super.export_docx());
      }
    },
  };
}
