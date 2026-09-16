import { buildFormLayer, fieldsOnPage, removeFormLayer } from "./form-layer.js";

/** Overlay session: dirty Map stays in JS until one Save batches replaceText + relock. */

export function readFormFields(viewer) {
  if (!viewer || typeof viewer.form_fields !== "function") return [];
  try {
    const list = JSON.parse(viewer.form_fields());
    return Array.isArray(list) ? list : [];
  } catch {
    return [];
  }
}

export function bindFormFill({
  wrapsOf,
  viewerOf,
  editorOf,
  zoomOf,
  editingOf,
  saveButton,
  onRelock,
  onError,
  signal,
}) {
  const values = new Map();
  const dirty = new Map();
  let focusedId = null;

  saveButton?.addEventListener("click", () => save(), { signal });

  function fields() {
    return readFormFields(viewerOf());
  }

  function isField(id) {
    return Boolean(id) && fields().some((f) => f.id === id);
  }

  function set(id, value) {
    const next = String(value);
    values.set(id, next);
    dirty.set(id, next);
    syncSaveButton();
  }

  function resetFromPackage() {
    values.clear();
    dirty.clear();
    focusedId = null;
    for (const f of fields()) values.set(f.id, f.value ?? "");
    syncSaveButton();
  }

  function syncSaveButton() {
    if (!saveButton) return;
    const editing = Boolean(editingOf());
    const n = fields().length;
    saveButton.hidden = !editing || n === 0;
    saveButton.disabled = !editing || dirty.size === 0;
  }

  function sync() {
    const wraps = wrapsOf() ?? [];
    const editing = Boolean(editingOf());
    if (!editing) {
      values.clear();
      dirty.clear();
      focusedId = null;
      for (const wrap of wraps) removeFormLayer(wrap);
      syncSaveButton();
      return;
    }
    if (values.size === 0) resetFromPackage();
    else syncSaveButton();
    const list = fields();
    const zoom = zoomOf();
    for (const wrap of wraps) {
      const page = Number(wrap.dataset.page);
      buildFormLayer(wrap, fieldsOnPage(list, page), zoom, {
        values,
        onChange: set,
        onFocus: (id) => {
          focusedId = id;
        },
        readOnly: false,
      });
    }
    restoreFocus(wraps);
  }

  function restoreFocus(wraps) {
    if (!focusedId) return;
    const el = controlById(wraps, focusedId);
    el?.focus?.();
  }

  function focus(id) {
    focusedId = id;
    const el = controlById(wrapsOf() ?? [], id);
    el?.focus?.();
  }

  async function save() {
    const ed = editorOf();
    if (!ed || dirty.size === 0) return;
    try {
      for (const [id, value] of dirty) ed.replaceText(id, value);
      const bytes = ed.save();
      await onRelock?.(bytes);
      dirty.clear();
      syncSaveButton();
    } catch (err) {
      onError?.(err);
    }
  }

  function clear() {
    values.clear();
    dirty.clear();
    focusedId = null;
    for (const wrap of wrapsOf() ?? []) removeFormLayer(wrap);
    syncSaveButton();
  }

  return { sync, clear, isField, focus, save, set };
}

function controlById(wraps, id) {
  for (const wrap of wraps) {
    const layer = wrap.querySelector(".k2f-form-layer");
    if (!layer) continue;
    for (const el of layer.querySelectorAll("[data-field-id]")) {
      if (el.dataset.fieldId === id) return el;
    }
  }
  return null;
}
