# k2f_sdk

Agent-facing Rust SDK for building and editing `.K2F` documents.

- **`Editor`** — open templates, packages, or author dirs; edit by stable node id; diff/outline; suggestions
- **Official templates** — `templates/` + `copy_to` / `resolve`
- **Markdown bridge** — `markdown_to_k2f` / `k2f_to_markdown`
- **Signing** — Ed25519 key generation and `sign`

**License:** Apache-2.0

## Quick example

```rust
use k2f_sdk::Editor;

let mut ed = Editor::open_template("report")?;
ed.insert_node(
    "root",
    0,
    r#"{"id":"root.title","role":"h1","content":{"type":"text","value":"Quarterly update"}}"#,
)?;
ed.insert_node(
    "root",
    1,
    r#"{"id":"root.body","role":"body","content":{"type":"text","value":"Revenue increased."}}"#,
)?;
let bytes = ed.save_bytes()?;

let mut editor = Editor::open(&bytes)?;
editor.replace_text("root.body", "Revenue increased sharply.")?;
let updated = editor.save_bytes()?;
```

## Agent authoring profile

Agents should follow the dialect in [`instructions/agent_v0.md`](instructions/agent_v0.md) (also exposed as `k2f_sdk::SYSTEM_PROMPT`).

Public format contract: [k2f-v0.1.md](https://github.com/suzheng/k2f/blob/main/docs/spec/k2f-v0.1.md).

## Install

```bash
cargo add k2f_sdk
# CLI: cargo install k2f
```
