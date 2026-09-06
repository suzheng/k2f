import { VIEWER_CSS } from "./styles.js";
import { applyTheme, readStoredTheme } from "./theme.js";
import { createMenu } from "./menus.js";
import {
  iconCaret,
  iconChevronLeft,
  iconChevronRight,
  iconExport,
  iconMinus,
  iconMoon,
  iconMore,
  iconPlus,
  iconSun,
} from "./icons.js";

export function attachMountRoot(host) {
  if (host.shadowRoot) return host.shadowRoot;
  try {
    return host.attachShadow({ mode: "open" });
  } catch {
    return host;
  }
}

export function createViewerShell({ bannerMode }) {
  const root = document.createElement("div");
  root.className = "k2f-root";
  applyTheme(root, readStoredTheme());
  const style = document.createElement("style");
  style.textContent = VIEWER_CSS;
  root.append(style);

  const chrome = document.createElement("div");
  chrome.className = "k2f-chrome";

  const bannerEl = document.createElement("header");
  bannerEl.className = "k2f-banner banner-idle";
  bannerEl.dataset.state = "IDLE";
  bannerEl.textContent =
    "Locked pages are painted by the engine, not by the browser.";
  if (bannerMode === "off") bannerEl.hidden = true;

  const toolbar = document.createElement("nav");
  toolbar.className = "k2f-toolbar";
  toolbar.setAttribute("aria-label", "Document toolbar");

  const left = document.createElement("div");
  left.className = "k2f-header-left";
  const logo = document.createElement("span");
  logo.className = "k2f-logo";
  logo.setAttribute("aria-hidden", "true");
  logo.textContent = "K";
  const docTitle = document.createElement("span");
  docTitle.className = "k2f-doc-title";
  docTitle.textContent = "Untitled";
  const pageNav = document.createElement("div");
  pageNav.className = "k2f-page-nav";
  const prev = iconButton("prev", "Previous", iconChevronLeft());
  const next = iconButton("next", "Next", iconChevronRight());
  const pageLabel = document.createElement("span");
  pageLabel.className = "k2f-page-label";
  pageLabel.textContent = "—";
  pageNav.append(prev, pageLabel, next);
  left.append(logo, docTitle, pageNav);

  const right = document.createElement("div");
  right.className = "k2f-header-right";
  const zoom = document.createElement("div");
  zoom.className = "k2f-zoom";
  const zoomOut = iconButton("zoom-out", "Zoom out", iconMinus());
  const zoomLabel = document.createElement("button");
  zoomLabel.type = "button";
  zoomLabel.className = "k2f-zoom-label";
  zoomLabel.dataset.act = "zoom-menu";
  zoomLabel.setAttribute("aria-label", "Zoom level");
  zoomLabel.textContent = "100%";
  const zoomIn = iconButton("zoom-in", "Zoom in", iconPlus());
  zoom.append(zoomOut, zoomLabel, zoomIn);

  const initialTheme = readStoredTheme();
  const themeBtn = iconButton(
    "theme",
    initialTheme === "light" ? "Switch to dark mode" : "Switch to light mode",
    initialTheme === "light" ? iconMoon() : iconSun(),
  );
  const editBtn = button("edit", "Edit");
  editBtn.classList.add("k2f-edit");
  editBtn.hidden = true;

  const exportWrap = document.createElement("div");
  exportWrap.className = "k2f-export";
  const exportBtn = button("export", "");
  exportBtn.classList.add("k2f-export-run");
  exportBtn.setAttribute("aria-label", "Export as K2F");
  exportBtn.title = "Export as K2F";
  exportBtn.append(iconExport(), exportLabelSpan("Export as K2F"));
  const exportCaret = iconButton("export-menu", "Export format", iconCaret());
  exportCaret.classList.add("k2f-export-caret");
  exportWrap.append(exportBtn, exportCaret);

  const moreBtn = iconButton("more", "More", iconMore());

  prev.disabled = true;
  next.disabled = true;
  zoomOut.disabled = true;
  zoomIn.disabled = true;
  exportBtn.disabled = true;
  exportCaret.disabled = true;

  right.append(
    zoom,
    sep(),
    themeBtn,
    editBtn,
    sep(),
    exportWrap,
    moreBtn,
  );
  toolbar.append(left, right);
  chrome.append(bannerEl, toolbar);

  const body = document.createElement("div");
  body.className = "k2f-body";
  const stage = document.createElement("main");
  stage.className = "k2f-stage";
  const empty = document.createElement("p");
  empty.className = "k2f-empty";
  empty.textContent = "No document loaded.";
  const stack = document.createElement("div");
  stack.className = "k2f-stack";
  stage.append(empty, stack);
  const status = document.createElement("div");
  status.className = "k2f-status";
  status.textContent = "—";
  body.append(stage, status);

  const zoomMenu = createMenu("k2f-zoom-menu");
  const exportMenu = createMenu("k2f-export-menu");
  const moreMenu = createMenu("k2f-more-menu");

  root.append(chrome, body, zoomMenu, exportMenu, moreMenu);

  return {
    root,
    chrome,
    banner: bannerEl,
    toolbar,
    docTitle,
    prev,
    next,
    pageLabel,
    zoomIn,
    zoomOut,
    zoomLabel,
    zoomMenu,
    themeBtn,
    editBtn,
    exportBtn,
    exportCaret,
    exportMenu,
    moreBtn,
    moreMenu,
    status,
    empty,
    stage,
    stack,
  };
}

function button(act, label) {
  const el = document.createElement("button");
  el.type = "button";
  el.dataset.act = act;
  if (label) el.textContent = label;
  return el;
}

function iconButton(act, label, icon) {
  const el = button(act, "");
  el.setAttribute("aria-label", label);
  el.title = label;
  if (icon) el.append(icon);
  return el;
}

function exportLabelSpan(text) {
  const el = document.createElement("span");
  el.className = "k2f-export-text";
  el.textContent = text;
  return el;
}

function sep() {
  const el = document.createElement("span");
  el.className = "k2f-sep";
  el.setAttribute("aria-hidden", "true");
  return el;
}
