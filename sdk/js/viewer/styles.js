/** CSS for the embeddable viewer. Document body is a PNG, never DOM text layout. */
export const VIEWER_CSS = `
:host { display: block; height: 100%; min-height: 240px; }
:host(:fullscreen) { height: 100%; background: #c8c8c8; }
.k2f-root {
  --banner-idle: #444;
  --banner-signed: #0d6b3a;
  --banner-unsigned: #3d5a80;
  --banner-broken: #9b1c1c;
  --banner-signed-broken: #6b0f0f;
  --banner-unlocked: #8a5a00;
  font-family: system-ui, sans-serif;
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #c8c8c8;
  color: #111;
}
.k2f-banner {
  padding: 12px 16px;
  color: #fff;
  font-weight: 600;
  letter-spacing: 0.02em;
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
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  padding: 8px 12px;
  background: #222;
  color: #eee;
}
.k2f-toolbar button {
  background: #444;
  color: #fff;
  border: 0;
  padding: 6px 10px;
  cursor: pointer;
}
.k2f-toolbar button:disabled { opacity: 0.4; cursor: default; }
.k2f-toolbar select.k2f-copy-format,
.k2f-toolbar select.k2f-export-format {
  background: #444;
  color: #fff;
  border: 0;
  padding: 6px 8px;
  cursor: pointer;
}
.k2f-stage {
  overflow: auto;
  flex: 1;
  padding: 24px;
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
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.25);
  max-width: none;
  pointer-events: none;
  user-select: none;
  -webkit-user-drag: none;
}
.k2f-highlight {
  position: absolute;
  pointer-events: none;
  border: 2px solid #1a73e8;
  background: rgba(26, 115, 232, 0.12);
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
.k2f-text-layer span::selection { background: rgba(0, 120, 215, 0.35); }
.k2f-popover {
  position: absolute;
  z-index: 3;
  width: 260px;
  padding: 10px;
  background: #1b1b1b;
  color: #eee;
  font-size: 13px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.k2f-popover[hidden] { display: none; }
.k2f-popover p { margin: 0; word-break: break-all; }
.k2f-popover label { display: flex; flex-direction: column; gap: 4px; }
.k2f-popover-text textarea,
.k2f-popover textarea, .k2f-popover input {
  background: #111;
  color: #eee;
  border: 0;
  padding: 6px 8px;
}
.k2f-popover button {
  background: #444;
  color: #fff;
  border: 0;
  padding: 6px 10px;
  cursor: pointer;
}
.k2f-popover button:disabled { opacity: 0.4; cursor: default; }
.k2f-empty { color: #333; }
`;
