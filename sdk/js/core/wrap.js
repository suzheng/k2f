import { agentError, call } from "./errors.js";

/** Thin JS/TS facade over the WASM SDK. Geometry stays in the engine. */
export function wrapWasm(wasm) {
  return {
    systemPrompt: () => call(() => wasm.system_prompt()),
    Editor: class {
      static open(bytes) {
        const e = Object.create(this.prototype);
        e._ed = call(() => new wasm.K2fEditor(bytes));
        return e;
      }
      getNode(id) {
        return JSON.parse(call(() => this._ed.get_node(id)));
      }
      outline() {
        return JSON.parse(call(() => this._ed.outline()));
      }
      diff() {
        return JSON.parse(call(() => this._ed.diff()));
      }
      selection(id) {
        return JSON.parse(call(() => this._ed.selection(id)));
      }
      clipboard(id) {
        return JSON.parse(call(() => this._ed.clipboard(id)));
      }
      replaceText(id, text) {
        call(() => this._ed.replace_text(id, text));
      }
      replace_text(id, text) {
        this.replaceText(id, text);
      }
      setRole(id, role, variant) {
        call(() => this._ed.set_role(id, role, variant));
      }
      set_role(id, role, variant) {
        this.setRole(id, role, variant);
      }
      insertNode(parentId, index, node) {
        const json =
          typeof node === "string" ? node : JSON.stringify(node);
        call(() => this._ed.insert_node(parentId, index, json));
      }
      deleteNode(id) {
        call(() => this._ed.delete_node(id));
      }
      setGeneratedBy(id) {
        call(() => this._ed.set_generated_by(id));
      }
      setRunningHeader(text) {
        call(() => this._ed.set_running_header(text));
      }
      set_running_header(text) {
        this.setRunningHeader(text);
      }
      setRunningFooter(text) {
        call(() => this._ed.set_running_footer(text));
      }
      set_running_footer(text) {
        this.setRunningFooter(text);
      }
      search(query) {
        return JSON.parse(call(() => this._ed.search(query)));
      }
      suggestions() {
        return JSON.parse(call(() => this._ed.suggestions()));
      }
      suggest(id, text) {
        call(() => this._ed.suggest(id, text));
      }
      acceptSuggestion(id) {
        call(() => this._ed.accept_suggestion(id));
      }
      accept_suggestion(id) {
        this.acceptSuggestion(id);
      }
      rejectSuggestion(id) {
        call(() => this._ed.reject_suggestion(id));
      }
      reject_suggestion(id) {
        this.rejectSuggestion(id);
      }
      save() {
        return call(() => this._ed.save());
      }
      saveWith(expectedContentHash) {
        return call(() => this._ed.save_with(expectedContentHash ?? null));
      }
      exportPptx() {
        return call(() => this._ed.export_pptx());
      }
      export_pptx() {
        return this.exportPptx();
      }
      exportDocx() {
        return call(() => this._ed.export_docx());
      }
      export_docx() {
        return this.exportDocx();
      }
      free() {
        this._ed.free();
      }
    },
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
    generateSigningKey() {
      return JSON.parse(call(() => wasm.generate_signing_key()));
    },
    sign(bytes, secretHex, signedBy, signedAt) {
      return call(() => wasm.sign_k2f(bytes, secretHex, signedBy, signedAt));
    },
    markdownToK2f(md, { title = "Document", templateBytes } = {}) {
      if (!templateBytes) {
        throw agentError(
          new Error(
            "markdownToK2f requires templateBytes (a packed .K2F used as the shell)",
          ),
        );
      }
      return call(() => wasm.markdown_to_k2f(md, title, templateBytes));
    },
    k2fToMarkdown(bytes) {
      return call(() => wasm.k2f_to_markdown(bytes));
    },
  };
}
