import { boxToCss } from "./coords.js";

/** Role, Variant, and Copy node stay in the save and clipboard path. */
export const SHOW_META_FIELDS = false;

/** On-page editor anchored to the hit box. Relock happens in `onRelock`. */

export function bindPopover({
  wrapsOf,
  highlightOf,
  editingOf,
  viewerOf,
  editorOf,
  zoomOf,
  highlight,
  root,
  onRelock,
  onError,
}) {
  let selectedId = null;
  let textDisabled = true;
  let originalText = "";
  const pop = document.createElement("div");
  pop.className = "k2f-popover";
  pop.hidden = true;
  pop.addEventListener("click", (e) => e.stopPropagation());
  pop.addEventListener("mousedown", (e) => e.stopPropagation());

  const text = bareTextarea();
  const role = field("Role", "input");
  const variant = field("Variant", "input");
  const cancel = button("Cancel", "cancel");
  const save = button("Save and relock", "save");
  const copy = button("Copy node", "copy");
  const actions = document.createElement("div");
  actions.className = "k2f-popover-actions";
  actions.append(cancel, save);
  save.disabled = true;
  copy.disabled = true;
  const parts = [text.wrap];
  if (SHOW_META_FIELDS) parts.push(role.wrap, variant.wrap);
  parts.push(actions);
  if (SHOW_META_FIELDS) parts.push(copy);
  pop.append(...parts);

  const prompt = discardPrompt(root);

  function closePrompt() {
    prompt.backdrop.hidden = true;
  }

  function clear() {
    closePrompt();
    selectedId = null;
    textDisabled = true;
    originalText = "";
    pop.hidden = true;
    pop.remove();
    save.disabled = true;
    copy.disabled = true;
    highlight.hide();
  }

  function show(sel) {
    if (!sel || !sel.id || !editingOf()) return;
    closePrompt();
    selectedId = sel.id;
    textDisabled = sel.text == null;
    originalText = sel.text || "";
    role.input.value = sel.role || "";
    variant.input.value = sel.variant || "";
    text.input.value = originalText;
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

  function textDirty() {
    return !textDisabled && text.input.value !== originalText;
  }

  function cancelEdit() {
    if (textDirty()) {
      prompt.backdrop.hidden = false;
      return;
    }
    clear();
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

  cancel.addEventListener("click", () => cancelEdit());
  save.addEventListener("click", () => saveEdit());
  copy.addEventListener("click", () => copyNode());
  prompt.keep.addEventListener("click", () => closePrompt());
  prompt.discard.addEventListener("click", () => clear());
  prompt.backdrop.addEventListener("click", (e) => {
    if (e.target === prompt.backdrop) closePrompt();
  });

  return { id: () => selectedId, clear, show, save: saveEdit, copy: copyNode, el: pop };
}

function discardPrompt(root) {
  const backdrop = document.createElement("div");
  backdrop.className = "k2f-dialog-backdrop";
  backdrop.hidden = true;

  const dialog = document.createElement("div");
  dialog.className = "k2f-dialog";
  dialog.setAttribute("role", "dialog");
  dialog.setAttribute("aria-modal", "true");
  dialog.addEventListener("click", (e) => e.stopPropagation());

  const title = document.createElement("h2");
  title.className = "k2f-dialog-title";
  title.textContent = "Discard unsaved changes?";

  const actions = document.createElement("div");
  actions.className = "k2f-dialog-actions";
  const keep = button("Keep editing", "cancel");
  const discard = button("Discard", "confirm");
  actions.append(keep, discard);
  dialog.append(title, actions);
  backdrop.append(dialog);
  root.append(backdrop);
  return { backdrop, keep, discard };
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
