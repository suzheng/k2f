import assert from "node:assert/strict";
import { boxToCss } from "../viewer/coords.js";
import { bindFormFill, readFormFields } from "../viewer/form-fill.js";
import { buildFormLayer, fieldsOnPage } from "../viewer/form-layer.js";
import { installMiniDom } from "./helpers/mini-dom.mjs";

installMiniDom();

const fields = [
  {
    id: "ex.form.name",
    page: 0,
    x: 72000,
    y: 100000,
    width: 220000,
    height: 19400,
    kind: "text",
    value: "",
    placeholder: "Full legal name",
    required: false,
  },
  {
    id: "ex.form.address",
    page: 0,
    x: 72000,
    y: 130000,
    width: 400000,
    height: 65600,
    kind: "multiline",
    value: "",
    placeholder: "Street address",
    required: false,
    max_length: 200,
  },
  {
    id: "ex.form.agree",
    page: 0,
    x: 72000,
    y: 210000,
    width: 11000,
    height: 11000,
    kind: "checkbox",
    value: "",
    required: true,
  },
  {
    id: "ex.form.sign.name",
    page: 1,
    x: 72000,
    y: 40000,
    width: 180000,
    height: 19400,
    kind: "text",
    value: "Alice",
    placeholder: "Signature name",
    required: false,
  },
];

{
  const wrap = document.createElement("div");
  wrap.dataset.page = "0";
  const layer = buildFormLayer(wrap, fieldsOnPage(fields, 0), 1, { values: new Map() });
  assert.ok(layer);
  assert.equal(layer.className, "k2f-form-layer");
  const controls = layer.querySelectorAll("[data-field-id]");
  assert.equal(controls.length, 3);
  const name = controls.find((el) => el.dataset.fieldId === "ex.form.name");
  const address = controls.find((el) => el.dataset.fieldId === "ex.form.address");
  const agree = controls.find((el) => el.dataset.fieldId === "ex.form.agree");
  assert.equal(name.tagName, "INPUT");
  assert.equal(name.type, "text");
  assert.equal(name.placeholder, "Full legal name");
  assert.equal(address.tagName, "TEXTAREA");
  assert.equal(address.maxLength, 200);
  assert.equal(agree.tagName, "INPUT");
  assert.equal(agree.type, "checkbox");
  const css = boxToCss(fields[0], 1);
  assert.equal(name.style.left, `${css.left + 2}px`);
  assert.equal(name.style.top, `${css.top + 2}px`);
  const wrap2 = document.createElement("div");
  wrap2.dataset.page = "1";
  const page1 = buildFormLayer(wrap2, fieldsOnPage(fields, 1), 1, {
    values: new Map([["ex.form.sign.name", "Alice"]]),
  });
  assert.equal(page1.querySelectorAll("[data-field-id]").length, 1);
  assert.equal(page1.querySelectorAll("[data-field-id]")[0].value, "Alice");
}

{
  const wrap = document.createElement("div");
  buildFormLayer(wrap, fieldsOnPage(fields, 0), 1, { values: new Map(), readOnly: true });
  assert.equal(wrap.querySelector(".k2f-form-layer").dataset.readonly, "true");
  const empty = document.createElement("div");
  assert.equal(buildFormLayer(empty, [], 1), null);
  assert.equal(empty.querySelector(".k2f-form-layer"), null);
}

{
  const wrap = document.createElement("div");
  wrap.dataset.page = "0";
  const seen = [];
  buildFormLayer(wrap, fieldsOnPage(fields, 0), 1, {
    values: new Map(),
    onChange: (id, value) => seen.push([id, value]),
  });
  const name = wrap.querySelectorAll("[data-field-id]").find((el) => el.dataset.fieldId === "ex.form.name");
  name.value = "张三";
  name.emit("input");
  const agree = wrap.querySelectorAll("[data-field-id]").find((el) => el.dataset.fieldId === "ex.form.agree");
  agree.checked = true;
  agree.emit("change");
  assert.deepEqual(seen, [
    ["ex.form.name", "张三"],
    ["ex.form.agree", "true"],
  ]);
}

{
  assert.deepEqual(readFormFields({ form_fields: () => "[]" }), []);
  assert.deepEqual(readFormFields({}), []);
  const calls = [];
  const saveButton = {
    hidden: true,
    disabled: true,
    listeners: {},
    addEventListener(type, fn) {
      this.listeners[type] = fn;
    },
  };
  const editor = {
    replaceText(id, value) {
      calls.push(["replace", id, value]);
    },
    save() {
      calls.push(["save"]);
      return new Uint8Array([1, 2]);
    },
  };
  const fill = bindFormFill({
    wrapsOf: () => [],
    viewerOf: () => ({
      form_fields: () => JSON.stringify(fields),
    }),
    editorOf: () => editor,
    zoomOf: () => 1,
    editingOf: () => true,
    saveButton,
    onRelock: async (bytes) => {
      calls.push(["relock", bytes.length]);
    },
  });
  fill.set("ex.form.name", "张三");
  fill.set("ex.form.agree", "true");
  assert.equal(saveButton.hidden, false);
  assert.equal(saveButton.disabled, false);
  await fill.save();
  assert.deepEqual(calls, [
    ["replace", "ex.form.name", "张三"],
    ["replace", "ex.form.agree", "true"],
    ["save"],
    ["relock", 2],
  ]);
}

console.log("ok form-layer overlay DOM + batched Save");
