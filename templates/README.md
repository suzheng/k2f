# `templates/` — checkout fixtures, not an SDK API

This directory is **in-repo test fixtures and examples** (empty author trees with fonts + theme). It is **not** part of the file format and **must not** ship inside published SDKs.

Do **not**:

- `include_dir!` this tree into `k2f_sdk` / WASM / npm (`@openk2f/k2f`)
- Export `official_templates`, `copy_template`, `resolve_template`, or `Editor.open_template` / `openTemplate`
- Teach agents to `open("invoice")` by name, or to assume `k2f/templates/` exists on their machine

The agent skill (`skills/k2f/`) never sees this folder. Create a shell with `init_package.py` + `skills/k2f/starter/`, unpack a `.K2F`, or Gallery; then `Editor.open_dir` / `Editor.open` / `open_bytes`. Markdown `--template` is an **author directory path**, not an id like `report`.

Keep these files in git so engine tests can `open_dir` them. Do **not** gitignore `templates/**` — ignoring the tree would break CI. Fonts here are fixtures, not a reason to bake them into WASM.
