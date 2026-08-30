/** CSS pixels on the painted page → lock pt. Official geometry is pt, not the 2× raster. */
export function eventToPt(event, pageEl, zoom) {
  const r = pageEl.getBoundingClientRect();
  return {
    x: (event.clientX - r.left) / zoom,
    y: (event.clientY - r.top) / zoom,
  };
}

export function boxToCss(box, zoom) {
  return {
    left: (box.x / 1000) * zoom,
    top: (box.y / 1000) * zoom,
    width: (box.width / 1000) * zoom,
    height: (box.height / 1000) * zoom,
  };
}
