import { paintOpenError } from "./banner.js";
import { attachMountRoot, createViewerShell } from "./shell.js";

/** Fetch / open failures before a Viewer exists. Document body stays empty. */
export function mountErrorShell(host, message, options = {}) {
  const root = attachMountRoot(host);
  const els = createViewerShell({
    banner: options.banner !== false,
    editable: Boolean(options.editable),
  });
  root.replaceChildren(els.root);
  if (options.banner !== false) paintOpenError(els.banner, message);
  els.empty.hidden = false;
  els.empty.textContent = message;
  els.stack.hidden = true;
}
