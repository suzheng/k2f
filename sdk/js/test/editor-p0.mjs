import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createK2f } from "../k2f.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "../../..");
const contract = readFileSync(join(root, "examples/published/contract.K2F"));

const k2f = await createK2f();
const ed = k2f.Editor.open(new Uint8Array(contract));
const before = ed.outline().length;
ed.replaceText("contract.clause_4", "Termination requires thirty days written notice.");
ed.setRole("contract.clause_4", "critical_warning");
const diff = ed.diff();
if (diff.length !== 1 || diff[0].id !== "contract.clause_4") {
  throw new Error(`expected single clause diff, got ${JSON.stringify(diff)}`);
}
ed.setGeneratedBy("editor-p0-test");
const saved = ed.save();
const again = k2f.Editor.open(saved);
if (again.outline().length !== before) {
  throw new Error("outline length changed unexpectedly after save");
}
console.log(`ok editor-p0 wasm diff=${diff.length} saved=${saved.length}`);
