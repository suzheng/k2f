/** Persist export format. Order: K2F, PDF, Markdown, PNG, JPG. */

export const EXPORT_FORMATS = [
  { value: "k2f", label: "Export: K2F" },
  { value: "pdf", label: "Export: PDF" },
  { value: "markdown", label: "Export: Markdown" },
  { value: "png", label: "Export: PNG" },
  { value: "jpg", label: "Export: JPG" },
];

export const EXPORT_FORMAT_KEY = "k2f.exportFormat";

export function normalizeExportFormat(v) {
  return EXPORT_FORMATS.some((f) => f.value === v) ? v : "k2f";
}

export function readStoredExportFormat() {
  try {
    if (typeof localStorage === "undefined") return "k2f";
    return normalizeExportFormat(localStorage.getItem(EXPORT_FORMAT_KEY));
  } catch {
    return "k2f";
  }
}

export function writeStoredExportFormat(format) {
  try {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(EXPORT_FORMAT_KEY, normalizeExportFormat(format));
  } catch {
    /* private mode */
  }
}
