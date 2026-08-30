/** Persist copy format: markdown (default) or plain. */

export const COPY_FORMAT_KEY = "k2f.copyFormat";

export function normalizeCopyFormat(v) {
  return v === "plain" ? "plain" : "markdown";
}

export function readStoredCopyFormat() {
  try {
    if (typeof localStorage === "undefined") return "markdown";
    return normalizeCopyFormat(localStorage.getItem(COPY_FORMAT_KEY));
  } catch {
    return "markdown";
  }
}

export function writeStoredCopyFormat(format) {
  try {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(COPY_FORMAT_KEY, normalizeCopyFormat(format));
  } catch {
    /* private mode */
  }
}
