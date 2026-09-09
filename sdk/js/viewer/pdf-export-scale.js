export const PDF_EXPORT_SCALES = [2, 3, 4];
export const DEFAULT_PDF_EXPORT_SCALE = 4;
const STORAGE_KEY = "k2f-pdf-export-scale";

export function normalizePdfExportScale(value) {
  const n = Number(value);
  return PDF_EXPORT_SCALES.includes(n) ? n : DEFAULT_PDF_EXPORT_SCALE;
}

export function readStoredPdfExportScale() {
  try {
    return normalizePdfExportScale(localStorage.getItem(STORAGE_KEY));
  } catch {
    return DEFAULT_PDF_EXPORT_SCALE;
  }
}

export function writeStoredPdfExportScale(scale) {
  try {
    localStorage.setItem(STORAGE_KEY, String(normalizePdfExportScale(scale)));
  } catch {
    /* ignore */
  }
}

export function pdfExportScaleLabel(scale) {
  switch (normalizePdfExportScale(scale)) {
    case 2:
      return "2× — Standard";
    case 3:
      return "3× — High (4K)";
    default:
      return "4× — Maximum";
  }
}
