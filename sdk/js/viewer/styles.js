/** CSS for the embeddable viewer. Document body is a PNG, never DOM text layout. */
export const VIEWER_CSS = `
:host { display: block; height: 100%; min-height: 240px; }
:host(:fullscreen) { height: 100%; background: var(--k2f-canvas, #fafafa); }
.k2f-root {
  --banner-idle: #444;
  --banner-signed: #0d6b3a;
  --banner-unsigned: #3d5a80;
  --banner-broken: #9b1c1c;
  --banner-signed-broken: #6b0f0f;
  --banner-unlocked: #8a5a00;
  --k2f-primary: #007aff;
  --k2f-primary-hover: #0066d9;
  --k2f-bg: #ffffff;
  --k2f-canvas: #fafafa;
  --k2f-surface: #ffffff;
  --k2f-text: #1a1a1a;
  --k2f-text-secondary: #6e6e73;
  --k2f-text-hint: #8e8e93;
  --k2f-divider: rgba(0, 0, 0, 0.06);
  --k2f-divider-strong: rgba(0, 0, 0, 0.08);
  --k2f-glass: rgba(255, 255, 255, 0.85);
  --k2f-hover: rgba(0, 0, 0, 0.06);
  --k2f-input: rgba(0, 0, 0, 0.04);
  --k2f-scroll: rgba(0, 0, 0, 0.2);
  --k2f-status-bg: rgba(0, 0, 0, 0.6);
  --k2f-status-fg: rgba(255, 255, 255, 0.8);
  --k2f-page-shadow: 0 2px 12px rgba(0, 0, 0, 0.12);
  --k2f-highlight: #007aff;
  --k2f-menu-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  container-type: inline-size;
  container-name: k2f;
  position: relative;
  font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "Segoe UI", Roboto, sans-serif;
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  min-width: 0;
  overflow: hidden;
  background: var(--k2f-bg);
  color: var(--k2f-text);
  transition: background-color 300ms ease, color 300ms ease;
}
.k2f-root[data-theme="dark"] {
  --k2f-primary: #0a84ff;
  --k2f-primary-hover: #409cff;
  --k2f-bg: #1a1a1a;
  --k2f-canvas: #242424;
  --k2f-surface: #2c2c2e;
  --k2f-text: #e8e8e8;
  --k2f-text-secondary: #98989d;
  --k2f-text-hint: #6e6e73;
  --k2f-divider: rgba(255, 255, 255, 0.06);
  --k2f-divider-strong: rgba(255, 255, 255, 0.08);
  --k2f-glass: rgba(26, 26, 26, 0.85);
  --k2f-hover: rgba(255, 255, 255, 0.08);
  --k2f-input: rgba(255, 255, 255, 0.08);
  --k2f-scroll: rgba(255, 255, 255, 0.2);
  --k2f-status-bg: rgba(255, 255, 255, 0.15);
  --k2f-status-fg: rgba(255, 255, 255, 0.85);
  --k2f-page-shadow: 0 2px 12px rgba(0, 0, 0, 0.45);
  --k2f-highlight: #0a84ff;
  --k2f-menu-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
}
.k2f-chrome {
  flex-shrink: 0;
  z-index: 20;
  transition: margin-top 250ms ease;
}
.k2f-banner {
  padding: 12px 16px;
  color: #fff;
  font-weight: 600;
  letter-spacing: 0.02em;
}
.k2f-banner[hidden] {
  display: none;
}
.k2f-banner.banner-compact {
  padding: 6px 16px;
  font-size: 0.875rem;
  font-weight: 600;
}
.banner-idle { background: var(--banner-idle); }
.banner-signed { background: var(--banner-signed); }
.banner-unsigned { background: var(--banner-unsigned); }
.banner-unlocked { background: var(--banner-unlocked); }
.banner-broken {
  background: var(--banner-broken);
  padding: 28px 16px;
  font-size: 1.15rem;
}
.banner-signed-broken {
  background: var(--banner-signed-broken);
  padding: 36px 16px;
  font-size: 1.25rem;
}
.k2f-toolbar {
  display: flex;
  flex-wrap: nowrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  height: 48px;
  padding: 0 20px;
  background: var(--k2f-glass);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--k2f-divider);
  color: var(--k2f-text);
  min-width: 0;
}
.k2f-header-left,
.k2f-header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
.k2f-header-right { flex-shrink: 0; }
.k2f-logo {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  background: var(--k2f-primary);
  color: #fff;
  font-size: 13px;
  font-weight: 700;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  letter-spacing: -0.04em;
}
.k2f-doc-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--k2f-text);
  max-width: 30ch;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}
.k2f-page-nav {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
}
.k2f-page-label {
  font-size: 13px;
  font-weight: 400;
  color: var(--k2f-text-hint);
  min-width: 4.5em;
  text-align: center;
  padding: 0 4px;
}
.k2f-sep {
  width: 1px;
  height: 24px;
  background: var(--k2f-divider-strong);
  flex-shrink: 0;
}
.k2f-toolbar button {
  background: transparent;
  color: var(--k2f-text);
  border: 0;
  border-radius: 6px;
  width: 32px;
  height: 32px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font: inherit;
  font-size: 14px;
  font-weight: 500;
  transition: background 150ms ease, transform 100ms ease, color 150ms ease;
}
.k2f-toolbar button:hover:not(:disabled) {
  background: var(--k2f-hover);
}
.k2f-toolbar button:active:not(:disabled) {
  transform: scale(0.97);
}
.k2f-toolbar button:disabled { opacity: 0.4; cursor: default; }
.k2f-toolbar button svg {
  display: block;
  flex-shrink: 0;
}
.k2f-zoom {
  display: flex;
  align-items: center;
  gap: 2px;
}
.k2f-zoom-label {
  width: auto;
  min-width: 44px;
  padding: 0 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--k2f-text-secondary);
}
.k2f-toolbar .k2f-zoom-label {
  width: auto;
}
.k2f-edit {
  width: auto;
  padding: 0 12px;
}
.k2f-toolbar .k2f-edit {
  width: auto;
}
.k2f-toolbar .k2f-edit[aria-pressed="true"] {
  background: var(--k2f-primary);
  color: #fff;
}
.k2f-toolbar .k2f-edit[aria-pressed="true"]:hover:not(:disabled) {
  background: var(--k2f-primary-hover);
}
.k2f-export {
  display: inline-flex;
  align-items: stretch;
}
.k2f-toolbar .k2f-export-run {
  width: auto;
  height: 32px;
  padding: 0 12px;
  gap: 6px;
  border-radius: 6px 0 0 6px;
  background: var(--k2f-primary);
  color: #fff;
  font-size: 14px;
  font-weight: 500;
}
.k2f-toolbar .k2f-export-run:hover:not(:disabled) {
  background: var(--k2f-primary-hover);
}
.k2f-toolbar .k2f-export-caret {
  width: 28px;
  border-radius: 0 6px 6px 0;
  background: var(--k2f-primary);
  color: #fff;
  border-left: 1px solid rgba(255, 255, 255, 0.25);
}
.k2f-toolbar .k2f-export-caret:hover:not(:disabled) {
  background: var(--k2f-primary-hover);
}
.k2f-body {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--k2f-canvas);
  transition: background-color 300ms ease;
}
.k2f-stage {
  overflow: auto;
  flex: 1;
  padding: 24px;
  scrollbar-width: thin;
  scrollbar-color: var(--k2f-scroll) transparent;
}
.k2f-stage::-webkit-scrollbar { width: 6px; height: 6px; }
.k2f-stage::-webkit-scrollbar-track { background: transparent; }
.k2f-stage::-webkit-scrollbar-thumb {
  background: var(--k2f-scroll);
  border-radius: 3px;
}
.k2f-stack {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  width: max-content;
  max-width: none;
  margin-inline: auto;
  gap: 24px;
}
.k2f-page-wrap { position: relative; }
.k2f-page {
  background: #fff;
  box-shadow: var(--k2f-page-shadow);
  max-width: none;
  pointer-events: none;
  user-select: none;
  -webkit-user-drag: none;
}
.k2f-highlight {
  position: absolute;
  pointer-events: none;
  border: 2px solid var(--k2f-highlight);
  background: color-mix(in srgb, var(--k2f-highlight) 12%, transparent);
  box-sizing: border-box;
  z-index: 1;
}
.k2f-text-layer {
  position: absolute;
  inset: 0;
  z-index: 2;
  overflow: hidden;
  pointer-events: auto;
  user-select: text;
  -webkit-user-select: text;
  line-height: 0;
}
.k2f-text-layer span {
  position: absolute;
  color: transparent;
  white-space: pre;
  overflow: hidden;
  line-height: 1;
  cursor: text;
  transform-origin: 0 0;
  font-family: sans-serif;
}
.k2f-text-layer br {
  display: block;
  width: 0;
  height: 0;
  overflow: hidden;
}
.k2f-text-layer span::selection { background: rgba(0, 122, 255, 0.28); }
.k2f-popover {
  position: absolute;
  z-index: 3;
  width: 260px;
  padding: 10px;
  background: var(--k2f-surface);
  color: var(--k2f-text);
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--k2f-divider);
  box-shadow: var(--k2f-menu-shadow);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.k2f-popover[hidden] { display: none; }
.k2f-popover p { margin: 0; word-break: break-all; }
.k2f-popover label { display: flex; flex-direction: column; gap: 4px; color: var(--k2f-text-secondary); }
.k2f-popover-text textarea,
.k2f-popover textarea, .k2f-popover input {
  background: var(--k2f-input);
  color: var(--k2f-text);
  border: 0;
  border-radius: 6px;
  padding: 6px 8px;
  font: inherit;
}
.k2f-popover button {
  background: var(--k2f-primary);
  color: #fff;
  border: 0;
  border-radius: 6px;
  padding: 6px 10px;
  cursor: pointer;
  font: inherit;
  font-size: 13px;
  font-weight: 500;
}
.k2f-popover button[data-act="copy"] {
  background: var(--k2f-hover);
  color: var(--k2f-text);
}
.k2f-popover button:disabled { opacity: 0.4; cursor: default; }
.k2f-empty { color: var(--k2f-text-secondary); }
.k2f-status {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 6;
  pointer-events: none;
  padding: 6px 16px;
  border-radius: 20px;
  background: var(--k2f-status-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  color: var(--k2f-status-fg);
  font-size: 12px;
  font-weight: 400;
  white-space: nowrap;
  opacity: 0;
  transition: opacity 400ms ease;
}
.k2f-status.is-visible { opacity: 1; }
.k2f-menu {
  position: absolute;
  z-index: 40;
  min-width: 200px;
  padding: 6px;
  background: var(--k2f-surface);
  color: var(--k2f-text);
  border: 1px solid var(--k2f-divider);
  border-radius: 8px;
  box-shadow: var(--k2f-menu-shadow);
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.k2f-menu[hidden] { display: none; }
.k2f-menu-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  border: 0;
  background: transparent;
  color: var(--k2f-text);
  border-radius: 6px;
  padding: 8px 10px;
  font: inherit;
  font-size: 14px;
  font-weight: 400;
  text-align: left;
  cursor: pointer;
}
.k2f-menu-item:hover { background: var(--k2f-hover); }
.k2f-menu-item[data-checked="true"]::after {
  content: "✓";
  color: var(--k2f-primary);
  font-size: 12px;
}
.k2f-menu-sep {
  height: 1px;
  margin: 4px 6px;
  background: var(--k2f-divider);
}
.k2f-menu-export { display: none; }
.k2f-export-text { display: none; }

@container k2f (min-width: 1200px) {
  .k2f-export-text { display: inline; }
}
@container k2f (max-width: 767px) {
  .k2f-toolbar {
    height: 40px;
    padding: 0 12px;
  }
  .k2f-toolbar button {
    width: 28px;
    height: 28px;
  }
  .k2f-export { display: none; }
  .k2f-menu-export { display: flex; flex-direction: column; gap: 2px; }
  .k2f-doc-title { max-width: 14ch; }
}
`;
