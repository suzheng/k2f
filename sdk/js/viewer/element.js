import { errorMessage } from "../core/errors.js";
import { mountK2fViewer } from "./mount.js";
import { mountErrorShell } from "./open-error.js";

const HtmlElement =
  typeof HTMLElement === "undefined" ? class {} : HTMLElement;

export class K2fViewerElement extends HtmlElement {
  static get observedAttributes() {
    return ["src"];
  }

  #seq = 0;

  get handle() {
    return this._handle;
  }

  attributeChangedCallback(name, prev, next) {
    if (name === "src" && this.isConnected && prev !== next) this.#load();
  }

  connectedCallback() {
    this.#load();
  }

  disconnectedCallback() {
    this.#seq += 1;
    this.#destroy();
  }

  async open(bytes) {
    const seq = ++this.#seq;
    await this.#mount(seq, bytes);
  }

  #options() {
    return {
      editable: this.hasAttribute("editable"),
      banner: this.hasAttribute("no-banner") ? "off" : "auto",
    };
  }

  #destroy() {
    this._handle?.destroy();
    this._handle = undefined;
  }

  async #load() {
    const src = this.getAttribute("src");
    if (!src) return;
    const seq = ++this.#seq;
    const res = await fetch(src);
    if (seq !== this.#seq) return;
    if (!res.ok) {
      this.#destroy();
      mountErrorShell(this, `Cannot fetch ${src} (${res.status})`, this.#options());
      return;
    }
    const buf = await res.arrayBuffer();
    if (seq !== this.#seq) return;
    await this.#mount(seq, new Uint8Array(buf));
  }

  async #mount(seq, bytes) {
    this.#destroy();
    try {
      const handle = await mountK2fViewer(this, bytes, this.#options());
      if (seq !== this.#seq) {
        handle.destroy();
        return;
      }
      this._handle = handle;
    } catch (err) {
      if (seq !== this.#seq) return;
      this._handle = undefined;
      if (!this.shadowRoot || !this.shadowRoot.querySelector(".k2f-banner")) {
        mountErrorShell(this, errorMessage(err), this.#options());
      }
    }
  }
}

export function defineK2fViewer() {
  if (typeof customElements === "undefined") return;
  if (!customElements.get("k2f-viewer")) {
    customElements.define("k2f-viewer", K2fViewerElement);
  }
}
