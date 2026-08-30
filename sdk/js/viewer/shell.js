import { VIEWER_CSS } from "./styles.js";
import { EXPORT_FORMATS } from "./export-format.js";

export function attachMountRoot(host) {
  if (host.shadowRoot) return host.shadowRoot;
  try {
    return host.attachShadow({ mode: "open" });
  } catch {
    return host;
  }
}

export function createViewerShell({ banner }) {
  const root = document.createElement("div");
  root.className = "k2f-root";
  const style = document.createElement("style");
  style.textContent = VIEWER_CSS;
  root.append(style);

  const bannerEl = document.createElement("header");
  bannerEl.className = "k2f-banner banner-idle";
  bannerEl.dataset.state = "IDLE";
  bannerEl.textContent =
    "Locked pages are painted by the engine, not by the browser.";
  if (banner === false) bannerEl.hidden = true;
  root.append(bannerEl);

  const toolbar = document.createElement("nav");
  toolbar.className = "k2f-toolbar";
  const prev = button("prev", "Previous");
  const next = button("next", "Next");
  const pageLabel = document.createElement("span");
  pageLabel.textContent = "—";
  const zoomOut = button("zoom-out", "−");
  const zoomIn = button("zoom-in", "+");
  const zoomLabel = document.createElement("span");
  zoomLabel.textContent = "100%";
  const editBtn = button("edit", "Edit");
  editBtn.hidden = true;
  const exportFormat = document.createElement("select");
  exportFormat.className = "k2f-export-format";
  exportFormat.title = "Export format";
  exportFormat.setAttribute("aria-label", "Export format");
  for (const { value, label } of EXPORT_FORMATS) {
    const opt = document.createElement("option");
    opt.value = value;
    opt.textContent = label;
    exportFormat.append(opt);
  }
  const exportBtn = button("export", "Export");
  const fullscreen = button("fullscreen", "Fullscreen");
  const copyFormat = document.createElement("select");
  copyFormat.className = "k2f-copy-format";
  copyFormat.title = "Clipboard text format";
  copyFormat.setAttribute("aria-label", "Copy format");
  for (const [value, label] of [
    ["markdown", "Copy: Markdown"],
    ["plain", "Copy: Plain"],
  ]) {
    const opt = document.createElement("option");
    opt.value = value;
    opt.textContent = label;
    copyFormat.append(opt);
  }
  prev.disabled = true;
  next.disabled = true;
  zoomOut.disabled = true;
  zoomIn.disabled = true;
  exportBtn.disabled = true;
  toolbar.append(
    prev,
    pageLabel,
    next,
    zoomOut,
    zoomLabel,
    zoomIn,
    editBtn,
    exportFormat,
    exportBtn,
    fullscreen,
    copyFormat,
  );
  root.append(toolbar);

  const stage = document.createElement("main");
  stage.className = "k2f-stage";
  const empty = document.createElement("p");
  empty.className = "k2f-empty";
  empty.textContent = "No document loaded.";
  const stack = document.createElement("div");
  stack.className = "k2f-stack";
  stage.append(empty, stack);
  root.append(stage);

  return {
    root,
    banner: bannerEl,
    prev,
    next,
    pageLabel,
    zoomIn,
    zoomOut,
    zoomLabel,
    editBtn,
    exportFormat,
    exportBtn,
    fullscreen,
    copyFormat,
    empty,
    stage,
    stack,
  };
}

function button(act, label) {
  const el = document.createElement("button");
  el.type = "button";
  el.dataset.act = act;
  el.textContent = label;
  return el;
}
