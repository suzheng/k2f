import { agentError, call } from "./errors.js";

/** Thin JS/TS facade over the WASM SDK. Geometry stays in the engine. */
export function wrapWasm(wasm) {
  return {
    systemPrompt: () => call(() => wasm.system_prompt()),
    officialTemplates: () => {
      const raw = call(() => wasm.official_templates());
      return typeof raw === "string" ? JSON.parse(raw) : raw;
    },
    resolveTemplate: (template) => call(() => wasm.resolve_template(template)),
    copyTemplate: (template, dest) =>
      call(() => wasm.copy_template(template, dest)),
    Editor: class {
      static open(bytes) {
        const e = Object.create(this.prototype);
        e._ed = call(() => new wasm.K2fEditor(bytes));
        return e;
      }
      static openTemplate(template) {
        const e = Object.create(this.prototype);
        e._ed = call(() => wasm.K2fEditor.openTemplate(template));
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
    },
    generateSigningKey() {
      return JSON.parse(call(() => wasm.generate_signing_key()));
    },
    sign(bytes, secretHex, signedBy, signedAt) {
      return call(() => wasm.sign_k2f(bytes, secretHex, signedBy, signedAt));
    },
    markdownToK2f(md, { title = "Document", template = "report" } = {}) {
      return call(() => wasm.markdown_to_k2f(md, title, template));
    },
    k2fToMarkdown(bytes) {
      return call(() => wasm.k2f_to_markdown(bytes));
    },
  };
}
