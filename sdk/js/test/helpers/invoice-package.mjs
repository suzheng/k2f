import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "../../../..");

function zipMembers(zipPath, cwd, members, { append = false } = {}) {
  const env = {
    ...process.env,
    PATH: `/usr/bin:/bin:/usr/sbin:/sbin:${process.env.PATH ?? ""}`,
  };
  const args = append
    ? ["-q", "-X", zipPath, ...members]
    : ["-q", "-X", "-D", "-r", zipPath, ...members];
  let lastErr;
  for (const zip of [
    process.env.ZIP,
    "/usr/bin/zip",
    "/bin/zip",
    "zip",
  ]) {
    if (!zip) continue;
    if (zip.includes("/") && !existsSync(zip)) continue;
    try {
      return execFileSync(zip, args, { cwd, env, stdio: "pipe" });
    } catch (err) {
      lastErr = err;
      if (err?.code !== "ENOENT") throw err;
    }
  }
  const py = process.env.PYTHON ?? "python3";
  const script = `
import os, sys, zipfile
out, cwd, append = sys.argv[1], sys.argv[2], sys.argv[3] == "1"
members = sys.argv[4:]
os.chdir(cwd)
mode = "a" if append and os.path.isfile(out) else "w"
with zipfile.ZipFile(out, mode, zipfile.ZIP_DEFLATED) as zf:
    for m in members:
        if os.path.isdir(m):
            for root, _, files in os.walk(m):
                for name in files:
                    path = os.path.join(root, name)
                    zf.write(path, path)
        else:
            zf.write(m, m)
`;
  execFileSync(
    py,
    ["-c", script, zipPath, cwd, append ? "1" : "0", ...members],
    { env, stdio: "pipe" },
  );
}

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
  zipMembers(out, src, [
    "manifest.json",
    "content",
    "styles",
    "changelog.json",
    "assets",
  ]);
  zipMembers(
    out,
    root,
    [
      "schema/manifest.schema.json",
      "schema/nodes.schema.json",
      "schema/signatures.schema.json",
      "schema/styles.schema.json",
      "schema/visual_primitives.schema.json",
    ],
    { append: true },
  );
  const bytes = new Uint8Array(readFileSync(out));
  rmSync(dir, { recursive: true, force: true });
  return bytes;
}

/** Pack an author directory and relock it through the WASM Editor. */
export function compileAuthorDir(k2f, rel) {
  const unlocked = packAuthorDir(rel);
  const ed = k2f.Editor.open(unlocked);
  try {
    return ed.save();
  } finally {
    ed.free();
  }
}
