const NS = "http://www.w3.org/2000/svg";

function svg(children, size = 20) {
  const el = document.createElementNS(NS, "svg");
  el.setAttribute("width", String(size));
  el.setAttribute("height", String(size));
  el.setAttribute("viewBox", "0 0 24 24");
  el.setAttribute("fill", "none");
  el.setAttribute("stroke", "currentColor");
  el.setAttribute("stroke-width", "2");
  el.setAttribute("stroke-linecap", "round");
  el.setAttribute("stroke-linejoin", "round");
  el.setAttribute("aria-hidden", "true");
  for (const child of children) el.append(child);
  return el;
}

function path(d) {
  const el = document.createElementNS(NS, "path");
  el.setAttribute("d", d);
  return el;
}

function circle(cx, cy, r, fill = false) {
  const el = document.createElementNS(NS, "circle");
  el.setAttribute("cx", String(cx));
  el.setAttribute("cy", String(cy));
  el.setAttribute("r", String(r));
  if (fill) {
    el.setAttribute("fill", "currentColor");
    el.setAttribute("stroke", "none");
  }
  return el;
}

export function iconChevronLeft() {
  return svg([path("M15 18l-6-6 6-6")]);
}

export function iconChevronRight() {
  return svg([path("M9 18l6-6-6-6")]);
}

export function iconMinus() {
  return svg([path("M5 12h14")]);
}

export function iconPlus() {
  return svg([path("M12 5v14M5 12h14")]);
}

export function iconSun() {
  return svg([
    circle(12, 12, 4),
    path("M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"),
  ]);
}

export function iconMoon() {
  return svg([path("M21 14.5A8.5 8.5 0 1 1 9.5 3 7 7 0 0 0 21 14.5z")]);
}

export function iconMore() {
  return svg([circle(6, 12, 1.4, true), circle(12, 12, 1.4, true), circle(18, 12, 1.4, true)]);
}

export function iconCaret() {
  return svg([path("M6 9l6 6 6-6")], 16);
}

export function iconExport() {
  return svg([path("M12 3v12M8 11l4 4 4-4M5 21h14")]);
}
