import assert from "node:assert/strict";
import { bindPopover, SHOW_META_FIELDS } from "../viewer/popover.js";
import { installMiniDom } from "./helpers/mini-dom.mjs";

installMiniDom();

assert.equal(SHOW_META_FIELDS, false);

function descendants(el, out = []) {
  for (const child of el.children) {
    out.push(child);
    descendants(child, out);
  }
  return out;
}

function buttons(el) {
  return descendants(el).filter((n) => n.tagName === "BUTTON");
}

function byText(el, text) {
  return descendants(el).find((n) => n.textContent === text);
}

function click(el, target = el) {
  return el.listeners.click?.({ target, stopPropagation() {} });
}

function harness(sel = {}) {
  const root = document.createElement("div");
  const wrap = document.createElement("div");
  const calls = [];
  let written = null;
  let editing = true;
  const prevNav = Object.getOwnPropertyDescriptor(globalThis, "navigator");
  Object.defineProperty(globalThis, "navigator", {
    configurable: true,
    value: {
      clipboard: {
        async writeText(value) {
          written = value;
        },
      },
    },
  });
  const edit = bindPopover({
    wrapsOf: () => [wrap],
    highlightOf: () => document.createElement("div"),
    editingOf: () => editing,
    viewerOf: () => ({
      boxes_for: () => JSON.stringify([{ page: 0, x: 0, y: 0, width: 1000, height: 1000 }]),
      clipboard: (id) => JSON.stringify({ id }),
    }),
    editorOf: () => ({
      setRole(id, role, variant) {
        calls.push(["setRole", id, role, variant]);
      },
      replaceText(id, value) {
        calls.push(["replaceText", id, value]);
      },
      save() {
        calls.push("save");
        return new Uint8Array([1]);
      },
    }),
    zoomOf: () => 1,
    highlight: { hide() {}, show() {} },
    root,
    onRelock: async () => {
      calls.push("relock");
    },
  });
  edit.show({
    id: "invoice.total",
    role: "total",
    variant: "due",
    text: "Hello",
    ...sel,
  });
  const backdrop = root.querySelector(".k2f-dialog-backdrop");
  const textarea = descendants(edit.el).find((n) => n.tagName === "TEXTAREA");
  return {
    root,
    edit,
    calls,
    backdrop,
    textarea,
    editing: () => editing,
    written: () => written,
    restore() {
      Object.defineProperty(globalThis, "navigator", prevNav);
    },
  };
}

{
  const h = harness();
  const labels = buttons(h.edit.el).map((n) => n.textContent);
  assert.deepEqual(labels, ["Cancel", "Save and relock"]);
  assert.equal(byText(h.edit.el, "Role"), undefined);
  assert.equal(byText(h.edit.el, "Variant"), undefined);
  assert.equal(byText(h.edit.el, "Copy node"), undefined);
  assert.equal(h.backdrop.hidden, true);
  assert.equal(h.editing(), true);
  h.restore();
}

{
  const h = harness();
  click(byText(h.edit.el, "Cancel"));
  assert.equal(h.backdrop.hidden, true);
  assert.equal(h.edit.id(), null);
  assert.equal(h.edit.el.hidden, true);
  assert.equal(h.editing(), true);
  assert.deepEqual(h.calls, []);
  h.restore();
}

{
  const h = harness();
  h.textarea.value = "Hello!";
  click(byText(h.edit.el, "Cancel"));
  assert.equal(h.backdrop.hidden, false);
  assert.equal(h.edit.id(), "invoice.total");
  assert.equal(byText(h.backdrop, "Discard unsaved changes?").textContent, "Discard unsaved changes?");
  click(byText(h.backdrop, "Keep editing"));
  assert.equal(h.backdrop.hidden, true);
  assert.equal(h.edit.id(), "invoice.total");
  assert.equal(h.textarea.value, "Hello!");
  assert.equal(h.editing(), true);

  h.textarea.value = "Hello?";
  click(byText(h.edit.el, "Cancel"));
  click(h.backdrop);
  assert.equal(h.backdrop.hidden, true);
  assert.equal(h.edit.id(), "invoice.total");
  assert.equal(h.textarea.value, "Hello?");

  click(byText(h.edit.el, "Cancel"));
  const dialog = h.backdrop.querySelector(".k2f-dialog");
  click(h.backdrop, dialog);
  assert.equal(h.backdrop.hidden, false);

  click(byText(h.backdrop, "Discard"));
  assert.equal(h.backdrop.hidden, true);
  assert.equal(h.edit.id(), null);
  assert.equal(h.editing(), true);
  assert.deepEqual(h.calls, []);
  h.restore();
}

{
  const h = harness();
  h.textarea.value = "Hello Z";
  await click(byText(h.edit.el, "Save and relock"));
  assert.deepEqual(h.calls, [
    ["setRole", "invoice.total", "total", "due"],
    ["replaceText", "invoice.total", "Hello Z"],
    "save",
    "relock",
  ]);
  h.restore();
}

{
  const h = harness();
  await h.edit.copy();
  assert.equal(h.written(), JSON.stringify({ id: "invoice.total" }));
  assert.equal(h.edit.id(), "invoice.total");
  assert.equal(h.editing(), true);
  h.restore();
}

{
  const h = harness({ text: null });
  const textWrap = descendants(h.edit.el).find((n) => n.className === "k2f-popover-text");
  assert.equal(textWrap.hidden, true);
  click(byText(h.edit.el, "Cancel"));
  assert.equal(h.backdrop.hidden, true);
  assert.equal(h.edit.id(), null);
  h.restore();
}

console.log("ok popover");
