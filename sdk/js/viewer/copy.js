/** Copy from the text layer: human `text/plain` plus optional K2F node JSON. */

import { normalizeCopyFormat } from "./copy-format.js";

export function bindCopy(root, { signal, viewerOf, copyFormatOf }) {
  const onCopy = (e) => {
    const payload = clipboardFromSelection(selectionOf(root));
    if (!payload || !e.clipboardData) return;
    const format = normalizeCopyFormat(copyFormatOf?.() ?? "markdown");
    let plain = payload.plain;
    if (format === "markdown") {
      const viewer = viewerOf?.();
      if (!viewer || typeof viewer.selection_markdown !== "function") return;
      try {
        plain = viewer.selection_markdown(JSON.stringify(payload.nodes));
      } catch {
        return;
      }
      if (!plain) return;
    }
    e.clipboardData.setData("text/plain", plain);
    e.clipboardData.setData("application/x-k2f-nodes+json", JSON.stringify(payload.nodes));
    e.preventDefault();
  };
  root.addEventListener("copy", onCopy, { signal });
  if (root !== document) document.addEventListener("copy", onCopy, { signal });
}

export function clipboardFromSelection(sel) {
  if (!sel || sel.isCollapsed) return null;
  const plain = sel.toString();
  if (!plain) return null;
  const nodes = collectK2fNodesFromSelection(sel);
  if (!nodes.length) return null;
  return { plain, nodes };
}

export function layersIn(root, range) {
  if (!root?.querySelectorAll || !range?.intersectsNode) return [];
  return [...root.querySelectorAll(".k2f-text-layer")].filter((el) =>
    range.intersectsNode(el),
  );
}

export function collectK2fNodesFromSelection(sel) {
  if (!sel || sel.rangeCount === 0) return [];
  const range = sel.getRangeAt(0);
  const layers = layersOf(range);
  if (!layers.length) return [];
  const byId = new Map();
  for (const layer of layers) {
    for (const el of layer.querySelectorAll("span[data-node-id]")) {
      if (!range.intersectsNode(el)) continue;
      const piece = spanSlice(el, range);
      if (!piece || !piece.text) continue;
      const prev = byId.get(piece.node_id);
      if (!prev) {
        byId.set(piece.node_id, piece);
        continue;
      }
      prev.char_start = Math.min(prev.char_start, piece.char_start);
      prev.char_end = Math.max(prev.char_end, piece.char_end);
      prev.text = `${prev.text}${piece.text}`;
    }
  }
  return [...byId.values()];
}

function selectionOf(root) {
  if (root && typeof root.getSelection === "function") {
    const sel = root.getSelection();
    if (sel) return sel;
  }
  return window.getSelection();
}

function layersOf(range) {
  const n = range.commonAncestorContainer;
  const el = n.nodeType === 1 ? n : n.parentElement;
  if (!el) return [];
  const root = el.closest?.(".k2f-stack") || el;
  const stacked = layersIn(root, range);
  if (stacked.length) return stacked;
  const one = el.closest?.(".k2f-text-layer");
  return one ? [one] : [];
}

function spanSlice(el, range) {
  const start = Number(el.dataset.charStart);
  const nodeId = el.dataset.nodeId;
  const contents = document.createRange();
  contents.selectNodeContents(el);
  const inter = range.cloneRange();
  if (inter.compareBoundaryPoints(Range.START_TO_START, contents) < 0) {
    inter.setStart(contents.startContainer, contents.startOffset);
  }
  if (inter.compareBoundaryPoints(Range.END_TO_END, contents) > 0) {
    inter.setEnd(contents.endContainer, contents.endOffset);
  }
  const selected = inter.toString();
  const prefix = document.createRange();
  prefix.selectNodeContents(el);
  prefix.setEnd(inter.startContainer, inter.startOffset);
  const from = start + charCount(prefix.toString());
  return {
    node_id: nodeId,
    char_start: from,
    char_end: from + charCount(selected),
    text: selected,
  };
}

function charCount(s) {
  return [...s].length;
}
