/** Persist export format. Order: K2F, PDF, PPTX, DOCX, Markdown, PNG, JPG. */

export const EXPORT_FORMATS = [
  { value: "k2f", label: "Export as K2F" },
  { value: "pdf", label: "Export as PDF" },
  { value: "pptx", label: "Export as PowerPoint" },
  { value: "docx", label: "Export as Word" },
  { value: "markdown", label: "Export as Markdown" },
  { value: "png", label: "Export as PNG" },
  { value: "jpg", label: "Export as JPG" },
];

export function exportFormatLabel(value) {
  return EXPORT_FORMATS.find((f) => f.value === value)?.label ?? "Export as K2F";
}

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
