import { createK2f } from "../k2f.js";
import { createViewer } from "../core/init-viewer.js";
import { errorMessage } from "../core/errors.js";
import { paintBanner, paintOpenError, resolveBannerMode } from "./banner.js";
import { bindPageNav } from "./page-nav.js";
import { eventToPt } from "./coords.js";
import { bindHighlight } from "./highlight.js";
import { downloadBytes } from "./download.js";
import { createStack } from "./stack.js";
import { attachMountRoot, createViewerShell } from "./shell.js";
import { bindPopover } from "./popover.js";
import { bindCopy } from "./copy.js";
import { bindDragFlag } from "./drag.js";
import { linkUrlFromSelection, openLink } from "./links.js";
import {
  normalizeCopyFormat,
  readStoredCopyFormat,
  writeStoredCopyFormat,
} from "./copy-format.js";
import {
  EXPORT_FORMATS,
  exportFormatLabel,
  normalizeExportFormat,
  readStoredExportFormat,
  writeStoredExportFormat,
} from "./export-format.js";
import { exportDocument } from "./export-actions.js";
import { bindPdfQualityDialog } from "./pdf-quality-dialog.js";
import { bindFullscreen } from "./fullscreen.js";
import { bindEditMode } from "./edit-mode.js";
import { ZOOM_STEPS, fitZoom, stageInnerWidth } from "./zoom-fit.js";
import { bindTheme } from "./theme.js";
import { iconMoon, iconSun } from "./icons.js";
import { createMenuController, menuItem, menuSep } from "./menus.js";
import {
  bindChromeScroll,
  bindStatusBar,
  formatBytes,
} from "./chrome-behavior.js";

