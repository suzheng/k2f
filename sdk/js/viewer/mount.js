import { createK2f } from "../k2f.js";
import { createViewer } from "../core/init-viewer.js";
import { errorMessage } from "../core/errors.js";
import { paintBanner, paintOpenError } from "./banner.js";
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
  normalizeExportFormat,
  readStoredExportFormat,
  writeStoredExportFormat,
} from "./export-format.js";
import { exportDocument } from "./export-actions.js";
import { bindFullscreen } from "./fullscreen.js";
import { bindEditMode } from "./edit-mode.js";
import { ZOOM_STEPS, fitZoom, stageInnerWidth } from "./zoom-fit.js";

export async function mountK2fViewer(host, bytes, options = {}) {
  const showBanner = options.banner !== false;
  const editable = Boolean(options.editable);
  const useSdk = editable || options.runtime === "sdk";
  const k2f = useSdk ? await createK2f() : await createViewer();
  const root = attachMountRoot(host);
  const els = createViewerShell({ banner: showBanner });
  root.replaceChildren(els.root);

  let copyFormat = normalizeCopyFormat(
    options.copyFormat ?? readStoredCopyFormat(),
  );
  els.copyFormat.value = copyFormat;
  let exportFormat = normalizeExportFormat(
    options.exportFormat ?? readStoredExportFormat(),
  );
  els.exportFormat.value = exportFormat;

  const abort = new AbortController();
  const { signal } = abort;
  let viewer = null;
  let editor = null;
  let packageBytes = bytes;
  let zoom = 1;

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
      if (showBanner) paintOpenError(els.banner, errorMessage(err));
    },
  });

  const dragged = bindDragFlag(els.stack, signal);
  bindFullscreen({ button: els.fullscreen, target: host, signal });
  bindCopy(root, {
    signal,
    viewerOf: () => viewer,
    copyFormatOf: () => copyFormat,
  });

  els.copyFormat.addEventListener(
    "change",
    () => {
      copyFormat = normalizeCopyFormat(els.copyFormat.value);
      writeStoredCopyFormat(copyFormat);
      emit(host, "k2f-copy-format", { copyFormat });
    },
    { signal },
  );

  els.exportFormat.addEventListener(
    "change",
    () => {
      exportFormat = normalizeExportFormat(els.exportFormat.value);
      writeStoredExportFormat(exportFormat);
      emit(host, "k2f-export-format", { exportFormat });
    },
    { signal },
  );

  function setZoom(next) {
    zoom = next;
    els.zoomLabel.textContent = `${Math.round(zoom * 100)}%`;
    els.zoomOut.disabled = !viewer || zoom === ZOOM_STEPS[0];
    els.zoomIn.disabled = !viewer || zoom === ZOOM_STEPS[ZOOM_STEPS.length - 1];
    els.exportBtn.disabled = !viewer || viewer.page_count() === 0;
    stack.layout(viewer, zoom);
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
      if (showBanner) {
        paintOpenError(els.banner, errorMessage(err));
      }
      stack.showEmpty("This file could not be opened.");
      throw err;
    }
    if (showBanner) {
      paintBanner(els.banner, {
        banner: viewer.banner(),
        statusCode: viewer.status_code(),
        hashCode: viewer.hash_code(),
        fingerprint: viewer.fingerprint(),
        signedBy: viewer.signed_by(),
        signedAt: viewer.signed_at(),
        generatedBy: viewer.generated_by(),
      });
    }
    nav.reset(viewer.page_count());
    setZoom(fitZoom(viewer, stageInnerWidth(els.stage)));
    stack.build(viewer, zoom);
    els.stage.scrollTop = 0;
    emit(host, "k2f-open", {
      banner: viewer.banner(),
      statusCode: viewer.status_code(),
      pageCount: viewer.page_count(),
      handle,
    });
  }

  function runExport(format = exportFormat) {
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
      try {
        const { bytes: out, filename, mime } = runExport();
        downloadBytes(out, filename, mime);
      } catch (err) {
        if (showBanner) {
          paintOpenError(els.banner, errorMessage(err));
        }
      }
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
