/** Click opens the popover in edit mode; a drag leaves native text selection alone. */

export function bindDragFlag(el, signal) {
  let origin = null;
  let dragged = false;
  el.addEventListener(
    "mousedown",
    (e) => {
      if (e.button !== 0) return;
      origin = { x: e.clientX, y: e.clientY };
      dragged = false;
    },
    { signal },
  );
  el.addEventListener(
    "mousemove",
    (e) => {
      if (!origin) return;
      if (Math.hypot(e.clientX - origin.x, e.clientY - origin.y) > 4) dragged = true;
    },
    { signal },
  );
  el.addEventListener(
    "mouseup",
    () => {
      origin = null;
    },
    { signal },
  );
  return () => dragged;
}
