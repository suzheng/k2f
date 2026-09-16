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
      export_pdf_at(scale) {
        return call(() => super.export_pdf_at(scale));
      }
      export_pdf_with(scale, flatten) {
        return call(() => super.export_pdf_with(scale, flatten));
      }
      export_pptx() {
        return call(() => super.export_pptx());
      }
      export_docx() {
        return call(() => super.export_docx());
      }
      export_idml() {
        return call(() => super.export_idml());
      }
    },
  };
}
