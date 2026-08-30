# Optional Python / JS Editor SDK

**Not the default skill path.** Agents with file editing should use [writing.md](../writing.md) (JSON + `pack_verify.py`). Use this reference for Python `Editor`, JS/WASM `Editor`, MCP tools, or [edit_and_verify.py](../../scripts/edit_and_verify.py).

Requires `pip install k2f` only (`k2f` on PATH or `K2F_CLI`).

## When to use the SDK

- Deterministic one-line patch script (`edit_and_verify.py`)
- Batch automation without unpacking to disk
- MCP tool-calling environments
- JS viewer popover save (`Editor.save`)

**When NOT to use:** authoring grid/stack/overlay in an unpacked directory — edit JSON + `pack_verify.py` instead.

## Python Editor loop

1. `Editor.open_bytes(path)` or `open_bytes(bytes)`
2. `outline` / `search` / `get_node`
3. `replace_text` / `set_role` / `insert_node` / `delete_node`
4. `diff`
5. `set_generated_by` → `save_bytes()` / `save_with(hash?)`
6. `k2f verify` on disk → **`UNSIGNED`**

`save` = collapse tables + compile + pack + append changelog + **strip signatures**.

## Binding names

| Concept | Python | JS / WASM | Rust (library) |
|---------|--------|-----------|----------------|
| Open | `Editor.open_bytes(bytes)` | `Editor.open(bytes)` | `Editor::open(&[u8])` |
| Read node | `get_node(id) → str` | `getNode(id)` | `get_node_json` |
| Replace | `replace_text` | `replaceText` / `replace_text` | `replace_text` |
| Role | `set_role(..., variant=None)` | `setRole` / `set_role` | `set_role` |
| Insert | `insert_node(parent, index, node_json)` | `insertNode` (string or object) | `insert_node` |
| Delete | `delete_node` | `deleteNode` | `delete_node` |
| Diff | `diff() → str` | `diff() → array` | `diff` / `diff_json` |
| Agent id | `set_generated_by` | `setGeneratedBy` | `set_generated_by` |
| Running header/footer | `set_running_header` / `set_running_footer` | `setRunningHeader` / `setRunningFooter` | `set_running_header` / `set_running_footer` |
| Save | `save_bytes()` / `save_with(hash?)` | `save()` / `saveWith` | `save_bytes` / `save_with` |
| PDF from Editor | `export_pdf_bytes` | not on Editor — `exportPdf(bytes)` after save | `export_pdf_bytes` |
| Suggest | `suggest` / `accept_suggestion` / `reject_suggestion` | same (+ `suggestions()`) | same (+ `suggestions_json`) |
| Selection / clipboard | `selection` / `clipboard` → JSON str | same → objects | `selection` / `clipboard` |

CLI has **no** edit subcommand.

## `insert_node` vs JSON authoring

`insert_node` validates against the **agent dialect** (closed role list; layout mostly `columns`). It is **not** the full format schema. To add `stack` / `grid` / `overlay`, edit loose JSON and run `pack_verify.py` ([writing.md](../writing.md)).

## `save` / optimistic lock

- `save_with(expected_content_hash)` compares to the **baseline hash captured at open**, not a freshly recomputed tree hash.
- Mismatch → `CONTENT_HASH_MISMATCH` → re-`open` and retry.
- Non-empty `diff` → one `changelog.json` entry (SDK `generated_by` becomes `agent` when set).
- Empty diff → still relock/pack; no new changelog entry.
- Signatures always cleared on save.

## Script (deterministic)

```bash
python3 scripts/edit_and_verify.py \
  --file contract.K2F \
  --id contract.clause_4 \
  --text "Updated clause text." \
  --role critical_warning \
  --generated-by demo-agent \
  --out contract-edited.K2F
```

Run from this skill directory. Roles come from **that package's theme** — legal/contract packs often use `critical_warning`, not the SDK catalog name `warning`.

The CLI's `engine_commit_sha` should match the Editor that saved. Mismatch prints `ENGINE_MISMATCH`; the script still confirms the node via reopen.

