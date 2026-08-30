import assert from "node:assert/strict";
import { bindEditMode } from "../viewer/edit-mode.js";

function fakeButton() {
  const el = {
    hidden: false,
    disabled: false,
    textContent: "",
    attrs: {},
    listeners: {},
    setAttribute(name, value) {
      this.attrs[name] = value;
    },
    addEventListener(type, fn, opts) {
      this.listeners[type] = fn;
      if (opts?.signal) {
        opts.signal.addEventListener("abort", () => {
          delete this.listeners[type];
        });
      }
    },
    click() {
      this.listeners.click?.();
    },
  };
  return el;
}

{
  const btn = fakeButton();
  const mode = bindEditMode({ button: btn, canEdit: true });
  assert.equal(mode.editing(), false);
  assert.equal(btn.textContent, "Edit");
  btn.click();
  assert.equal(mode.editing(), true);
  assert.equal(btn.textContent, "Done");
  assert.equal(btn.attrs["aria-pressed"], "true");
  mode.setEditing(false);
  assert.equal(mode.editing(), false);
  assert.equal(btn.textContent, "Edit");
}

{
  const btn = fakeButton();
  bindEditMode({ button: btn, canEdit: false });
  assert.equal(btn.hidden, true);
  assert.equal(btn.disabled, true);
}

console.log("ok edit-mode");
