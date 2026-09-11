/** Paint one lock PNG sheet. Does not recompile. */

import { buildTextLayer } from "./text-layer.js";

export function layoutSheet(wrap, img, viewer, page, zoom) {
  if (!viewer || viewer.page_count() === 0) return;
  const w = viewer.page_width_pt(page) * zoom;
  const h = viewer.page_height_pt(page) * zoom;
  img.style.width = `${w}px`;
  img.style.height = `${h}px`;
  wrap.style.width = `${w}px`;
  wrap.style.height = `${h}px`;
  buildTextLayer(wrap, viewer, page, zoom);
}

export function cacheKey(page, scale) {
  return `${page}:${scale}`;
}

export function paintSheet({ wrap, img, viewer, page, zoom, scale, cache, Viewer }) {
  const paintScale =
    scale ??
    (typeof Viewer.official_scale === "function" ? Viewer.official_scale() : 2);
  const key = cacheKey(page, paintScale);
  let bytes = cache.get(key);
  if (!bytes) {
    bytes = viewer.render_page(page, paintScale);
    cache.set(key, bytes);
  }
  if (img._k2fUrl) URL.revokeObjectURL(img._k2fUrl);
  const url = URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
  img._k2fUrl = url;
  img.dataset.paintScale = String(paintScale);
  img.src = url;
  layoutSheet(wrap, img, viewer, page, zoom);
  return url;
}