export async function mountK2fViewer(host, bytes, options = {}) {
  const bannerMode = resolveBannerMode(options.banner);
  const editable = Boolean(options.editable);
  const useSdk = editable || options.runtime === "sdk";
  const k2f = useSdk ? await createK2f() : await createViewer();
  const root = attachMountRoot(host);
  const els = createViewerShell({ bannerMode });
  root.replaceChildren(els.root);

  let copyFormat = normalizeCopyFormat(
    options.copyFormat ?? readStoredCopyFormat(),
  );
  let exportFormat = normalizeExportFormat(
    options.exportFormat ?? readStoredExportFormat(),
  );

  const abort = new AbortController();
  const { signal } = abort;
  let viewer = null;
  let editor = null;
  let packageBytes = bytes;
  let zoom = 1;

  const menus = createMenuController({ root: els.root, signal });
  const pdfQuality = bindPdfQualityDialog(els.root, signal);
  const chromeScroll = bindChromeScroll({
    root: els.root,
    chrome: els.chrome,
    stage: els.stage,
    banner: els.banner,
    signal,
  });
  const statusBar = bindStatusBar({
    stage: els.stage,
    status: els.status,
    signal,
  });

  bindTheme({
    root: els.root,
    button: els.themeBtn,
    signal,
    onChange: syncThemeIcon,
  });

  const stack = createStack({
    stackEl: els.stack,
    empty: els.empty,
    Viewer: k2f.Viewer,
  });

  const nav = bindPageNav({
    prev: els.prev,
    next: els.next,
    label: els.pageLabel,
    wrapsOf: () => stack.wraps(),
    stage: els.stage,
    signal,
    onChange: (page) => {
      updateStatus();
      emit(host, "k2f-page-change", { page });
      const id = edit.id();
      if (id) selectId(id);
    },
  });

  const highlight = bindHighlight();
  const editMode = bindEditMode({
    button: els.editBtn,
    canEdit: editable,
    signal,
    onChange: (editing) => {
      if (!editing) edit.clear();
    },
  });
  const edit = bindPopover({
    wrapsOf: () => stack.wraps(),
    highlightOf: (page) => stack.highlightOf(page),
    editingOf: () => editMode.editing(),
    viewerOf: () => viewer,
    editorOf: () => editor,
    zoomOf: () => zoom,
    highlight,
    onRelock: async (nextBytes, id) => {
      await open(nextBytes);
      const boxes = JSON.parse(viewer.boxes_for(id));
      if (boxes.length) nav.go(boxes[0].page);
      selectId(id);
    },
    onError: (err) => {
      if (bannerMode !== "off") paintOpenError(els.banner, errorMessage(err), bannerMode);
      chromeScroll.syncPin();
    },
  });

  const dragged = bindDragFlag(els.stack, signal);
  bindCopy(root, {
    signal,
    viewerOf: () => viewer,
    copyFormatOf: () => copyFormat,
  });

  const copyMarkdown = menuItem({
    label: "Copy as Markdown",
    value: "markdown",
    checked: copyFormat === "markdown",
  });
  const copyPlain = menuItem({
    label: "Copy as Plain Text",
    value: "plain",
    checked: copyFormat === "plain",
  });
  const fullscreenSep = menuSep();
  const fullscreenItem = menuItem({ label: "Fullscreen", value: "fullscreen" });
  fullscreenItem.dataset.act = "fullscreen";
  const mobileExport = document.createElement("div");
  mobileExport.className = "k2f-menu-export";
  mobileExport.append(menuSep());
  for (const format of EXPORT_FORMATS) {
    const item = menuItem({ label: format.label, value: format.value });
    item.dataset.kind = "export";
    mobileExport.append(item);
  }
  els.moreMenu.append(copyMarkdown, copyPlain, fullscreenSep, fullscreenItem, mobileExport);
  bindFullscreen({ button: fullscreenItem, target: host, signal });
  fullscreenItem.addEventListener("click", () => menus.closeAll(), { signal });
  if (fullscreenItem.hidden) fullscreenSep.hidden = true;

  for (const format of EXPORT_FORMATS) {
    const item = menuItem({
      label: format.label,
      value: format.value,
      checked: format.value === exportFormat,
    });
    item.dataset.kind = "export";
    els.exportMenu.append(item);
  }
  for (const step of ZOOM_STEPS) {
    els.zoomMenu.append(
      menuItem({
        label: `${Math.round(step * 100)}%`,
        value: String(step),
        checked: step === 1,
      }),
    );
  }
  els.zoomMenu.append(menuItem({ label: "Fit width", value: "fit" }));

  menus.register({ trigger: els.zoomLabel, menu: els.zoomMenu, align: "left" });
  menus.register({ trigger: els.exportCaret, menu: els.exportMenu, align: "right" });
  menus.register({ trigger: els.moreBtn, menu: els.moreMenu, align: "right" });

  copyMarkdown.addEventListener("click", () => setCopyFormat("markdown"), { signal });
  copyPlain.addEventListener("click", () => setCopyFormat("plain"), { signal });
  els.exportMenu.addEventListener("click", onExportMenuClick, { signal });
  mobileExport.addEventListener("click", onExportMenuClick, { signal });
  els.zoomMenu.addEventListener(
    "click",
    (e) => {
      const item = e.target.closest(".k2f-menu-item");
      if (!item || !viewer) return;
      menus.closeAll();
      if (item.dataset.value === "fit") {
        setZoom(fitZoom(viewer, stageInnerWidth(els.stage)));
        return;
      }
      const next = Number(item.dataset.value);
      if (ZOOM_STEPS.includes(next)) setZoom(next);
    },
    { signal },
  );
  els.stage.addEventListener("scroll", () => menus.closeAll(), { signal, passive: true });

  syncExportButton();

  function syncThemeIcon(theme) {
    els.themeBtn.replaceChildren(theme === "light" ? iconMoon() : iconSun());
    const label = theme === "light" ? "Switch to dark mode" : "Switch to light mode";
    els.themeBtn.setAttribute("aria-label", label);
    els.themeBtn.title = label;
  }

  function setCopyFormat(next) {
    copyFormat = normalizeCopyFormat(next);
    writeStoredCopyFormat(copyFormat);
    copyMarkdown.dataset.checked = copyFormat === "markdown" ? "true" : "false";
    copyPlain.dataset.checked = copyFormat === "plain" ? "true" : "false";
    menus.closeAll();
    emit(host, "k2f-copy-format", { copyFormat });
  }

  function setExportFormat(next) {
    exportFormat = normalizeExportFormat(next);
    writeStoredExportFormat(exportFormat);
    syncExportButton();
    emit(host, "k2f-export-format", { exportFormat });
  }

  function syncExportButton() {
    const label = exportFormatLabel(exportFormat);
    els.exportBtn.setAttribute("aria-label", label);
    els.exportBtn.title = label;
    const text = els.exportBtn.querySelector(".k2f-export-text");
    if (text) text.textContent = label;
    for (const item of els.root.querySelectorAll(".k2f-menu-item[data-kind=export]")) {
      item.dataset.checked = item.dataset.value === exportFormat ? "true" : "false";
    }
  }

  function onExportMenuClick(e) {
    const item = e.target.closest(".k2f-menu-item[data-kind=export]");
    if (!item) return;
    menus.closeAll();
    setExportFormat(item.dataset.value);
    beginExport(item.dataset.value);
  }

  function beginExport(format = exportFormat) {
    if (normalizeExportFormat(format) === "pdf") {
      pdfQuality.open((scale) => downloadExport("pdf", scale));
      return;
    }
    downloadExport(format);
  }

  function downloadExport(format = exportFormat, pdfScale) {
    try {
      const { bytes: out, filename, mime } = runExport(format, pdfScale);
      downloadBytes(out, filename, mime);
    } catch (err) {
      if (bannerMode !== "off") {
        paintOpenError(els.banner, errorMessage(err), bannerMode);
      }
      chromeScroll.syncPin();
    }
  }

  function setDocTitle(title) {
    const t = String(title || "Untitled").trim() || "Untitled";
    els.docTitle.textContent = t;
    els.docTitle.title = t;
  }

  function updateStatus() {
    const page = els.pageLabel.textContent || "—";
    const z = els.zoomLabel.textContent || "100%";
    els.status.textContent = `${page} · ${z} · ${formatBytes(packageBytes?.byteLength ?? 0)}`;
  }

  function setZoom(next) {
    zoom = next;
    els.zoomLabel.textContent = `${Math.round(zoom * 100)}%`;
    els.zoomOut.disabled = !viewer || zoom === ZOOM_STEPS[0];
    els.zoomIn.disabled = !viewer || zoom === ZOOM_STEPS[ZOOM_STEPS.length - 1];
    const noDoc = !viewer || viewer.page_count() === 0;
    els.exportBtn.disabled = noDoc;
    els.exportCaret.disabled = noDoc;
    for (const item of els.root.querySelectorAll(".k2f-menu-item[data-kind=export]")) {
      item.disabled = noDoc;
    }
    for (const item of els.zoomMenu.querySelectorAll(".k2f-menu-item")) {
      item.dataset.checked =
        item.dataset.value !== "fit" && Number(item.dataset.value) === zoom ? "true" : "false";
    }
    stack.layout(viewer, zoom);
    updateStatus();
    const id = edit.id();
    if (id) selectId(id);
  }

  function selectId(id) {
    if (!viewer || !id) return;
    const raw = viewer.selection(id);
    if (!raw) return;
    const sel = JSON.parse(raw);
    const url = linkUrlFromSelection(sel);
    if (url) openLink(url);
    if (editMode.editing()) edit.show(sel);
    emit(host, "k2f-select", sel);
  }

  async function open(nextBytes) {
    packageBytes = nextBytes;
    stack.clear();
    edit.clear();
    editMode.setEditing(false);
    highlight.hide();
    if (viewer) {
      viewer.free();
      viewer = null;
    }
    if (editor) {
      editor.free();
      editor = null;
    }
    try {
      viewer = new k2f.Viewer(packageBytes);
      editor = editable ? k2f.Editor.open(packageBytes) : null;
    } catch (err) {
      viewer = null;
      editor = null;
      nav.reset(0);
      setZoom(1);
      setDocTitle("Untitled");
      if (bannerMode !== "off") {
        paintOpenError(els.banner, errorMessage(err), bannerMode);
      }
      chromeScroll.syncPin();
      updateStatus();
      stack.showEmpty("This file could not be opened.");
      throw err;
    }
    if (bannerMode !== "off") {
      paintBanner(
        els.banner,
        {
          banner: viewer.banner(),
          statusCode: viewer.status_code(),
          hashCode: viewer.hash_code(),
          fingerprint: viewer.fingerprint(),
          signedBy: viewer.signed_by(),
          signedAt: viewer.signed_at(),
          generatedBy: viewer.generated_by(),
        },
        bannerMode,
      );
    }
    chromeScroll.syncPin();
    setDocTitle(viewer.title?.() || options.title || "Untitled");
    nav.reset(viewer.page_count());
    setZoom(fitZoom(viewer, stageInnerWidth(els.stage)));
    stack.build(viewer, zoom);
    els.stage.scrollTop = 0;
    chromeScroll.show();
    statusBar.show();
    emit(host, "k2f-open", {
      banner: viewer.banner(),
      statusCode: viewer.status_code(),
      pageCount: viewer.page_count(),
      handle,
    });
  }

  function runExport(format = exportFormat, pdfScale) {
    if (!viewer) throw new Error("UNLOCKED: no viewer");
    const title = viewer.title?.() ?? options.title ?? "document";
    if (editor) {
      packageBytes = editor.save();
    }
    return exportDocument({
      viewer,
      editor: null,
      packageBytes,
      title,
      format: normalizeExportFormat(format),
      Viewer: k2f.Viewer,
      k2fToMarkdown: k2f.k2fToMarkdown?.bind(k2f),
      pdfScale,
    });
  }

  const handle = {
    get viewer() {
      return viewer;
    },
    get editor() {
      return editor;
    },
    get editing() {
      return editMode.editing();
    },
    destroy() {
      abort.abort();
      stack.dispose();
      if (viewer) viewer.free();
      if (editor) editor.free();
      viewer = null;
      editor = null;
      els.root.remove();
    },
    goPage(page) {
      nav.go(page);
    },
    export(format) {
      return runExport(format);
    },
    open,
    selectId,
  };

  els.zoomIn.addEventListener(
    "click",
    () => {
      const i = ZOOM_STEPS.indexOf(zoom);
      if (i < ZOOM_STEPS.length - 1) setZoom(ZOOM_STEPS[i + 1]);
    },
    { signal },
  );
  els.zoomOut.addEventListener(
    "click",
    () => {
      const i = ZOOM_STEPS.indexOf(zoom);
      if (i > 0) setZoom(ZOOM_STEPS[i - 1]);
    },
    { signal },
  );
  els.exportBtn.addEventListener(
    "click",
    () => {
      beginExport();
    },
    { signal },
  );
  els.stack.addEventListener(
    "click",
    (e) => {
      if (!viewer || viewer.page_count() === 0) return;
      if (e.target.closest(".k2f-popover")) return;
      if (dragged()) return;
      const sel = window.getSelection();
      if (sel && !sel.isCollapsed && sel.toString()) return;
      const wrap = e.target.closest(".k2f-page-wrap");
      if (!wrap) {
        edit.clear();
        highlight.hide();
        return;
      }
      const page = Number(wrap.dataset.page);
      const pt = eventToPt(e, wrap, zoom);
      const raw = viewer.hit_selection(page, pt.x, pt.y);
      if (!raw) {
        edit.clear();
        highlight.hide();
        return;
      }
      const next = JSON.parse(raw);
      const url = linkUrlFromSelection(next);
      if (url) openLink(url);
      if (editMode.editing()) edit.show(next);
      emit(host, "k2f-select", next);
    },
    { signal },
  );

  await open(bytes);
  return handle;
}

function emit(host, type, detail) {
  host.dispatchEvent(new CustomEvent(type, { detail, bubbles: true, composed: true }));
}
