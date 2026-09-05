import { exportFilename, exportMime } from "./filenames.js";

function officialScale(viewer) {
  if (typeof viewer.official_scale === "function") {
    return viewer.official_scale();
  }
  if (typeof viewer.constructor.official_scale === "function") {
    return viewer.constructor.official_scale();
  }
  return 2;
}

function resolvePackageBytes({ editor, packageBytes }) {
  if (editor) return editor.save();
  if (!packageBytes) throw new Error("UNLOCKED: no package bytes");
  return packageBytes;
}

function withViewer(Viewer, bytes, fn) {
  const v = new Viewer(bytes);
  try {
    return fn(v);
  } finally {
    v.free();
  }
}

/**
 * @param {object} opts
 * @param {import("../types.d.ts").Viewer} opts.viewer
 * @param {import("../types.d.ts").Editor | null} opts.editor
 * @param {Uint8Array | null} opts.packageBytes
 * @param {string} opts.title
 * @param {string} opts.format
 * @param {typeof import("../types.d.ts").Viewer} opts.Viewer
 * @param {(bytes: Uint8Array) => string} [opts.k2fToMarkdown]
 */
export function exportDocument({
  viewer,
  editor,
  packageBytes,
  title,
  format,
  Viewer,
  k2fToMarkdown,
}) {
  const bytes = resolvePackageBytes({ editor, packageBytes });
  const pageCount = withViewer(Viewer, bytes, (v) => v.page_count());
  const filename = exportFilename(title, format, pageCount);
  const mime = exportMime(format, pageCount);
  const scale = officialScale(viewer);

  let out;
  switch (format) {
    case "k2f":
      out = bytes;
      break;
    case "pdf":
      out = withViewer(Viewer, bytes, (v) => {
        if (typeof v.export_pdf_at === "function") {
          return v.export_pdf_at(scale);
        }
        return v.export_pdf();
      });
      break;
    case "pptx":
      out = withViewer(Viewer, bytes, (v) => v.export_pptx());
      break;
    case "docx":
      out = withViewer(Viewer, bytes, (v) => v.export_docx());
      break;
    case "markdown":
      out = withViewer(Viewer, bytes, (v) => {
        if (typeof v.document_markdown === "function") {
          return new TextEncoder().encode(v.document_markdown());
        }
        if (!k2fToMarkdown) {
          throw new Error("document_markdown unavailable in this WASM build");
        }
        return new TextEncoder().encode(k2fToMarkdown(bytes));
      });
      break;
    case "png":
      out = withViewer(Viewer, bytes, (v) => {
        if (typeof v.export_pages_png_zip !== "function") {
          throw new Error("export_pages_png_zip unavailable");
        }
        return v.export_pages_png_zip(scale);
      });
      break;
    case "jpg":
      out = withViewer(Viewer, bytes, (v) => {
        if (typeof v.export_pages_jpeg_zip !== "function") {
          throw new Error("export_pages_jpeg_zip unavailable");
        }
        return v.export_pages_jpeg_zip(scale);
      });
      break;
    default:
      throw new Error(`unknown export format: ${format}`);
  }

  return { bytes: out, filename, mime };
}
