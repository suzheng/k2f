/** Vertical stack of lock page sheets (Word-style continuous scroll). */

import { layoutSheet, paintSheet } from "./paint-page.js";
import { quantizePaintScale, neededPaintScale } from "./display-scale.js";

export function createStack({ stackEl, empty, Viewer }) {
  const cache = new Map();
  let sheets = [];

  function revoke() {
    for (const s of sheets) {
      if (s.img?._k2fUrl) {
        URL.revokeObjectURL(s.img._k2fUrl);
        s.img._k2fUrl = null;
      }
    }
  }

  function clear() {
    revoke();
    cache.clear();
    stackEl.replaceChildren();
    sheets = [];
  }

  function showEmpty(text) {
    empty.hidden = false;
    empty.textContent = text;
    stackEl.hidden = true;
    clear();
  }

  function sheetEls(page) {
    const wrap = document.createElement("div");
    wrap.className = "k2f-page-wrap";
    wrap.dataset.page = String(page);
    const img = document.createElement("img");
    img.className = "k2f-page";
    img.alt = `K2F page ${page + 1}`;
    img.draggable = false;
    const highlight = document.createElement("div");
    highlight.className = "k2f-highlight";
    highlight.hidden = true;
    wrap.append(img, highlight);
    return { wrap, img, highlight, page };
  }

  function officialScale() {
    return typeof Viewer.official_scale === "function" ? Viewer.official_scale() : 2;
  }

  function sheetVisible(wrap, root) {
    const rootEl = root || wrap.closest(".k2f-stage") || document.documentElement;
    const rr = rootEl.getBoundingClientRect?.() ?? {
      top: 0,
      bottom: window.innerHeight,
    };
    const r = wrap.getBoundingClientRect();
    return r.bottom >= rr.top - 64 && r.top <= rr.bottom + 64;
  }

  return {
    build(viewer, zoom) {
      clear();
      if (!viewer || viewer.page_count() === 0) {
        showEmpty(
          viewer && viewer.banner() === "UNLOCKED"
            ? "Unlocked draft: no published lock to paint. Compile is a publish step, not an open step."
            : "No published lock to paint.",
        );
        return;
      }
      empty.hidden = true;
      stackEl.hidden = false;
      const n = viewer.page_count();
      const baseline = officialScale();
      for (let page = 0; page < n; page++) {
        const sheet = sheetEls(page);
        stackEl.append(sheet.wrap);
        paintSheet({
          ...sheet,
          viewer,
          zoom,
          scale: baseline,
          cache,
          Viewer,
        });
        sheets.push(sheet);
      }
    },
    layout(viewer, zoom) {
      for (const s of sheets) layoutSheet(s.wrap, s.img, viewer, s.page, zoom);
    },
    /** Upgrade visible sheets to quantized display paint scale. */
    ensureDisplay(viewer, zoom, dpr = 1) {
      if (!viewer || sheets.length === 0) return;
      const bucket = quantizePaintScale(neededPaintScale(zoom, dpr));
      const stage = stackEl.closest(".k2f-stage");
      for (const s of sheets) {
        if (!sheetVisible(s.wrap, stage)) continue;
        const cur = Number(s.img.dataset.paintScale);
        if (Number.isFinite(cur) && Math.abs(cur - bucket) < 1e-6) continue;
        paintSheet({
          ...s,
          viewer,
          zoom,
          scale: bucket,
          cache,
          Viewer,
        });
      }
    },
    wraps: () => sheets.map((s) => s.wrap),
    highlightOf(page) {
      return sheets[page]?.highlight ?? null;
    },
    showEmpty,
    clear,
    dispose() {
      clear();
    },
  };
}
