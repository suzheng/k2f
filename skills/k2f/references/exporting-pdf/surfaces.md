# Export surfaces

Load when the default `k2f export-pdf` path is not the one in use. All draw the published lock.

## CLI (default)

```bash
k2f export-pdf file.K2F -o out.pdf
k2f export-pdf file.K2F -o out.pdf --scale 4
```

Requires `pip install k2f`. Refuses PDF input (`PDF_IS_NOT_A_SOURCE`). Draws the published lock; does not compile. Optional `--scale 2|3|4` (default 2) for stamp pages only.

## Python (`import k2f`)

No module-level `export_pdf`.

| Object | Method | Lock |
|--------|--------|------|
| `Editor` | `export_pdf_bytes()` | Current package — **old** lock if not saved |

```python
import k2f

ed = k2f.Editor.open_bytes(open("report.K2F", "rb").read())
ed.replace_text("doc.body", "Updated.")
pdf = bytes(ed.export_pdf_bytes())  # stale until save_bytes()
open("report-edited.K2F", "wb").write(ed.save_bytes())
pdf = bytes(ed.export_pdf_bytes())
```

## JavaScript (`@openk2f/k2f`)

| Call | Lock |
|------|------|
| `exportPdf(bytes)` | Opens as Viewer — no recompile |
| `Viewer.export_pdf()` / `handle.exportPdf()` | Published lock |
| `Editor` | **No export API** — `save()` then `exportPdf(bytes)` |

## Desktop reader (not the agent default)

`k2f-reader --export-pdf out.pdf file.K2F` — prefer `k2f export-pdf`.
