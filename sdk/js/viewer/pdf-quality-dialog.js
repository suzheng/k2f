import {
  DEFAULT_PDF_EXPORT_SCALE,
  PDF_EXPORT_SCALES,
  pdfExportScaleLabel,
  readStoredPdfExportScale,
  writeStoredPdfExportScale,
} from "./pdf-export-scale.js";

/**
 * Modal to pick PDF stamp scale before export.
 * @param {HTMLElement} root
 * @param {AbortSignal} signal
 */
export function bindPdfQualityDialog(root, signal) {
  const backdrop = document.createElement("div");
  backdrop.className = "k2f-dialog-backdrop";
  backdrop.hidden = true;

  const dialog = document.createElement("div");
  dialog.className = "k2f-dialog";
  dialog.setAttribute("role", "dialog");
  dialog.setAttribute("aria-modal", "true");
  dialog.setAttribute("aria-labelledby", "k2f-pdf-quality-title");

  const title = document.createElement("h2");
  title.id = "k2f-pdf-quality-title";
  title.className = "k2f-dialog-title";
  title.textContent = "Export PDF";

  const hint = document.createElement("p");
  hint.className = "k2f-dialog-hint";
  hint.textContent =
    "Choose quality for pages with blur, shadow, or gradient. Other pages stay vector.";

  const options = document.createElement("div");
  options.className = "k2f-dialog-options";

  let selected = readStoredPdfExportScale();
  const radios = PDF_EXPORT_SCALES.map((scale) => {
    const row = document.createElement("label");
    row.className = "k2f-dialog-option";
    const input = document.createElement("input");
    input.type = "radio";
    input.name = "k2f-pdf-scale";
    input.value = String(scale);
    input.checked = scale === selected;
    const text = document.createElement("span");
    text.textContent = pdfExportScaleLabel(scale);
    row.append(input, text);
    input.addEventListener(
      "change",
      () => {
        if (input.checked) selected = scale;
      },
      { signal },
    );
    options.append(row);
    return input;
  });

  const actions = document.createElement("div");
  actions.className = "k2f-dialog-actions";
  const cancel = document.createElement("button");
  cancel.type = "button";
  cancel.dataset.act = "cancel";
  cancel.textContent = "Cancel";
  const confirm = document.createElement("button");
  confirm.type = "button";
  confirm.dataset.act = "confirm";
  confirm.textContent = "Export";
  actions.append(cancel, confirm);

  dialog.append(title, hint, options, actions);
  backdrop.append(dialog);
  root.append(backdrop);

  let onConfirm = null;

  function syncRadios() {
    for (const input of radios) {
      input.checked = Number(input.value) === selected;
    }
  }

  function close() {
    backdrop.hidden = true;
    onConfirm = null;
  }

  function open(confirmCb) {
    selected = readStoredPdfExportScale();
    syncRadios();
    onConfirm = confirmCb;
    backdrop.hidden = false;
    confirm.focus();
  }

  cancel.addEventListener("click", () => close(), { signal });
  backdrop.addEventListener(
    "click",
    (e) => {
      if (e.target === backdrop) close();
    },
    { signal },
  );
  confirm.addEventListener(
    "click",
    () => {
      const scale = selected;
      writeStoredPdfExportScale(scale);
      const cb = onConfirm;
      close();
      cb?.(scale);
    },
    { signal },
  );
  dialog.addEventListener("click", (e) => e.stopPropagation(), { signal });
  root.addEventListener(
    "keydown",
    (e) => {
      if (backdrop.hidden) return;
      if (e.key === "Escape") {
        e.preventDefault();
        close();
      }
    },
    { signal },
  );

  return { open, close, defaultScale: DEFAULT_PDF_EXPORT_SCALE };
}
