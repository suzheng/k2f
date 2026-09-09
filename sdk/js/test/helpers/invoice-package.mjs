import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "../../../..");

/** Published invoice fixture (compiled lock). SDK does not bundle named templates. */
export function invoicePackage(_k2f) {
  return new Uint8Array(
    readFileSync(join(root, "examples/published/invoice.K2F")),
  );
}

/**
 * Zip an in-repo author directory into .K2F bytes (unlocked shell: fonts + theme).
 * Schema files come from the skill folder so save/pack can validate without the CLI.
 */
export function packAuthorDir(rel) {
  const src = join(root, rel);
  const dir = mkdtempSync(join(tmpdir(), "k2f-pack-"));
  const out = join(dir, "pkg.K2F");
  execFileSync(
    "zip",
    ["-q", "-X", "-D", "-r", out, "manifest.json", "content", "styles", "changelog.json", "assets"],
    { cwd: src },
  );
  execFileSync(
    "zip",
    [
      "-q",
      "-X",
      out,
      "schema/manifest.schema.json",
      "schema/nodes.schema.json",
      "schema/signatures.schema.json",
      "schema/styles.schema.json",
      "schema/visual_primitives.schema.json",
    ],
    { cwd: join(root, "skills/k2f") },
  );
  const bytes = new Uint8Array(readFileSync(out));
  rmSync(dir, { recursive: true, force: true });
  return bytes;
}
