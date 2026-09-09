import { initWasm } from "./init.js";
import { agentError, call } from "./errors.js";

export async function markdownToK2f(md, { title = "Document", templateBytes } = {}) {
  if (!templateBytes) {
    throw agentError(
      new Error(
        "markdownToK2f requires templateBytes (a packed .K2F used as the shell)",
      ),
    );
  }
  const wasm = await initWasm();
  return call(() => wasm.markdown_to_k2f(md, title, templateBytes));
}

export async function k2fToMarkdown(bytes) {
  const wasm = await initWasm();
  return call(() => wasm.k2f_to_markdown(bytes));
}
