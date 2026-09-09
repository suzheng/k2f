# k2f (Python)

Restricted document SDK for AI agents. Build `.K2F` packages, edit by stable node id, convert Markdown, and export PDF, PPTX, or DOCX from the published lock.

**License:** Apache-2.0

## Install

```bash
pip install k2f
```

`pip install k2f` puts the **`k2f` CLI on PATH** (pack, compile, verify, render, export-pdf, export-pptx, export-docx, markdown) and installs the Python SDK (`import k2f`).

From source (development):

```bash
# from repo root
bash scripts/dev-install.sh
# or
cd sdk/python && maturin develop --release
```

Requires Python 3.9+.

## Create a document

Open an author directory (from `init_package.py` or `k2f unpack`) or packed bytes, then insert semantic nodes as JSON:

```python
import json
import k2f

ed = k2f.Editor.open_dir("./source")
ed.insert_node(
    "root",
    0,
    json.dumps({
        "id": "invoice.title",
        "role": "h1",
        "content": {"type": "text", "value": "Invoice #1042"},
        "keep_with_next": True,
    }),
)
ed.insert_node(
    "root",
    1,
    json.dumps({
        "id": "invoice.body",
        "role": "body",
        "content": {"type": "text", "value": "Payment due in 30 days."},
    }),
)
ed.set_running_footer("Page {{page_current}} of {{page_total}}")
ed.validate_package()
open("invoice.K2F", "wb").write(ed.save_bytes())
```

## Edit an existing package

```python
import k2f

editor = k2f.Editor.open_bytes(open("invoice.K2F", "rb").read())
print(editor.outline())
editor.replace_text("invoice.body", "Updated copy.")
open("invoice-edited.K2F", "wb").write(editor.save_bytes())
```

## Markdown and signing

```python
import k2f

package = k2f.markdown_to_k2f("# Hello\n\nBody.", template="./source", title="From Markdown")
md = k2f.k2f_to_markdown(package)

secret, public, fingerprint = k2f.generate_signing_key()
signed = k2f.sign(package, secret, signed_by="agent@example.com")
```

## Agent system prompt

```python
import k2f

prompt = k2f.system_prompt()
```

## Format schemas

Reconcile against the embedded engine schemas (same bytes as `k2f schema dump -o …`):

```python
import json
import k2f

for path, text in k2f.format_schemas().items():
    json.loads(text)  # e.g. schema/nodes.schema.json
```

## Common errors

Runtime errors include stable codes in the message:

| Code | Meaning |
|------|---------|
| `DUPLICATE_ID` | Two nodes share the same dotted id |
| `PDF_IS_NOT_A_SOURCE` | PDF cannot be opened as a semantic source |
| `PPTX_IS_NOT_A_SOURCE` | PPTX cannot be opened as a semantic source |
| `DOCX_IS_NOT_A_SOURCE` | DOCX cannot be opened as a semantic source |
| `CONTENT_HASH_MISMATCH` | Save rejected — content hash drift |
| `UNKNOWN_ID` | Node id not found |
| `SCHEMA_INVALID` | JSON fails embedded schema validation |

## Development

```bash
cd sdk/python
python -m venv .venv && source .venv/bin/activate
maturin develop --release
pytest -q
maturin build --release
```

See the [repository README](https://github.com/suzheng/k2f#readme) for format overview.
