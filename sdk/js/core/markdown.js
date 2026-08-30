import { initWasm } from "./init.js";
import { call } from "./errors.js";

export async function markdownToK2f(md, { title = "Document", template = "report" } = {}) {
  const wasm = await initWasm();
  return call(() => wasm.markdown_to_k2f(md, title, template));
}

export async function k2fToMarkdown(bytes) {
  const wasm = await initWasm();
  return call(() => wasm.k2f_to_markdown(bytes));
}
