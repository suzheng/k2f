import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { markdownToK2f, k2fToMarkdown } from "../k2f.js";
import { packAuthorDir } from "./helpers/invoice-package.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const templateBytes = packAuthorDir("templates/report");
const md = await readFile(
  path.join(root, "tests/fixtures/markdown/readme_simple.md"),
  "utf8",
);
const bytes = await markdownToK2f(md, { title: "K2F", templateBytes });
if (bytes[0] !== 0x50 || bytes[1] !== 0x4b) {
  throw new Error("markdownToK2f must return a ZIP package");
}
const emojiMd = "# x\n\n✅ Decision\n";
const emojiBytes = await markdownToK2f(emojiMd, { title: "Emoji", templateBytes });
if (emojiBytes[0] !== 0x50 || emojiBytes[1] !== 0x4b) {
  throw new Error("markdownToK2f with checkmark emoji must return a ZIP package");
}
const out = await k2fToMarkdown(bytes);
if (!out.includes("# K2F")) {
  throw new Error(`expected exported heading, got:\n${out}`);
}
if (!out.includes("semantic document format")) {
  throw new Error("expected exported body");
}
const mathBytes = await markdownToK2f("$$E=mc^2$$", { title: "Math", templateBytes });
const mathMd = await k2fToMarkdown(mathBytes);
if (!mathMd.includes("$$") || !mathMd.includes("E=mc^2")) {
  throw new Error(`expected display math roundtrip, got:\n${mathMd}`);
}
const inlineBytes = await markdownToK2f("The ratio is $a/b$ in the body.", {
  title: "Inline math",
  templateBytes,
});
const inlineMd = await k2fToMarkdown(inlineBytes);
if (!inlineMd.includes("$a/b$")) {
  throw new Error(`expected inline math roundtrip, got:\n${inlineMd}`);
}
console.log(`ok markdown bridge zip=${bytes.length} md=${out.length}`);
