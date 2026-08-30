/** Transparent selectable spans over the engine PNG. Positions come from `text_layer`. */

export function buildTextLayer(pageWrap, viewer, page, zoom) {
  pageWrap.querySelector(".k2f-text-layer")?.remove();
  if (!viewer || typeof viewer.text_layer !== "function") return null;
  const spans = JSON.parse(viewer.text_layer(page));
  const layer = document.createElement("div");
  layer.className = "k2f-text-layer";
  pageWrap.appendChild(layer);
  for (const s of spans) {
    if (!s.text) continue;
    const el = document.createElement("span");
    el.textContent = s.text;
    const w = s.width_pt * zoom;
    const h = s.height_pt * zoom;
    el.style.left = `${s.x_pt * zoom}px`;
    el.style.top = `${s.y_pt * zoom}px`;
    el.style.fontSize = `${h}px`;
    el.dataset.nodeId = s.node_id;
    el.dataset.charStart = String(s.char_start);
    el.dataset.charEnd = String(s.char_end);
    layer.appendChild(el);
    layer.appendChild(document.createElement("br"));
    fitSpanWidth(el, w);
  }
  return layer;
}

/** Scale glyph-run text to the lock box. Transform does not change layout; measure first. */
function fitSpanWidth(el, widthPx) {
  const natural = el.getBoundingClientRect().width;
  if (!(natural > 0) || !(widthPx > 0)) return;
  el.style.transform = `scaleX(${widthPx / natural})`;
}
