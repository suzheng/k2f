import { paintOpenError, resolveBannerMode } from "./banner.js";
import { attachMountRoot, createViewerShell } from "./shell.js";
import { bindTheme } from "./theme.js";
import { iconMoon, iconSun } from "./icons.js";

/** Fetch / open failures before a Viewer exists. Document body stays empty. */
export function mountErrorShell(host, message, options = {}) {
  const bannerMode = resolveBannerMode(options.banner);
  const root = attachMountRoot(host);
  const els = createViewerShell({
    bannerMode,
    editable: Boolean(options.editable),
  });
  root.replaceChildren(els.root);
  bindTheme({
    root: els.root,
    button: els.themeBtn,
    onChange: (theme) => {
      els.themeBtn.replaceChildren(theme === "light" ? iconMoon() : iconSun());
      const label = theme === "light" ? "Switch to dark mode" : "Switch to light mode";
      els.themeBtn.setAttribute("aria-label", label);
      els.themeBtn.title = label;
    },
  });
  if (bannerMode !== "off") paintOpenError(els.banner, message, bannerMode);
  els.empty.hidden = false;
  els.empty.textContent = message;
  els.stack.hidden = true;
}
