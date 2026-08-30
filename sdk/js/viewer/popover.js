import { boxToCss } from "./coords.js";

/** On-page editor anchored to the hit box. Relock happens in `onRelock`. */

export function bindPopover({
  wrapsOf,
  highlightOf,
  editingOf,
  viewerOf,
  editorOf,
  zoomOf,
  highlight,
  onRelock,
  onError,
}) {
  let selectedId = null;
  let textDisabled = true;
  const pop = document.createElement("div");
  pop.className = "k2f-popover";
  pop.hidden = true;
  pop.addEventListener("click", (e) => e.stopPropagation());
  pop.addEventListener("mousedown", (e) => e.stopPropagation());

  const text = bareTextarea();
  const role = field("Role", "input");
  const variant = field("Variant", "input");
  const save = button("Save and relock", "save");
  const copy = button("Copy node", "copy");
  save.disabled = true;
  copy.disabled = true;
  pop.append(text.wrap, role.wrap, variant.wrap, save, copy);

  function clear() {
    selectedId = null;
    textDisabled = true;
    pop.hidden = true;
    pop.remove();
    save.disabled = true;
    copy.disabled = true;
    highlight.hide();
  }

  function show(sel) {
    if (!sel || !sel.id || !editingOf()) return;
    selectedId = sel.id;
    textDisabled = sel.text == null;
    role.input.value = sel.role || "";
    variant.input.value = sel.variant || "";
    text.input.value = sel.text || "";
    const editing = editingOf();
    role.input.disabled = !editing;
    variant.input.disabled = !editing;
    text.input.disabled = !editing || textDisabled;
    text.wrap.hidden = sel.text == null;
    save.disabled = !editing;
    copy.disabled = false;
    pop.hidden = false;
    const viewer = viewerOf();
    if (!viewer) return;
    const boxes = JSON.parse(viewer.boxes_for(sel.id));
    if (!boxes.length) {
      highlight.hide();
      return;
    }
    const box = boxes[0];
    const wrap = wrapsOf()[box.page];
    if (!wrap) {
      highlight.hide();
      return;
    }
    wrap.append(pop);
    const css = boxToCss(box, zoomOf());
    highlight.show(highlightOf(box.page), css);
    pop.style.left = `${css.left}px`;
    pop.style.top = `${css.top + css.height + 6}px`;
  }

  async function saveEdit() {
    const ed = editorOf();
    if (!ed || !selectedId) return;
    try {
      const nextRole = role.input.value.trim();
      if (nextRole) ed.setRole(selectedId, nextRole, variant.input.value.trim() || undefined);
      if (!textDisabled) ed.replaceText(selectedId, text.input.value);
      const bytes = ed.save();
      await onRelock(bytes, selectedId);
    } catch (err) {
      onError?.(err);
    }
  }

  async function copyNode() {
    const viewer = viewerOf();
    if (!viewer || !selectedId) return;
    const raw = viewer.clipboard(selectedId);
    if (!raw || !navigator.clipboard) return;
    try {
      await navigator.clipboard.writeText(raw);
    } catch (err) {
      onError?.(err);
    }
  }

  save.addEventListener("click", () => saveEdit());
  copy.addEventListener("click", () => copyNode());

  return { id: () => selectedId, clear, show, save: saveEdit, copy: copyNode, el: pop };
}

function bareTextarea() {
  const wrap = document.createElement("div");
  wrap.className = "k2f-popover-text";
  const input = document.createElement("textarea");
  input.rows = 3;
  wrap.append(input);
  return { wrap, input };
}

function field(label, kind) {
  const wrap = document.createElement("label");
  wrap.append(label);
  const input = kind === "textarea" ? document.createElement("textarea") : document.createElement("input");
  if (kind !== "textarea") input.type = "text";
  else input.rows = 3;
  wrap.append(input);
  return { wrap, input };
}

function button(label, act) {
  const el = document.createElement("button");
  el.type = "button";
  el.dataset.act = act;
  el.textContent = label;
  return el;
}
