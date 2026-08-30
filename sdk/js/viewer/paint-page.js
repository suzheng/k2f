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

export function paintSheet({ wrap, img, viewer, page, zoom, cache, Viewer }) {
  const scale = Viewer.official_scale();
  const key = `${page}:${scale}`;
  let bytes = cache.get(key);
  if (!bytes) {
    bytes = viewer.render_page(page, scale);
    cache.set(key, bytes);
  }
  const url = URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
  img.src = url;
  layoutSheet(wrap, img, viewer, page, zoom);
  return url;
}
