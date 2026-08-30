/** Vertical stack of lock page sheets (Word-style continuous scroll). */

import { layoutSheet, paintSheet } from "./paint-page.js";

export function createStack({ stackEl, empty, Viewer }) {
  const cache = new Map();
  const urls = [];
  let sheets = [];

  function revoke() {
    for (const url of urls) URL.revokeObjectURL(url);
    urls.length = 0;
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
      for (let page = 0; page < n; page++) {
        const sheet = sheetEls(page);
        stackEl.append(sheet.wrap);
        urls.push(paintSheet({ ...sheet, viewer, zoom, cache, Viewer }));
        sheets.push(sheet);
      }
    },
    layout(viewer, zoom) {
      for (const s of sheets) layoutSheet(s.wrap, s.img, viewer, s.page, zoom);
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
