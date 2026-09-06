/** Persist viewer chrome theme (document pixels are unchanged). */

export const THEME_KEY = "k2f.chromeTheme";

export function normalizeTheme(v) {
  return v === "dark" ? "dark" : "light";
}

export function preferredTheme() {
  try {
    if (typeof matchMedia === "function" && matchMedia("(prefers-color-scheme: dark)").matches) {
      return "dark";
    }
  } catch {
    /* ignore */
  }
  return "light";
}

export function readStoredTheme() {
  try {
    if (typeof localStorage === "undefined") return preferredTheme();
    const raw = localStorage.getItem(THEME_KEY);
    if (raw === "light" || raw === "dark") return raw;
  } catch {
    /* private mode */
  }
  return preferredTheme();
}

export function writeStoredTheme(theme) {
  try {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(THEME_KEY, normalizeTheme(theme));
  } catch {
    /* private mode */
  }
}

export function applyTheme(root, theme) {
  root.dataset.theme = normalizeTheme(theme);
}

export function bindTheme({ root, button, signal, onChange }) {
  let theme = readStoredTheme();
  applyTheme(root, theme);
  onChange?.(theme);

  button.addEventListener(
    "click",
    () => {
      theme = theme === "light" ? "dark" : "light";
      writeStoredTheme(theme);
      applyTheme(root, theme);
      onChange?.(theme);
    },
    signal ? { signal } : undefined,
  );

  return {
    theme: () => theme,
  };
}
