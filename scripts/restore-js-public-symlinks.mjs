#!/usr/bin/env node
/**
 * Restore sdk/js/public symlinks after npm pack (git checkout of public/).
 */
import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
execFileSync("git", ["checkout", "--", "sdk/js/public"], {
  cwd: repoRoot,
  stdio: "inherit",
});
console.log("restored sdk/js/public symlinks");
