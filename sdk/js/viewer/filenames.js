const HOSTILE = /[\x00-\x1f/\\:<>\"|?*]/g;

function stripExt(name, ext) {
  const lower = name.toLowerCase();
  const dot = lower.lastIndexOf(`.${ext}`);
  if (dot >= 0 && dot === name.length - ext.length - 1) {
    return name.slice(0, dot);
  }
  return name;
}

/** Safe download stem from document title. */
export function safeTitleStem(title) {
  const cleaned = (title ?? "")
    .replace(HOSTILE, "_")
    .trim();
  return cleaned || "document";
}

export function exportFilename(title, format, pageCount) {
  const stem = safeTitleStem(title);
  switch (format) {
    case "k2f":
      return `${stripExt(stem, "k2f")}.K2F`;
    case "pdf":
      return `${stripExt(stem, "pdf")}.pdf`;
    case "pptx":
      return `${stripExt(stem, "pptx")}.pptx`;
    case "docx":
      return `${stripExt(stem, "docx")}.docx`;
    case "markdown":
      return `${stripExt(stem, "md")}.md`;
    case "png":
      return pageCount > 1 ? `${stem}-pages.zip` : `${stem}.png`;
    case "jpg":
      return pageCount > 1 ? `${stem}-pages.zip` : `${stem}.jpg`;
    default:
      return `${stem}.bin`;
  }
}

export function exportMime(format, pageCount) {
  switch (format) {
    case "k2f":
      return "application/zip";
    case "pdf":
      return "application/pdf";
    case "pptx":
      return "application/vnd.openxmlformats-officedocument.presentationml.presentation";
    case "docx":
      return "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
    case "markdown":
      return "text/markdown;charset=utf-8";
    case "png":
      return pageCount > 1 ? "application/zip" : "image/png";
    case "jpg":
      return pageCount > 1 ? "application/zip" : "image/jpeg";
    default:
      return "application/octet-stream";
  }
}
