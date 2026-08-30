export function linkUrlFromSelection(sel) {
  const mods = sel?.node?.modifiers;
  if (!Array.isArray(mods)) return null;
  const text = typeof sel?.text === "string" ? sel.text : null;
  const range = Array.isArray(sel?.char_range) ? sel.char_range : null;
  for (const m of mods) {
    if (m?.type !== "link" || typeof m.intent !== "string") continue;
    if (!(m.intent.startsWith("https://") || m.intent.startsWith("http://"))) continue;
    if (range && text && Array.isArray(m.range) && m.range.length === 2) {
      const start = utf8ByteToCharIndex(text, m.range[0]);
      const end = utf8ByteToCharIndex(text, m.range[1]);
      if (range[1] <= start || range[0] >= end) continue;
    }
    return m.intent;
  }
  return null;
}

export function openLink(url) {
  if (typeof window === "undefined" || typeof window.open !== "function") return;
  window.open(url, "_blank", "noopener,noreferrer");
}

function utf8ByteToCharIndex(text, byteOffset) {
  const bytes = new TextEncoder().encode(text);
  let i = 0;
  let chars = 0;
  while (i < byteOffset && i < bytes.length) {
    const lead = bytes[i];
    if (lead < 0x80) i += 1;
    else if (lead < 0xe0) i += 2;
    else if (lead < 0xf0) i += 3;
    else i += 4;
    chars += 1;
  }
  return chars;
}