## `replace_text`

Allowed content types: **`Text`** and **`Math`** (Math replaces TeX source).  
**Not** CodeBlock, Table, Image, Container → `WRONG_CONTENT`.

- Tables: replace text on **cell** node ids, never the table root.
- Lists: item ids often look like `{listId}.i{n}`; replace those text nodes.
- Running header/footer: same id APIs (outline includes running trees).
- Replacing text drops modifiers that no longer fit the new UTF-8 length.

## `set_role`

- Role and optional `variant` must exist in **this package's** theme (`UNKNOWN_ROLE` / unknown variant otherwise).
- Custom legal/contract themes often use **`critical_warning`**, not the SDK skin name `warning`.
- Side effects in core: `warning` / `critical_warning` / `signature_block` / `math` → `break_inside: avoid`; `h1`–`h4` → `keep_with_next`.

## `insert_node`

Minimal JSON:

```json
{"id":"contract.demo_clause","role":"body","content":{"type":"text","value":"Demo insert."}}
```

Rules:

- Parent must be a **container**; common parent id: `"root"`.
- **No** `x` / `y` (or other coordinate fields) → `INVALID_ARGUMENT`.
- Id must match dotted pattern; must be unique (`DUPLICATE_ID` / `INVALID_ID`).
- Insert JSON is validated against the **agent profile** role enum first:
  `document`, `section`, `h1`–`h4`, `body`, `warning`, `card`, `table`, `table_header_cell`, `table_row_cell`, `list_item`, `code`, `quote`, `rule`, `math`, `running_header`, `running_footer`, `signature_block`
  then against the package theme.
- Custom theme roles (e.g. `critical_warning`): **insert** with `body` or `warning`, then **`set_role`** to the custom role.
- Columns: insert a container with `layout: { "type": "columns", "count": 2, "gap": … }` (gap in millipt). Full-width child: `column_span: "all"`.
- **Grid / stack / overlay:** not via agent `insert_node` — edit author JSON ([writing.md](../writing.md), copy from `catalog/content/ex_*.json`).
- Images: Editor cannot upload new bytes; only insert an Image node if the asset path already exists in the package, or regenerate via [writing.md](../writing.md).

## `delete_node`

- Cannot delete **root** → `INVALID_ARGUMENT`.
- Removes from root tree or running-block trees.

## Content matrix

| Kind | How to edit (SDK) | How to edit (JSON — default) |
|------|-------------------|------------------------------|
| Text / Math | `replace_text` | Edit `content.value` in the node JSON |
| CodeBlock | Not via `replace_text`; delete + insert or regenerate | Edit or replace node in JSON |
| Table | Cell text ids; or delete table + insert new table JSON | Edit cell nodes in JSON |
| List item | `replace_text` on item; or insert/delete items | Edit item nodes in parent's `children` |
| Image | Delete/insert only; no byte swap API | Replace file under `assets/images/` + node path |
| Running header/footer | Same replace/set_role by id | Edit node in JSON |
| Columns region | Insert/delete container with `layout.type=columns` | Add container node in JSON |

## SDK errors

| Code | Typical cause |
|------|----------------|
| `UNKNOWN_ID` | Missing node or missing suggestion |
| `WRONG_CONTENT` | `replace_text` on non-Text/Math; insert under non-container |
| `UNKNOWN_ROLE` | Role/variant not in package theme |
| `DUPLICATE_ID` | Insert id already in tree |
| `INVALID_ID` | Empty or illegal dotted id |
| `INVALID_ARGUMENT` | Bad JSON, coordinates, bad index, delete root |
| `CONTENT_HASH_MISMATCH` | `save_with` expected ≠ open-time baseline |
| `SCHEMA_INVALID` / `FONT_MISSING` / `MATH_*` | Relock/compile failures — fix tree, never lock |

Pack/compile errors: [errors.md](errors.md).

## See also

- [writing.md](../writing.md) — default JSON authoring workflow
- [errors.md](errors.md) — compile and validation error codes
